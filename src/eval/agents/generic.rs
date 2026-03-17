//! Generic agent runner for agents that follow the copy-to-skills-dir pattern.
//!
//! Most agents (Codex, Cursor, Goose, etc.) don't have a native `--skill` flag.
//! They load skills from a conventional directory (e.g., `.codex/skills/`).
//! This runner copies the skill into the agent's skills directory, runs the
//! agent in non-interactive mode, and cleans up afterward.
//!
//! The binary, flags, and skills directory are configurable per agent.

use crate::eval::agent::{execute_command, verify_binary, AgentError, AgentOutput, AgentRunner};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Configuration for a generic agent runner.
#[derive(Debug, Clone)]
pub struct GenericRunner {
    /// Path to the agent binary.
    pub bin: String,
    /// Display name for error messages.
    pub name: String,
    /// Non-interactive mode flags (e.g., `["--quiet"]`, `["-p"]`).
    pub non_interactive_flags: Vec<String>,
    /// Flag(s) to pass the prompt (empty means prompt is a positional arg).
    pub prompt_flags: Vec<String>,
    /// Model override flag name (e.g., `"--model"`).
    pub model_flag: Option<String>,
    /// Model value.
    pub model: Option<String>,
    /// Agent's skills directory (relative to project root).
    pub skills_dir: String,
}

impl GenericRunner {
    /// Build the base command with non-interactive flags.
    fn base_command(&self) -> Command {
        let mut cmd = Command::new(&self.bin);
        for flag in &self.non_interactive_flags {
            cmd.arg(flag);
        }

        if let (Some(flag), Some(model)) = (&self.model_flag, &self.model) {
            cmd.args([flag.as_str(), model.as_str()]);
        }

        cmd
    }

    /// Resolve the skills directory to an absolute path.
    ///
    /// Walks up from `skill_path` looking for the agent's config directory
    /// (the first component of `self.skills_dir`, e.g., `.codex`), falling
    /// back to the current working directory.
    fn resolve_skills_dir(&self, skill_path: &Path) -> PathBuf {
        // Extract the agent config dir (e.g., ".codex" from ".codex/skills").
        let agent_dir = Path::new(&self.skills_dir)
            .components()
            .next()
            .map(|c| c.as_os_str().to_string_lossy().to_string());

        if let Some(ref agent_dir) = agent_dir {
            let start = if skill_path.is_absolute() {
                skill_path.to_path_buf()
            } else {
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(skill_path)
            };
            let mut dir = start.as_path();
            while let Some(parent) = dir.parent() {
                if parent.join(agent_dir).exists() {
                    return parent.join(&self.skills_dir);
                }
                dir = parent;
            }
        }

        // Fallback: use CWD.
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        cwd.join(&self.skills_dir)
    }

    /// Install a skill by copying it into the agent's skills directory.
    fn install_skill(&self, skill_path: &Path) -> Result<PathBuf, AgentError> {
        let skill_name = skill_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| {
                AgentError::SkillSetupFailed("Invalid skill directory name".to_string())
            })?;

        let unique_id = format!("{}_{}", std::process::id(), timestamp_nanos());
        let installed_name = format!("_skilo_eval_{skill_name}_{unique_id}");
        let skills_dir = self.resolve_skills_dir(skill_path);
        let target = skills_dir.join(&installed_name);

        std::fs::create_dir_all(&skills_dir).map_err(|e| {
            AgentError::SkillSetupFailed(format!("Failed to create skills directory: {}", e))
        })?;

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

    /// Add the prompt to the command (using prompt_flags if set, else positional).
    fn add_prompt(&self, cmd: &mut Command, prompt: &str) {
        for flag in &self.prompt_flags {
            cmd.arg(flag);
        }
        cmd.arg(prompt);
    }
}

impl AgentRunner for GenericRunner {
    fn verify(&self) -> Result<(), AgentError> {
        verify_binary(&self.bin)
    }

    fn run_with_skill(
        &self,
        skill_path: &Path,
        prompt: &str,
        timeout_secs: u64,
    ) -> Result<AgentOutput, AgentError> {
        let installed_path = self.install_skill(skill_path)?;

        let mut cmd = self.base_command();
        self.add_prompt(&mut cmd, prompt);
        let result = execute_command(cmd, timeout_secs);

        Self::uninstall_skill(&installed_path);

        result
    }

    fn run_without_skill(
        &self,
        prompt: &str,
        timeout_secs: u64,
    ) -> Result<AgentOutput, AgentError> {
        let mut cmd = self.base_command();
        self.add_prompt(&mut cmd, prompt);
        execute_command(cmd, timeout_secs)
    }

    fn display_name(&self) -> &str {
        &self.name
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
    fn test_display_name() {
        let runner = GenericRunner {
            bin: "codex".into(),
            name: "Codex".into(),
            non_interactive_flags: vec!["--quiet".into()],
            prompt_flags: vec![],
            model_flag: Some("--model".into()),
            model: None,
            skills_dir: ".codex/skills".into(),
        };
        assert_eq!(runner.display_name(), "Codex");
    }
}
