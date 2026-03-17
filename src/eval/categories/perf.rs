//! Performance comparison test runner.
//!
//! Runs the same prompt with and without the skill, comparing metrics.

use crate::eval::agent::{AgentError, AgentRunner};
use crate::eval::runner::{TestRunResult, TestStatus};
use crate::eval::{PerfAssertion, PerfMetric, PerfTest};
use std::path::Path;
use std::time::Duration;

/// Collected metrics from an agent run (using available proxies).
#[derive(Debug, Clone)]
pub struct RunMetrics {
    /// Output length as a proxy for token usage.
    pub output_length: u64,
    /// Wall-clock duration in milliseconds.
    pub duration_ms: u64,
    /// Exit code (0 = success).
    pub exit_code: i32,
}

/// Result of a perf comparison.
#[derive(Debug)]
pub struct PerfComparison {
    /// Skill run metrics.
    pub skill: RunMetrics,
    /// Baseline (no-skill) run metrics.
    pub baseline: RunMetrics,
    /// Per-metric assertion results.
    pub assertions: Vec<(String, bool, String)>,
}

/// Run a single performance comparison test.
pub fn run_perf_test(
    test: &PerfTest,
    agent: &dyn AgentRunner,
    skill_path: &Path,
    timeout: u64,
) -> Result<TestRunResult, AgentError> {
    // Run with skill.
    let skill_output = agent.run_with_skill(skill_path, &test.prompt, timeout)?;
    let skill_metrics = RunMetrics {
        output_length: skill_output.stdout.len() as u64,
        duration_ms: skill_output.duration.as_millis() as u64,
        exit_code: skill_output.exit_code,
    };

    // Run without skill (baseline).
    let baseline_output = agent.run_without_skill(&test.prompt, timeout)?;
    let baseline_metrics = RunMetrics {
        output_length: baseline_output.stdout.len() as u64,
        duration_ms: baseline_output.duration.as_millis() as u64,
        exit_code: baseline_output.exit_code,
    };

    let total_duration = skill_output.duration + baseline_output.duration;

    // Evaluate metric assertions.
    let mut assertions = Vec::new();
    let mut all_passed = true;

    for metric in &test.metrics {
        let (name, passed, msg) = evaluate_metric(metric, &skill_metrics, &baseline_metrics);
        if !passed {
            all_passed = false;
        }
        assertions.push((name, passed, msg));
    }

    let status = if all_passed {
        TestStatus::Passed
    } else {
        TestStatus::Failed
    };

    let output_lines: Vec<String> = assertions
        .iter()
        .map(|(name, passed, msg)| {
            let icon = if *passed { "✓" } else { "✗" };
            format!("{} {}: {}", icon, name, msg)
        })
        .collect();

    Ok(TestRunResult {
        run: 1,
        status,
        duration: total_duration,
        output: output_lines.join("\n    "),
        error: None,
    })
}

fn evaluate_metric(
    metric: &PerfMetric,
    skill: &RunMetrics,
    baseline: &RunMetrics,
) -> (String, bool, String) {
    match metric {
        PerfMetric::TokenUsage(assertion) => {
            // Use output_length as proxy.
            evaluate_assertion(
                "token_usage",
                skill.output_length,
                baseline.output_length,
                assertion,
            )
        }
        PerfMetric::MessageCount(assertion) => {
            // Not directly measurable via agent output; use a placeholder.
            evaluate_assertion("message_count", 0, 0, assertion)
        }
        PerfMetric::ToolCallCount(assertion) => {
            evaluate_assertion("tool_call_count", 0, 0, assertion)
        }
        PerfMetric::ErrorCount(assertion) => {
            let skill_errors = if skill.exit_code != 0 { 1 } else { 0 };
            let baseline_errors = if baseline.exit_code != 0 { 1 } else { 0 };
            evaluate_assertion("error_count", skill_errors, baseline_errors, assertion)
        }
    }
}

fn evaluate_assertion(
    name: &str,
    skill_value: u64,
    baseline_value: u64,
    assertion: &PerfAssertion,
) -> (String, bool, String) {
    match assertion {
        PerfAssertion::LessThanBaseline => {
            let passed = skill_value < baseline_value;
            let msg = format!(
                "{} vs {} baseline ({})",
                skill_value,
                baseline_value,
                if passed {
                    format!("↓ {}%", percentage_decrease(baseline_value, skill_value))
                } else {
                    "not improved".into()
                }
            );
            (name.to_string(), passed, msg)
        }
        PerfAssertion::Equals(expected) => {
            let passed = skill_value == *expected;
            let msg = format!("{} (target: {})", skill_value, expected);
            (name.to_string(), passed, msg)
        }
    }
}

fn percentage_decrease(baseline: u64, current: u64) -> u64 {
    if baseline == 0 {
        return 0;
    }
    ((baseline.saturating_sub(current)) * 100) / baseline
}

/// Run a perf test with multiple runs and return all results.
pub fn run_perf_test_multi(
    test: &PerfTest,
    agent: &dyn AgentRunner,
    skill_path: &Path,
    timeout: u64,
    runs: u32,
) -> Vec<TestRunResult> {
    let mut results = Vec::new();
    for run_num in 1..=runs {
        match run_perf_test(test, agent, skill_path, timeout) {
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
