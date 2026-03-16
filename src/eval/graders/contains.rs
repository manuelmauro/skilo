//! Contains / not-contains grader.

use super::GradeResult;

/// Grades output by checking for substring presence.
pub struct ContainsGrader;

impl ContainsGrader {
    /// Check that `output` contains `expected`.
    pub fn grade(output: &str, expected: &str) -> GradeResult {
        let passed = output.contains(expected);
        GradeResult {
            passed,
            message: if passed {
                format!("Output contains \"{}\"", expected)
            } else {
                format!("Expected output to contain \"{}\"", expected)
            },
        }
    }

    /// Check that `output` does NOT contain `unexpected`.
    pub fn grade_not(output: &str, unexpected: &str) -> GradeResult {
        let passed = !output.contains(unexpected);
        GradeResult {
            passed,
            message: if passed {
                format!("Output does not contain \"{}\"", unexpected)
            } else {
                format!("Expected output to NOT contain \"{}\"", unexpected)
            },
        }
    }
}
