//! Triggering test runner.
//!
//! Runs a prompt with the skill loaded and checks whether the skill was
//! activated by inspecting the agent output.

use crate::eval::agent::{AgentError, AgentRunner};
use crate::eval::runner::{TestRunResult, TestStatus};
use crate::eval::TriggerTest;
use std::path::Path;
use std::time::Duration;

/// Run a single triggering test.
pub fn run_trigger_test(
    test: &TriggerTest,
    agent: &dyn AgentRunner,
    skill_path: &Path,
    timeout: u64,
) -> Result<TestRunResult, AgentError> {
    let output = agent.run_with_skill(skill_path, &test.prompt, timeout)?;

    // Heuristic: if the skill was triggered, the output should reference
    // skill-related content. We compare with-skill vs without-skill output
    // lengths and content as a proxy for activation.
    let baseline = agent.run_without_skill(&test.prompt, timeout)?;

    let skill_was_triggered = detect_skill_activation(&output.stdout, &baseline.stdout);

    let passed = if test.should_trigger {
        skill_was_triggered
    } else {
        !skill_was_triggered
    };

    let status = if passed {
        TestStatus::Passed
    } else {
        TestStatus::Failed
    };

    let message = if test.should_trigger && !skill_was_triggered {
        "Skill did NOT trigger (expected it to trigger)".to_string()
    } else if !test.should_trigger && skill_was_triggered {
        "Skill DID trigger (expected it to NOT trigger)".to_string()
    } else if test.should_trigger {
        "triggered".to_string()
    } else {
        "not triggered".to_string()
    };

    Ok(TestRunResult {
        run: 1,
        status,
        duration: output.duration,
        output: message,
        error: None,
    })
}

/// Detect if a skill was activated by comparing with-skill vs without-skill output.
fn detect_skill_activation(with_skill: &str, without_skill: &str) -> bool {
    // Simple heuristic: if outputs differ significantly, the skill was activated.
    // A more sophisticated approach would parse pi's structured output.
    if with_skill == without_skill {
        return false;
    }

    // Check for significant difference (more than trivial variance).
    let len_diff = (with_skill.len() as i64 - without_skill.len() as i64).unsigned_abs();
    let max_len = with_skill.len().max(without_skill.len()).max(1) as u64;
    let relative_diff = (len_diff * 100) / max_len;

    // If the outputs differ by more than 10%, consider the skill activated.
    relative_diff > 10
}

/// Run a trigger test with multiple runs and return all results.
pub fn run_trigger_test_multi(
    test: &TriggerTest,
    agent: &dyn AgentRunner,
    skill_path: &Path,
    timeout: u64,
    runs: u32,
) -> Vec<TestRunResult> {
    let mut results = Vec::new();
    for run_num in 1..=runs {
        match run_trigger_test(test, agent, skill_path, timeout) {
            Ok(mut result) => {
                result.run = run_num;
                results.push(result);
            }
            Err(e) => {
                results.push(TestRunResult {
                    run: run_num,
                    status: TestStatus::Failed,
                    duration: Duration::ZERO,
                    output: String::new(),
                    error: Some(e.to_string()),
                });
            }
        }
    }
    results
}
