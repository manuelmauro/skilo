//! Claude Code agent runner.
//!
//! Claude Code does not have a `--skill` flag for loading skills at runtime.
//! Instead, skills must be placed in `.claude/skills/` before invocation.
//! This runner copies the skill into a temporary `.claude/skills/` directory,
//! runs the agent, and cleans up afterward.
//!
//! Invocation: `claude -p --no-input [--model ...] <prompt>`

use crate::eval::agent::{execute_command, verify_binary, AgentError, AgentOutput, AgentRunner};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Configuration for the Claude Code agent runner.
#[derive(Debug, Clone)]
pub struct ClaudeRunner {
    /// Path to the claude binary.
    pub bin: String,
    /// Model override (passed to `claude --model`).
    pub model: Option<String>,
}

impl Default for ClaudeRunner {
    fn default() -> Self {
        Self {
            bin: "claude".into(),
            model: None,
        }
    }
}

impl ClaudeRunner {
    /// Build the base `claude` command with common flags.
    fn base_command(&self) -> Command {
        let mut cmd = Command::new(&self.bin);
        cmd.args(["-p", "--no-input"]);

        if let Some(model) = &self.model {
            cmd.args(["--model", model]);
        }

        cmd
    }

    /// Determine the skills directory for Claude Code.
    ///
    /// Resolves relative to the skill's project root by walking up from
    /// `skill_path` to find an existing `.claude/` directory, falling back
    /// to the current working directory.
    fn skills_dir(skill_path: &Path) -> PathBuf {
        // Walk up from the skill path looking for an existing .claude directory.
        let start = if skill_path.is_absolute() {
            skill_path.to_path_buf()
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(skill_path)
        };
        let mut dir = start.as_path();
        while let Some(parent) = dir.parent() {
            if parent.join(".claude").exists() {
                return parent.join(".claude").join("skills");
            }
            dir = parent;
        }
        // Fallback: use CWD.
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        cwd.join(".claude").join("skills")
    }

    /// Install a skill by copying it into Claude's skills directory.
    ///
    /// Returns the path to the installed skill (for cleanup).
    /// Uses PID + timestamp to create a unique directory name, preventing
    /// collisions when running concurrent evals.
    fn install_skill(skill_path: &Path) -> Result<PathBuf, AgentError> {
        let skill_name = skill_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| {
                AgentError::SkillSetupFailed("Invalid skill directory name".to_string())
            })?;

        // Use PID + timestamp for a unique staging directory.
        let unique_id = format!("{}_{}", std::process::id(), timestamp_nanos());
        let installed_name = format!("_skilo_eval_{skill_name}_{unique_id}");
        let target = Self::skills_dir(skill_path).join(&installed_name);

        // Create the skills directory if it doesn't exist.
        std::fs::create_dir_all(Self::skills_dir(skill_path)).map_err(|e| {
            AgentError::SkillSetupFailed(format!("Failed to create skills directory: {}", e))
        })?;

        // Copy the skill directory recursively.
        copy_dir_recursive(skill_path, &target)
            .map_err(|e| AgentError::SkillSetupFailed(format!("Failed to copy skill: {}", e)))?;

        Ok(target)
    }

    /// Remove an installed skill.
    fn uninstall_skill(installed_path: &Path) {
        if installed_path.exists() {
            let _ = std::fs::remove_dir_all(installed_path);
        }
    }
}

impl AgentRunner for ClaudeRunner {
    fn verify(&self) -> Result<(), AgentError> {
        verify_binary(&self.bin)
    }

    fn run_with_skill(
        &self,
        skill_path: &Path,
        prompt: &str,
        timeout_secs: u64,
    ) -> Result<AgentOutput, AgentError> {
        // Install the skill into Claude's skills directory.
        let installed_path = Self::install_skill(skill_path)?;

        // Run the agent.
        let mut cmd = self.base_command();
        cmd.arg(prompt);
        let result = execute_command(cmd, timeout_secs);

        // Always clean up, even if the run failed.
        Self::uninstall_skill(&installed_path);

        result
    }

    fn run_without_skill(
        &self,
        prompt: &str,
        timeout_secs: u64,
    ) -> Result<AgentOutput, AgentError> {
        let mut cmd = self.base_command();
        cmd.arg(prompt);
        execute_command(cmd, timeout_secs)
    }

    fn display_name(&self) -> &str {
        "Claude Code"
    }
}

/// Return a nanosecond timestamp for unique naming.
fn timestamp_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

/// Recursively copy a directory, skipping symlinks to prevent traversal.
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());

        if file_type.is_symlink() {
            // Skip symlinks to avoid traversal outside the skill directory.
            continue;
        } else if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dst_path)?;
        } else if file_type.is_file() {
            std::fs::copy(entry.path(), &dst_path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let runner = ClaudeRunner::default();
        assert_eq!(runner.bin, "claude");
        assert!(runner.model.is_none());
    }

    #[test]
    fn test_display_name() {
        let runner = ClaudeRunner::default();
        assert_eq!(runner.display_name(), "Claude Code");
    }
}
