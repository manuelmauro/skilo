//! Remove installed skills.

use crate::agent::Agent;
use crate::cli::{Cli, RemoveArgs};
use crate::config::Config;
use crate::error::SkiloError;
use crate::output::get_formatter;
use crate::scope::Scope;
use colored::Colorize;
use dialoguer::Confirm;
use std::path::PathBuf;

/// Run the remove command.
pub fn run(args: RemoveArgs, config: &Config, cli: &Cli) -> Result<i32, SkiloError> {
    let formatter = get_formatter(cli.format, cli.quiet);
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Determine scope
    let scope = if args.global {
        Scope::Global
    } else {
        Scope::Project
    };

    // Determine agent (None means use ./skills/)
    let agent: Option<Agent> = match args.agent.as_ref().map(|a| a.to_selection()) {
        Some(crate::cli::AgentSelection::Single(a)) => Some(a),
        Some(crate::cli::AgentSelection::All) => config.add.default_agent,
        None => config.add.default_agent,
    };

    // Resolve skills directory
    let skills_dir = match agent {
        Some(agent) => match scope {
            Scope::Global => agent.resolve_global_skills_dir().ok_or_else(|| {
                SkiloError::Config("Could not determine global skills directory".to_string())
            })?,
            Scope::Project => agent.resolve_project_skills_dir(&project_root),
        },
        None => {
            if args.global {
                return Err(SkiloError::Config(
                    "Global removal requires an agent (use --agent)".to_string(),
                ));
            }
            project_root.join("skills")
        }
    };

    if !skills_dir.exists() {
        let target = agent
            .map(|a| a.display_name().to_string())
            .unwrap_or_else(|| "skills/".to_string());
        formatter.format_error(&format!("Skills directory does not exist for {}", target));
        return Ok(1);
    }

    // Find skills to remove
    let mut to_remove: Vec<(String, PathBuf)> = Vec::new();
    let mut not_found: Vec<String> = Vec::new();

    for skill_name in &args.skills {
        let skill_path = skills_dir.join(skill_name);
        if skill_path.exists() && skill_path.join("SKILL.md").exists() {
            to_remove.push((skill_name.clone(), skill_path));
        } else {
            not_found.push(skill_name.clone());
        }
    }

    // Report not found skills
    if !not_found.is_empty() && !cli.quiet {
        for name in &not_found {
            eprintln!("{}: Skill '{}' not found", "Warning".yellow(), name);
        }
    }

    if to_remove.is_empty() {
        formatter.format_error("No skills to remove");
        return Ok(1);
    }

    // Confirm removal
    if !args.yes {
        println!();
        println!("Skills to remove:");
        for (name, path) in &to_remove {
            println!(
                "  {} ({})",
                name.cyan(),
                path.display().to_string().dimmed()
            );
        }
        println!();

        let prompt = format!(
            "Remove {} skill{}?",
            to_remove.len(),
            if to_remove.len() == 1 { "" } else { "s" }
        );

        if !Confirm::new()
            .with_prompt(prompt)
            .interact()
            .map_err(|_| SkiloError::Cancelled)?
        {
            return Err(SkiloError::Cancelled);
        }
        println!();
    }

    // Remove skills
    let mut removed = 0;
    for (name, path) in &to_remove {
        if !cli.quiet {
            print!("Removing {}...", name.cyan());
        }

        match std::fs::remove_dir_all(path) {
            Ok(()) => {
                removed += 1;
                if !cli.quiet {
                    println!(" {}", "done".green());
                }
            }
            Err(e) => {
                if !cli.quiet {
                    println!(" {}", "failed".red());
                }
                formatter.format_error(&format!("Failed to remove '{}': {}", name, e));
            }
        }
    }

    if !cli.quiet {
        println!();
        formatter.format_success(&format!(
            "Removed {} skill{}",
            removed,
            if removed == 1 { "" } else { "s" }
        ));
    }

    if removed == to_remove.len() {
        Ok(0)
    } else {
        Ok(1)
    }
}
