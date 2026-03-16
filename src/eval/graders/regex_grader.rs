//! Regex pattern grader.

use super::GradeResult;

/// Grades output by regex pattern matching.
pub struct RegexGrader;

impl RegexGrader {
    /// Check that `output` matches the given regex `pattern`.
    pub fn grade(output: &str, pattern: &str) -> GradeResult {
        match regex::Regex::new(pattern) {
            Ok(re) => {
                let passed = re.is_match(output);
                GradeResult {
                    passed,
                    message: if passed {
                        format!("Output matches regex /{}/", pattern)
                    } else {
                        format!("Expected output to match regex /{}/", pattern)
                    },
                }
            }
            Err(e) => GradeResult {
                passed: false,
                message: format!("Invalid regex /{}/ : {}", pattern, e),
            },
        }
    }
}
