//! The `eval` command implementation.

use crate::cli::{Cli, EvalAgent, EvalArgs, EvalCommand, EvalFormat};
use crate::config::Config;
use crate::error::SkiloError;
use crate::eval::agents::{ClaudeRunner, GenericRunner, PiRunner};
use crate::eval::{
    format_results, run_suite, scaffold_eval, AgentRunner, EvalSuite, ReportFormat, RunOptions,
    TestStatus,
};
use crate::skill::Discovery;

/// Run the eval command.
pub fn run(args: EvalArgs, config: &Config, cli: &Cli) -> Result<i32, SkiloError> {
    match &args.command {
        Some(EvalCommand::Init(init_args)) => run_init(&init_args.path),
        None => run_eval(args, config, cli),
    }
}

fn run_init(path: &std::path::Path) -> Result<i32, SkiloError> {
    // If path points to a skill directory (has SKILL.md), scaffold there.
    // Otherwise, find skills under the path.
    let skill_paths = if path.join("SKILL.md").exists() {
        vec![path.to_path_buf()]
    } else {
        Discovery::find_skills(path, &[])
            .into_iter()
            .map(|p| p.parent().unwrap_or(path).to_path_buf())
            .collect()
    };

    if skill_paths.is_empty() {
        return Err(SkiloError::NoSkillsFound {
            path: path.display().to_string(),
        });
    }

    for skill_path in &skill_paths {
        match scaffold_eval(skill_path) {
            Ok(created) => {
                for path in &created {
                    println!("Created {}", path);
                }
            }
            Err(e) => {
                eprintln!("Error scaffolding {}: {}", skill_path.display(), e);
            }
        }
    }

    Ok(0)
}

fn run_eval(args: EvalArgs, config: &Config, cli: &Cli) -> Result<i32, SkiloError> {
    let path = &args.path;

    // Find skills with EVAL.md files.
    let eval_paths = find_eval_files(path, &config.discovery.ignore);

    if eval_paths.is_empty() {
        eprintln!("No EVAL.md files found in {}", path.display());
        return Ok(2);
    }

    // Build agent config from args + config + suite frontmatter.
    let report_format = match args.eval_format {
        Some(EvalFormat::Json) => ReportFormat::Json,
        Some(EvalFormat::Markdown) => ReportFormat::Markdown,
        Some(EvalFormat::Text) | None => ReportFormat::Text,
    };

    let mut overall_exit = 0;

    for eval_path in &eval_paths {
        // Parse the EVAL.md.
        let suite = match EvalSuite::parse(eval_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error parsing {}: {}", eval_path.display(), e);
                overall_exit = 2;
                continue;
            }
        };

        if args.dry_run {
            let agent_name = resolve_agent_name(&args, &suite, config);
            println!(
                "Parsed: {} ({} tests, agent: {})",
                suite.name,
                suite.total_tests(),
                agent_name
            );
            continue;
        }

        // Build the agent runner with CLI overrides.
        let agent = build_agent_runner(&args, &suite, config);

        // Verify agent is available.
        if let Err(e) = agent.verify() {
            eprintln!("Agent error ({}): {}", agent.display_name(), e);
            return Ok(4);
        }

        // Build run options.
        let options = RunOptions {
            runs: args.runs,
            timeout: args.timeout,
            test_filter: args.test.clone(),
            category_filter: args.category.clone(),
            fail_fast: args.fail_fast,
            verbose: args.verbose && !cli.quiet,
        };

        // Run the suite.
        let results = run_suite(&suite, agent.as_ref(), &options);

        // Report results.
        let skill_name = suite
            .skill_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&suite.name);

        let output = format_results(skill_name, &results, report_format);
        print!("{}", output);

        // Check for failures.
        if results.iter().any(|r| r.status == TestStatus::Failed) {
            overall_exit = 1;
        }
    }

    Ok(overall_exit)
}

/// Resolve which agent to use from CLI args > EVAL.md frontmatter > config > default.
fn resolve_agent_name(args: &EvalArgs, suite: &EvalSuite, config: &Config) -> String {
    // CLI --agent takes highest priority.
    if let Some(agent) = &args.agent {
        return eval_agent_to_cli_name(agent).to_string();
    }

    // EVAL.md frontmatter `agent:` field.
    if let Some(agent) = &suite.agent {
        return agent.clone();
    }

    // Config file `[eval] agent` field.
    if let Some(agent) = &config.eval.default_agent {
        return agent.clone();
    }

    // Default to pi-mono.
    "pi-mono".to_string()
}

/// Build the appropriate agent runner based on resolved agent selection.
fn build_agent_runner(args: &EvalArgs, suite: &EvalSuite, config: &Config) -> Box<dyn AgentRunner> {
    let agent_name = resolve_agent_name(args, suite, config);

    // Resolve model/provider/thinking with CLI > EVAL.md > config precedence.
    let model = args
        .model
        .clone()
        .or_else(|| suite.model.clone())
        .or_else(|| config.eval.default_model.clone());
    let provider = args
        .provider
        .clone()
        .or_else(|| suite.provider.clone())
        .or_else(|| config.eval.default_provider.clone());
    let thinking = args
        .thinking
        .clone()
        .or_else(|| suite.thinking.clone())
        .or_else(|| config.eval.default_thinking.clone());

    match agent_name.as_str() {
        "pi-mono" | "pi" => Box::new(PiRunner {
            bin: config.eval.agent_bin.clone().unwrap_or_else(|| "pi".into()),
            model,
            provider,
            thinking,
        }),
        "claude" => Box::new(ClaudeRunner {
            bin: config
                .eval
                .agent_bin
                .clone()
                .unwrap_or_else(|| "claude".into()),
            model,
        }),
        "codex" => Box::new(GenericRunner {
            bin: config
                .eval
                .agent_bin
                .clone()
                .unwrap_or_else(|| "codex".into()),
            name: "Codex".into(),
            non_interactive_flags: vec!["--quiet".into()],
            prompt_flags: vec![],
            model_flag: Some("--model".into()),
            model,
            skills_dir: ".codex/skills".into(),
        }),
        "cursor" => Box::new(GenericRunner {
            bin: config
                .eval
                .agent_bin
                .clone()
                .unwrap_or_else(|| "cursor".into()),
            name: "Cursor".into(),
            non_interactive_flags: vec![],
            prompt_flags: vec![],
            model_flag: Some("--model".into()),
            model,
            skills_dir: ".cursor/skills".into(),
        }),
        "amp" => Box::new(GenericRunner {
            bin: config
                .eval
                .agent_bin
                .clone()
                .unwrap_or_else(|| "amp".into()),
            name: "Amp".into(),
            non_interactive_flags: vec![],
            prompt_flags: vec![],
            model_flag: Some("--model".into()),
            model,
            skills_dir: ".agents/skills".into(),
        }),
        "goose" => Box::new(GenericRunner {
            bin: config
                .eval
                .agent_bin
                .clone()
                .unwrap_or_else(|| "goose".into()),
            name: "Goose".into(),
            non_interactive_flags: vec!["run".into()],
            prompt_flags: vec![],
            model_flag: Some("--model".into()),
            model,
            skills_dir: ".goose/skills".into(),
        }),
        "gemini" => Box::new(GenericRunner {
            bin: config
                .eval
                .agent_bin
                .clone()
                .unwrap_or_else(|| "gemini".into()),
            name: "Gemini CLI".into(),
            non_interactive_flags: vec![],
            prompt_flags: vec![],
            model_flag: Some("--model".into()),
            model,
            skills_dir: ".gemini/skills".into(),
        }),
        "opencode" => Box::new(GenericRunner {
            bin: config
                .eval
                .agent_bin
                .clone()
                .unwrap_or_else(|| "opencode".into()),
            name: "OpenCode".into(),
            non_interactive_flags: vec![],
            prompt_flags: vec![],
            model_flag: Some("--model".into()),
            model,
            skills_dir: ".opencode/skill".into(),
        }),
        "kilocode" => Box::new(GenericRunner {
            bin: config
                .eval
                .agent_bin
                .clone()
                .unwrap_or_else(|| "kilocode".into()),
            name: "Kilo Code".into(),
            non_interactive_flags: vec![],
            prompt_flags: vec![],
            model_flag: Some("--model".into()),
            model,
            skills_dir: ".kilocode/skills".into(),
        }),
        "roocode" => Box::new(GenericRunner {
            bin: config
                .eval
                .agent_bin
                .clone()
                .unwrap_or_else(|| "roocode".into()),
            name: "Roo Code".into(),
            non_interactive_flags: vec![],
            prompt_flags: vec![],
            model_flag: Some("--model".into()),
            model,
            skills_dir: ".roo/skills".into(),
        }),
        "antigravity" => Box::new(GenericRunner {
            bin: config
                .eval
                .agent_bin
                .clone()
                .unwrap_or_else(|| "antigravity".into()),
            name: "Antigravity".into(),
            non_interactive_flags: vec![],
            prompt_flags: vec![],
            model_flag: Some("--model".into()),
            model,
            skills_dir: ".agent/skills".into(),
        }),
        "copilot" => Box::new(GenericRunner {
            bin: config
                .eval
                .agent_bin
                .clone()
                .unwrap_or_else(|| "copilot".into()),
            name: "GitHub Copilot".into(),
            non_interactive_flags: vec![],
            prompt_flags: vec![],
            model_flag: Some("--model".into()),
            model,
            skills_dir: ".github/skills".into(),
        }),
        "windsurf" => Box::new(GenericRunner {
            bin: config
                .eval
                .agent_bin
                .clone()
                .unwrap_or_else(|| "windsurf".into()),
            name: "Windsurf".into(),
            non_interactive_flags: vec![],
            prompt_flags: vec![],
            model_flag: Some("--model".into()),
            model,
            skills_dir: ".windsurf/skills".into(),
        }),
        // Fallback: treat unknown agent names as pi-mono.
        other => {
            eprintln!(
                "Warning: Unknown agent '{}', falling back to Pi Mono",
                other
            );
            Box::new(PiRunner {
                bin: config.eval.agent_bin.clone().unwrap_or_else(|| "pi".into()),
                model,
                provider,
                thinking,
            })
        }
    }
}

/// Map EvalAgent enum variant to the CLI name used in agent resolution.
fn eval_agent_to_cli_name(agent: &EvalAgent) -> &'static str {
    match agent {
        EvalAgent::PiMono => "pi-mono",
        EvalAgent::Claude => "claude",
        EvalAgent::Codex => "codex",
        EvalAgent::Cursor => "cursor",
        EvalAgent::Amp => "amp",
        EvalAgent::Goose => "goose",
        EvalAgent::Gemini => "gemini",
        EvalAgent::OpenCode => "opencode",
        EvalAgent::KiloCode => "kilocode",
        EvalAgent::RooCode => "roocode",
        EvalAgent::Antigravity => "antigravity",
        EvalAgent::Copilot => "copilot",
        EvalAgent::Windsurf => "windsurf",
    }
}

fn find_eval_files(path: &std::path::Path, ignore: &[String]) -> Vec<std::path::PathBuf> {
    // If the path itself is an EVAL.md, use it directly.
    if path.is_file() && path.file_name().is_some_and(|n| n == "EVAL.md") {
        return vec![path.to_path_buf()];
    }

    // If the path is a skill directory, check for EVAL.md.
    let eval_in_dir = path.join("EVAL.md");
    if eval_in_dir.exists() {
        return vec![eval_in_dir];
    }

    // Otherwise discover skills and look for EVAL.md in each.
    Discovery::find_skills(path, ignore)
        .into_iter()
        .filter_map(|skill_md| {
            let skill_dir = skill_md.parent()?;
            let eval_path = skill_dir.join("EVAL.md");
            if eval_path.exists() {
                Some(eval_path)
            } else {
                None
            }
        })
        .collect()
}
