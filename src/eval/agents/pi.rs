//! Pi Mono agent runner.
//!
//! Pi has native `--skill` flag support, making it the simplest runner.
//! Invocation: `pi --print --no-session [--skill <path>] [--model ...] <prompt>`

use crate::eval::agent::{execute_command, verify_binary, AgentError, AgentOutput, AgentRunner};
use std::path::Path;
use std::process::Command;

/// Configuration for the Pi Mono agent runner.
#[derive(Debug, Clone)]
pub struct PiRunner {
    /// Path to the pi binary.
    pub bin: String,
    /// Model override (passed to `pi --model`).
    pub model: Option<String>,
    /// Provider override (passed to `pi --provider`).
    pub provider: Option<String>,
    /// Thinking level override (passed to `pi --thinking`).
    pub thinking: Option<String>,
}

impl Default for PiRunner {
    fn default() -> Self {
        Self {
            bin: "pi".into(),
            model: None,
            provider: None,
            thinking: None,
        }
    }
}

impl PiRunner {
    /// Build the base `pi` command with common flags.
    fn base_command(&self) -> Command {
        let mut cmd = Command::new(&self.bin);
        cmd.args(["--print", "--no-session"]);

        if let Some(model) = &self.model {
            cmd.args(["--model", model]);
        }
        if let Some(provider) = &self.provider {
            cmd.args(["--provider", provider]);
        }
        if let Some(thinking) = &self.thinking {
            cmd.args(["--thinking", thinking]);
        }

        cmd
    }
}

impl AgentRunner for PiRunner {
    fn verify(&self) -> Result<(), AgentError> {
        verify_binary(&self.bin)
    }

    fn run_with_skill(
        &self,
        skill_path: &Path,
        prompt: &str,
        timeout_secs: u64,
    ) -> Result<AgentOutput, AgentError> {
        let mut cmd = self.base_command();
        cmd.args(["--skill", &skill_path.display().to_string()]);
        cmd.arg(prompt);
        execute_command(cmd, timeout_secs)
    }

    fn run_without_skill(
        &self,
        prompt: &str,
        timeout_secs: u64,
    ) -> Result<AgentOutput, AgentError> {
        let mut cmd = self.base_command();
        cmd.arg("--no-skills");
        cmd.arg(prompt);
        execute_command(cmd, timeout_secs)
    }

    fn display_name(&self) -> &str {
        "Pi Mono"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let runner = PiRunner::default();
        assert_eq!(runner.bin, "pi");
        assert!(runner.model.is_none());
        assert!(runner.provider.is_none());
        assert!(runner.thinking.is_none());
    }

    #[test]
    fn test_display_name() {
        let runner = PiRunner::default();
        assert_eq!(runner.display_name(), "Pi Mono");
    }
}
