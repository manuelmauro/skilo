//! Exact match grader.

use super::GradeResult;

/// Grades output by exact string match.
pub struct ExactGrader;

impl ExactGrader {
    /// Check that `output` exactly matches `expected`.
    pub fn grade(output: &str, expected: &str) -> GradeResult {
        let passed = output.trim() == expected.trim();
        GradeResult {
            passed,
            message: if passed {
                "Output matches exactly".into()
            } else {
                format!(
                    "Expected exact match.\n  Expected: \"{}\"\n  Got:      \"{}\"",
                    truncate(expected, 80),
                    truncate(output, 80)
                )
            },
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}
