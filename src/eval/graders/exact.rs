//! Exact match grader.

use super::GradeResult;

/// Grades output by exact string match.
pub struct ExactGrader;

impl ExactGrader {
    /// Check that `output` exactly matches `expected`.
    pub fn grade(output: &str, expected: &str) -> GradeResult {
        let passed = output == expected;
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

/// Truncate a string to at most `max` characters (not bytes), appending "..."
/// if truncated. Safe for multibyte UTF-8.
fn truncate(s: &str, max: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max).collect();
        format!("{}...", truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let result = ExactGrader::grade("hello", "hello");
        assert!(result.passed);
    }

    #[test]
    fn test_whitespace_sensitive() {
        let result = ExactGrader::grade("hello ", "hello");
        assert!(!result.passed);
    }

    #[test]
    fn test_truncate_multibyte() {
        // Should not panic on multibyte characters.
        let s = "héllo wörld 🎉 test";
        let t = truncate(s, 5);
        assert_eq!(t, "héllo...");
    }
}
