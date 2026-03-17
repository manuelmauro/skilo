//! Test execution orchestration.

use crate::eval::agent::AgentRunner;
use crate::eval::categories::{functional, perf, trigger};
use crate::eval::EvalSuite;
use std::time::Duration;

/// Status of a test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestStatus {
    /// Test passed.
    Passed,
    /// Test failed.
    Failed,
    /// Test was skipped.
    Skipped,
    /// Test timed out.
    TimedOut,
}

/// Test category tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestCategory {
    /// Triggering test.
    Trigger,
    /// Functional test.
    Functional,
    /// Performance comparison test.
    Perf,
}

/// Result of a single test run.
#[derive(Debug, Clone)]
pub struct TestRunResult {
    /// Run number (1-indexed).
    pub run: u32,
    /// Status of this run.
    pub status: TestStatus,
    /// Wall-clock duration.
    pub duration: Duration,
    /// Output or message.
    pub output: String,
    /// Error message, if any.
    pub error: Option<String>,
}

/// Aggregated result of a test across all runs.
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Test name.
    pub name: String,
    /// Test category.
    pub category: TestCategory,
    /// Individual run results.
    pub runs: Vec<TestRunResult>,
    /// Overall status (passed if all runs passed).
    pub status: TestStatus,
}

impl TestResult {
    /// Total duration across all runs.
    pub fn total_duration(&self) -> Duration {
        self.runs.iter().map(|r| r.duration).sum()
    }

    /// Number of passed runs.
    pub fn passed_count(&self) -> usize {
        self.runs
            .iter()
            .filter(|r| r.status == TestStatus::Passed)
            .count()
    }
}

/// Options for running a suite.
#[derive(Default)]
pub struct RunOptions {
    /// Override number of runs per test.
    pub runs: Option<u32>,
    /// Override per-test timeout.
    pub timeout: Option<u64>,
    /// Run only tests matching this name.
    pub test_filter: Option<String>,
    /// Run only tests of this category.
    pub category_filter: Option<String>,
    /// Stop on first failure.
    pub fail_fast: bool,
    /// Show verbose output.
    pub verbose: bool,
}

/// Run a full eval suite and return results.
pub fn run_suite(
    suite: &EvalSuite,
    agent: &dyn AgentRunner,
    options: &RunOptions,
) -> Vec<TestResult> {
    let runs = options.runs.unwrap_or(suite.runs);
    let timeout = options.timeout.unwrap_or(suite.timeout);
    let mut results = Vec::new();

    let run_triggers = options
        .category_filter
        .as_ref()
        .is_none_or(|c| c == "trigger");
    let run_functional = options
        .category_filter
        .as_ref()
        .is_none_or(|c| c == "functional");
    let run_perf = options.category_filter.as_ref().is_none_or(|c| c == "perf");

    // Triggering tests.
    if run_triggers {
        for test in &suite.trigger_tests {
            if let Some(filter) = &options.test_filter {
                if !test.name.contains(filter.as_str()) {
                    continue;
                }
            }

            let run_results =
                trigger::run_trigger_test_multi(test, agent, &suite.skill_path, timeout, runs);
            let status = aggregate_status(&run_results);
            results.push(TestResult {
                name: test.name.clone(),
                category: TestCategory::Trigger,
                runs: run_results,
                status,
            });

            if options.fail_fast
                && results.last().is_some_and(|r| {
                    r.status == TestStatus::Failed || r.status == TestStatus::TimedOut
                })
            {
                return results;
            }
        }
    }

    // Functional tests.
    if run_functional {
        for test in &suite.functional_tests {
            if let Some(filter) = &options.test_filter {
                if !test.name.contains(filter.as_str()) {
                    continue;
                }
            }

            let run_results = functional::run_functional_test_multi(
                test,
                agent,
                &suite.skill_path,
                timeout,
                runs,
            );
            let status = aggregate_status(&run_results);
            results.push(TestResult {
                name: test.name.clone(),
                category: TestCategory::Functional,
                runs: run_results,
                status,
            });

            if options.fail_fast
                && results.last().is_some_and(|r| {
                    r.status == TestStatus::Failed || r.status == TestStatus::TimedOut
                })
            {
                return results;
            }
        }
    }

    // Performance tests.
    if run_perf {
        for test in &suite.perf_tests {
            if let Some(filter) = &options.test_filter {
                if !test.name.contains(filter.as_str()) {
                    continue;
                }
            }

            let run_results =
                perf::run_perf_test_multi(test, agent, &suite.skill_path, timeout, runs);
            let status = aggregate_status(&run_results);
            results.push(TestResult {
                name: test.name.clone(),
                category: TestCategory::Perf,
                runs: run_results,
                status,
            });

            if options.fail_fast
                && results.last().is_some_and(|r| {
                    r.status == TestStatus::Failed || r.status == TestStatus::TimedOut
                })
            {
                return results;
            }
        }
    }

    results
}

fn aggregate_status(runs: &[TestRunResult]) -> TestStatus {
    if runs.iter().all(|r| r.status == TestStatus::Passed) {
        TestStatus::Passed
    } else if runs.iter().all(|r| r.status == TestStatus::Skipped) {
        TestStatus::Skipped
    } else if runs.iter().any(|r| r.status == TestStatus::TimedOut) {
        TestStatus::TimedOut
    } else {
        TestStatus::Failed
    }
}
