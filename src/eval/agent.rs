//! Agent runner trait and shared types.
//!
//! Defines the interface that all agent harnesses must implement,
//! plus shared output/error types.

use std::path::Path;
use std::process::Command;
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
/// Stdout and stderr are drained in dedicated threads to avoid deadlocks
/// when the child's pipe buffers fill. On timeout the child is killed and
/// reaped to prevent zombie processes.
pub fn execute_command(mut cmd: Command, timeout_secs: u64) -> Result<AgentOutput, AgentError> {
    let start = Instant::now();

    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(AgentError::SpawnFailed)?;

    // Take ownership of the pipes and drain them in background threads
    // so the child never blocks on a full buffer.
    let stdout_pipe = child.stdout.take();
    let stderr_pipe = child.stderr.take();

    let stdout_handle = std::thread::spawn(move || -> Vec<u8> {
        let mut buf = Vec::new();
        if let Some(mut pipe) = stdout_pipe {
            use std::io::Read;
            let _ = pipe.read_to_end(&mut buf);
        }
        buf
    });

    let stderr_handle = std::thread::spawn(move || -> Vec<u8> {
        let mut buf = Vec::new();
        if let Some(mut pipe) = stderr_pipe {
            use std::io::Read;
            let _ = pipe.read_to_end(&mut buf);
        }
        buf
    });

    let timeout = Duration::from_secs(timeout_secs);
    let poll_interval = Duration::from_millis(100);

    // Poll the child process with a timeout.
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if start.elapsed() > timeout {
                    // Kill the child and reap it to avoid zombies.
                    let _ = child.kill();
                    let _ = child.wait();
                    // Join reader threads so they don't leak.
                    let _ = stdout_handle.join();
                    let _ = stderr_handle.join();
                    return Err(AgentError::Timeout(timeout_secs));
                }
                std::thread::sleep(poll_interval);
            }
            Err(e) => return Err(AgentError::SpawnFailed(e)),
        }
    };

    // Child exited — collect buffered output from reader threads.
    let stdout = stdout_handle.join().unwrap_or_default();
    let stderr = stderr_handle.join().unwrap_or_default();
    let duration = start.elapsed();

    Ok(AgentOutput {
        stdout: String::from_utf8_lossy(&stdout).to_string(),
        stderr: String::from_utf8_lossy(&stderr).to_string(),
        exit_code: status.code().unwrap_or(-1),
        duration,
    })
}

/// Verify that a binary is available on PATH.
///
/// Uses `which` on Unix and `where` on Windows.
pub fn verify_binary(bin: &str) -> Result<(), AgentError> {
    let lookup_cmd = if cfg!(windows) { "where" } else { "which" };
    let output = Command::new(lookup_cmd)
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
