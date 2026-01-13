//! JSON output formatter.

use super::OutputFormatter;
use crate::skill::{Diagnostic, ValidationResult};
use serde::Serialize;

/// Formatter that outputs JSON.
pub struct JsonFormatter {
    quiet: bool,
}

impl JsonFormatter {
    /// Create a new JSON formatter.
    pub fn new(quiet: bool) -> Self {
        Self { quiet }
    }
}

#[derive(Serialize)]
struct JsonOutput {
    skills: Vec<SkillResult>,
    summary: Summary,
}

#[derive(Serialize)]
struct SkillResult {
    path: String,
    errors: Vec<JsonDiagnostic>,
    warnings: Vec<JsonDiagnostic>,
}

#[derive(Serialize)]
struct JsonDiagnostic {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    column: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fix_hint: Option<String>,
}

#[derive(Serialize)]
struct Summary {
    skills_checked: usize,
    total_errors: usize,
    total_warnings: usize,
    success: bool,
}

impl From<&Diagnostic> for JsonDiagnostic {
    fn from(diag: &Diagnostic) -> Self {
        Self {
            code: diag.code.to_string(),
            message: diag.message.clone(),
            line: diag.line,
            column: diag.column,
            fix_hint: diag.fix_hint.clone(),
        }
    }
}

impl OutputFormatter for JsonFormatter {
    fn format_validation(&self, results: &[(String, ValidationResult)]) -> String {
        let skills: Vec<SkillResult> = results
            .iter()
            .map(|(path, result)| SkillResult {
                path: path.clone(),
                errors: result.errors.iter().map(Into::into).collect(),
                warnings: result.warnings.iter().map(Into::into).collect(),
            })
            .collect();

        let total_errors: usize = results.iter().map(|(_, r)| r.errors.len()).sum();
        let total_warnings: usize = results.iter().map(|(_, r)| r.warnings.len()).sum();

        let output = JsonOutput {
            skills,
            summary: Summary {
                skills_checked: results.len(),
                total_errors,
                total_warnings,
                success: total_errors == 0,
            },
        };

        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
    }

    fn format_message(&self, message: &str) {
        if !self.quiet {
            let obj = serde_json::json!({ "message": message });
            println!("{}", serde_json::to_string(&obj).unwrap());
        }
    }

    fn format_error(&self, message: &str) {
        let obj = serde_json::json!({ "error": message });
        eprintln!("{}", serde_json::to_string(&obj).unwrap());
    }

    fn format_success(&self, message: &str) {
        if !self.quiet {
            let obj = serde_json::json!({ "success": true, "message": message });
            println!("{}", serde_json::to_string(&obj).unwrap());
        }
    }
}
