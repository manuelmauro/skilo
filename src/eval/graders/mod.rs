//! Grading strategies for eval test outputs.

mod contains;
mod exact;
mod regex_grader;
mod script;

pub use contains::ContainsGrader;
pub use exact::ExactGrader;
pub use regex_grader::RegexGrader;
pub use script::ScriptGrader;

use crate::eval::Expectation;

/// Result of grading a single expectation.
#[derive(Debug, Clone)]
pub struct GradeResult {
    /// Whether the expectation was met.
    pub passed: bool,
    /// Human-readable explanation.
    pub message: String,
}

/// Grade an output against a list of expectations.
pub fn grade_output(
    output: &str,
    exit_code: i32,
    expectations: &[Expectation],
) -> Vec<GradeResult> {
    expectations
        .iter()
        .map(|exp| match exp {
            Expectation::Contains(s) => ContainsGrader::grade(output, s),
            Expectation::NotContains(s) => ContainsGrader::grade_not(output, s),
            Expectation::Regex(pattern) => RegexGrader::grade(output, pattern),
            Expectation::ExitCode(expected) => GradeResult {
                passed: exit_code == *expected,
                message: if exit_code == *expected {
                    format!("Exit code {} matches", expected)
                } else {
                    format!("Expected exit code {}, got {}", expected, exit_code)
                },
            },
            Expectation::Rubric(text) => {
                // Rubric expectations are for LLM grading — always pass in non-LLM mode.
                GradeResult {
                    passed: true,
                    message: format!("Rubric (skipped, LLM grader not active): {}", text),
                }
            }
        })
        .collect()
}
