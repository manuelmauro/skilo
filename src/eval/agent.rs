//! Pi agent subprocess management.

use std::path::Path;
use std::process::{Command, Output};
use std::time::{Duration, Instant};
use thiserror::Error;

/// Errors from agent invocation.
#[derive(Debug, Error)]
pub enum AgentError {
    /// The agent binary was not found.
    #[error("Agent binary '{bin}' not found. Is pi installed?")]
    NotFound {
        /// The binary name.
        bin: String,
    },

    /// The agent process failed to start.
    #[error("Failed to start agent: {0}")]
    SpawnFailed(#[from] std::io::Error),

    /// The agent process timed out.
    #[error("Agent timed out after {0} seconds")]
    Timeout(u64),
}

/// Configuration for invoking the pi agent.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Path to the pi binary.
    pub bin: String,
    /// Model override (passed to `pi --model`).
    pub model: Option<String>,
    /// Provider override (passed to `pi --provider`).
    pub provider: Option<String>,
    /// Thinking level override (passed to `pi --thinking`).
    pub thinking: Option<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            bin: "pi".into(),
            model: None,
            provider: None,
            thinking: None,
        }
    }
}

/// Result of an agent invocation.
#[derive(Debug)]
pub struct AgentOutput {
    /// Stdout from the agent.
    pub stdout: String,
    /// Stderr from the agent.
    pub stderr: String,
    /// Process exit code.
    pub exit_code: i32,
    /// Wall-clock duration of the run.
    pub duration: Duration,
}

impl AgentConfig {
    /// Check that the agent binary is available.
    pub fn verify(&self) -> Result<(), AgentError> {
        let output = Command::new("which")
            .arg(&self.bin)
            .output()
            .map_err(AgentError::SpawnFailed)?;

        if !output.status.success() {
            return Err(AgentError::NotFound {
                bin: self.bin.clone(),
            });
        }
        Ok(())
    }

    /// Run the agent with a skill loaded.
    pub fn run_with_skill(
        &self,
        skill_path: &Path,
        prompt: &str,
        timeout_secs: u64,
    ) -> Result<AgentOutput, AgentError> {
        let mut cmd = self.base_command();
        cmd.args(["--skill", &skill_path.display().to_string()]);
        cmd.arg(prompt);
        self.execute(cmd, timeout_secs)
    }

    /// Run the agent without any skills (for baseline).
    pub fn run_without_skill(
        &self,
        prompt: &str,
        timeout_secs: u64,
    ) -> Result<AgentOutput, AgentError> {
        let mut cmd = self.base_command();
        cmd.arg("--no-skills");
        cmd.arg(prompt);
        self.execute(cmd, timeout_secs)
    }

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

    fn execute(&self, mut cmd: Command, timeout_secs: u64) -> Result<AgentOutput, AgentError> {
        let start = Instant::now();

        let mut child = cmd
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(AgentError::SpawnFailed)?;

        let timeout = Duration::from_secs(timeout_secs);
        let output: Output = match wait_with_timeout(&mut child, timeout) {
            Some(result) => result.map_err(AgentError::SpawnFailed)?,
            None => {
                let _ = child.kill();
                return Err(AgentError::Timeout(timeout_secs));
            }
        };

        let duration = start.elapsed();

        Ok(AgentOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            duration,
        })
    }
}

/// Wait for a child process with a timeout.
fn wait_with_timeout(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Option<std::io::Result<Output>> {
    let start = Instant::now();
    let poll_interval = Duration::from_millis(100);

    loop {
        match child.try_wait() {
            Ok(Some(_status)) => {
                // Process exited — collect output.
                let stdout = child.stdout.take().map_or_else(Vec::new, |mut s| {
                    let mut buf = Vec::new();
                    std::io::Read::read_to_end(&mut s, &mut buf).unwrap_or(0);
                    buf
                });
                let stderr = child.stderr.take().map_or_else(Vec::new, |mut s| {
                    let mut buf = Vec::new();
                    std::io::Read::read_to_end(&mut s, &mut buf).unwrap_or(0);
                    buf
                });
                return Some(Ok(Output {
                    status: _status,
                    stdout,
                    stderr,
                }));
            }
            Ok(None) => {
                // Still running.
                if start.elapsed() > timeout {
                    return None;
                }
                std::thread::sleep(poll_interval);
            }
            Err(e) => return Some(Err(e)),
        }
    }
}
