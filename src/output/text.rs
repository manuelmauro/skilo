use super::OutputFormatter;
use crate::skill::ValidationResult;
use colored::Colorize;

pub struct TextFormatter {
    quiet: bool,
}

impl TextFormatter {
    pub fn new(quiet: bool) -> Self {
        Self { quiet }
    }
}

impl OutputFormatter for TextFormatter {
    fn format_validation(&self, results: &[(String, ValidationResult)]) -> String {
        let mut output = String::new();

        for (skill_path, result) in results {
            if !result.errors.is_empty() || !result.warnings.is_empty() {
                output.push_str(&format!("\n{}\n", skill_path.bold()));

                for diag in &result.errors {
                    let location = match (diag.line, diag.column) {
                        (Some(line), Some(col)) => format!("{}:{}", line, col),
                        (Some(line), None) => format!("{}:", line),
                        _ => String::new(),
                    };

                    output.push_str(&format!(
                        "  {} {} {}: {}\n",
                        "error".red().bold(),
                        format!("[{}]", diag.code).dimmed(),
                        location.dimmed(),
                        diag.message
                    ));

                    if let Some(hint) = &diag.fix_hint {
                        output.push_str(&format!("    {} {}\n", "hint:".cyan(), hint));
                    }
                }

                for diag in &result.warnings {
                    let location = match (diag.line, diag.column) {
                        (Some(line), Some(col)) => format!("{}:{}", line, col),
                        (Some(line), None) => format!("{}:", line),
                        _ => String::new(),
                    };

                    output.push_str(&format!(
                        "  {} {} {}: {}\n",
                        "warning".yellow().bold(),
                        format!("[{}]", diag.code).dimmed(),
                        location.dimmed(),
                        diag.message
                    ));

                    if let Some(hint) = &diag.fix_hint {
                        output.push_str(&format!("    {} {}\n", "hint:".cyan(), hint));
                    }
                }
            }
        }

        // Summary
        let total_errors: usize = results.iter().map(|(_, r)| r.errors.len()).sum();
        let total_warnings: usize = results.iter().map(|(_, r)| r.warnings.len()).sum();
        let skills_checked = results.len();

        output.push('\n');
        if total_errors == 0 && total_warnings == 0 {
            output.push_str(&format!(
                "{} {} skill(s) checked, no issues found\n",
                "✓".green().bold(),
                skills_checked
            ));
        } else {
            output.push_str(&format!(
                "{} {} skill(s) checked: {} error(s), {} warning(s)\n",
                if total_errors > 0 {
                    "✗".red()
                } else {
                    "!".yellow()
                },
                skills_checked,
                total_errors,
                total_warnings
            ));
        }

        output
    }

    fn format_message(&self, message: &str) {
        if !self.quiet {
            println!("{}", message);
        }
    }

    fn format_error(&self, message: &str) {
        eprintln!("{} {}", "error:".red().bold(), message);
    }

    fn format_success(&self, message: &str) {
        if !self.quiet {
            println!("{} {}", "✓".green().bold(), message);
        }
    }
}
