//! Agent runner trait and shared types.
//!
//! Defines the interface that all agent harnesses must implement,
//! plus shared output/error types.

use std::path::Path;
use std::process::{Command, Output};
use std::time::{Duration, Instant};
use thiserror::Error;

/// Errors from agent invocation.
#[derive(Debug, Error)]
pub enum AgentError {
    /// The agent binary was not found.
    #[error("Agent binary '{bin}' not found. Is it installed?")]
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

    /// Skill installation/setup failed.
    #[error("Failed to set up skill for agent: {0}")]
    SkillSetupFailed(String),

    /// Skill cleanup failed (non-fatal).
    #[error("Failed to clean up skill after test: {0}")]
    CleanupFailed(String),
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

/// Trait for agent harness implementations.
///
/// Each supported AI coding agent implements this trait to define how
/// to invoke it with and without skills loaded.
pub trait AgentRunner: Send + Sync {
    /// Check that the agent binary is available.
    fn verify(&self) -> Result<(), AgentError>;

    /// Run the agent with a skill loaded.
    fn run_with_skill(
        &self,
        skill_path: &Path,
        prompt: &str,
        timeout_secs: u64,
    ) -> Result<AgentOutput, AgentError>;

    /// Run the agent without any skills (for baseline comparison).
    fn run_without_skill(&self, prompt: &str, timeout_secs: u64)
        -> Result<AgentOutput, AgentError>;

    /// Display name for this agent runner (for error messages and logs).
    fn display_name(&self) -> &str;
}

/// Execute a command with a timeout and return the output.
///
/// Shared helper used by all agent runner implementations.
pub fn execute_command(mut cmd: Command, timeout_secs: u64) -> Result<AgentOutput, AgentError> {
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

/// Verify that a binary is available on PATH.
pub fn verify_binary(bin: &str) -> Result<(), AgentError> {
    let output = Command::new("which")
        .arg(bin)
        .output()
        .map_err(AgentError::SpawnFailed)?;

    if !output.status.success() {
        return Err(AgentError::NotFound {
            bin: bin.to_string(),
        });
    }
    Ok(())
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
