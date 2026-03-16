//! The `eval` command implementation.

use crate::cli::{Cli, EvalArgs, EvalCommand};
use crate::config::Config;
use crate::error::SkiloError;
use crate::eval::{
    format_results, run_suite, scaffold_eval, AgentConfig, EvalSuite, ReportFormat, RunOptions,
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
    let report_format = match args.eval_format.as_deref() {
        Some("json") => ReportFormat::Json,
        Some("markdown") => ReportFormat::Markdown,
        _ => ReportFormat::Text,
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
            println!("Parsed: {} ({} tests)", suite.name, suite.total_tests());
            continue;
        }

        // Build agent config with CLI overrides.
        let agent = build_agent_config(&args, &suite, config);

        // Verify agent is available.
        if let Err(e) = agent.verify() {
            eprintln!("Agent error: {}", e);
            return Ok(4);
        }

        // Build run options.
        let options = RunOptions {
            runs: args.runs,
            timeout: args.timeout,
            test_filter: args.test.clone(),
            category_filter: args.category.clone(),
            fail_fast: args.fail_fast,
            verbose: args.verbose || !cli.quiet,
        };

        // Run the suite.
        let results = run_suite(&suite, &agent, &options);

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

fn build_agent_config(args: &EvalArgs, suite: &EvalSuite, config: &Config) -> AgentConfig {
    AgentConfig {
        bin: config.eval.agent_bin.clone().unwrap_or_else(|| "pi".into()),
        model: args
            .model
            .clone()
            .or_else(|| suite.model.clone())
            .or_else(|| config.eval.default_model.clone()),
        provider: args
            .provider
            .clone()
            .or_else(|| suite.provider.clone())
            .or_else(|| config.eval.default_provider.clone()),
        thinking: args
            .thinking
            .clone()
            .or_else(|| suite.thinking.clone())
            .or_else(|| config.eval.default_thinking.clone()),
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
