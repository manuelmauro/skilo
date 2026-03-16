//! Functional test runner.
//!
//! Runs a prompt with the skill loaded and grades the output against expectations.

use crate::eval::agent::{AgentConfig, AgentError};
use crate::eval::graders::grade_output;
use crate::eval::runner::{TestRunResult, TestStatus};
use crate::eval::FunctionalTest;
use std::path::Path;
use std::time::Duration;

/// Run a single functional test.
pub fn run_functional_test(
    test: &FunctionalTest,
    agent: &AgentConfig,
    skill_path: &Path,
    timeout: u64,
) -> Result<TestRunResult, AgentError> {
    // Build the full prompt including input fixtures.
    let full_prompt = build_prompt(test);

    let output = agent.run_with_skill(skill_path, &full_prompt, timeout)?;

    // Grade the output.
    let grades = grade_output(&output.stdout, output.exit_code, &test.expected);

    let all_passed = grades.iter().all(|g| g.passed);
    let status = if all_passed {
        TestStatus::Passed
    } else {
        TestStatus::Failed
    };

    let messages: Vec<String> = grades
        .iter()
        .filter(|g| !g.passed)
        .map(|g| g.message.clone())
        .collect();

    let output_text = if messages.is_empty() {
        "passed".to_string()
    } else {
        messages.join("\n    ")
    };

    Ok(TestRunResult {
        run: 1,
        status,
        duration: output.duration,
        output: output_text,
        error: if output.exit_code != 0 && !output.stderr.is_empty() {
            Some(output.stderr.clone())
        } else {
            None
        },
    })
}

/// Build the full prompt from the test definition, including input fixtures.
fn build_prompt(test: &FunctionalTest) -> String {
    let mut parts = vec![test.prompt.clone()];

    for input_path in &test.inputs {
        if let Ok(content) = std::fs::read_to_string(input_path) {
            parts.push(format!(
                "\n--- Input: {} ---\n{}",
                input_path.display(),
                content
            ));
        }
    }

    parts.join("\n")
}

/// Run a functional test with multiple runs and return all results.
pub fn run_functional_test_multi(
    test: &FunctionalTest,
    agent: &AgentConfig,
    skill_path: &Path,
    timeout: u64,
    runs: u32,
) -> Vec<TestRunResult> {
    let mut results = Vec::new();
    for run_num in 1..=runs {
        match run_functional_test(test, agent, skill_path, timeout) {
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
