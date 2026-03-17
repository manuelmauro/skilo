//! External script grader.

use super::GradeResult;
use std::path::Path;
use std::process::Command;

/// Grades output by running an external script.
pub struct ScriptGrader;

impl ScriptGrader {
    /// Run the grading script with `output` on stdin.
    /// Exit code 0 = pass, non-zero = fail.
    pub fn grade(output: &str, script_path: &Path) -> GradeResult {
        if !script_path.exists() {
            return GradeResult {
                passed: false,
                message: format!("Grader script not found: {}", script_path.display()),
            };
        }

        let result = Command::new(script_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                if let Some(ref mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    stdin.write_all(output.as_bytes())?;
                    // Drop stdin to signal EOF so the child can proceed.
                }
                child.wait_with_output()
            });

        match result {
            Ok(out) => {
                let passed = out.status.success();
                let stderr = String::from_utf8_lossy(&out.stderr);
                GradeResult {
                    passed,
                    message: if passed {
                        format!("Script grader passed: {}", script_path.display())
                    } else {
                        format!(
                            "Script grader failed: {}\n  {}",
                            script_path.display(),
                            stderr.trim()
                        )
                    },
                }
            }
            Err(e) => GradeResult {
                passed: false,
                message: format!(
                    "Failed to run grader script {}: {}",
                    script_path.display(),
                    e
                ),
            },
        }
    }
}
