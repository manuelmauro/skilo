//! Output formatting for eval results.

use crate::eval::runner::{TestCategory, TestResult, TestStatus};
use std::time::Duration;

/// Report output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    /// Human-readable text.
    Text,
    /// JSON.
    Json,
    /// Markdown table.
    Markdown,
}

/// Format eval results for output.
pub fn format_results(skill_name: &str, results: &[TestResult], format: ReportFormat) -> String {
    match format {
        ReportFormat::Text => format_text(skill_name, results),
        ReportFormat::Json => format_json(skill_name, results),
        ReportFormat::Markdown => format_markdown(skill_name, results),
    }
}

// ── Text format ───────────────────────────────────────────────────

fn format_text(skill_name: &str, results: &[TestResult]) -> String {
    let total = results.len();
    let passed = results
        .iter()
        .filter(|r| r.status == TestStatus::Passed)
        .count();
    let failed = results
        .iter()
        .filter(|r| r.status == TestStatus::Failed || r.status == TestStatus::TimedOut)
        .count();

    let total_duration: Duration = results.iter().map(|r| r.total_duration()).sum();

    let mut out = String::new();

    // Header.
    out.push_str(&format!("Evaluating: {} ({} tests)\n\n", skill_name, total));

    // Group by category.
    let triggers: Vec<_> = results
        .iter()
        .filter(|r| r.category == TestCategory::Trigger)
        .collect();
    let functionals: Vec<_> = results
        .iter()
        .filter(|r| r.category == TestCategory::Functional)
        .collect();
    let perfs: Vec<_> = results
        .iter()
        .filter(|r| r.category == TestCategory::Perf)
        .collect();

    if !triggers.is_empty() {
        out.push_str("Triggering:\n");
        for r in &triggers {
            out.push_str(&format_text_result(r));
        }
        out.push('\n');
    }

    if !functionals.is_empty() {
        out.push_str("Functional:\n");
        for r in &functionals {
            out.push_str(&format_text_result(r));
        }
        out.push('\n');
    }

    if !perfs.is_empty() {
        out.push_str("Performance:\n");
        for r in &perfs {
            out.push_str(&format_text_result(r));
        }
        out.push('\n');
    }

    // Summary.
    out.push_str(&format!(
        "Results: {} passed, {} failed, {} total ({:.1}s)\n",
        passed,
        failed,
        total,
        total_duration.as_secs_f64()
    ));

    out
}

fn format_text_result(result: &TestResult) -> String {
    let icon = match result.status {
        TestStatus::Passed => "✓",
        TestStatus::Failed => "✗",
        TestStatus::Skipped => "⊘",
        TestStatus::TimedOut => "⏱",
    };

    let duration = format!("{:.1}s", result.total_duration().as_secs_f64());

    let detail = if let Some(first_run) = result.runs.first() {
        &first_run.output
    } else {
        ""
    };

    let mut line = format!(
        "  {} {} {} ({})",
        icon,
        pad_right(&result.name, 30),
        detail,
        duration
    );

    // Show errors for failed tests.
    if result.status == TestStatus::Failed {
        for run in &result.runs {
            if let Some(ref err) = run.error {
                line.push_str(&format!("\n    Error: {}", err));
            }
        }
    }

    line.push('\n');
    line
}

fn pad_right(s: &str, width: usize) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        format!("{}{}", s, ".".repeat(width - s.len()))
    }
}

// ── JSON format ───────────────────────────────────────────────────

fn format_json(skill_name: &str, results: &[TestResult]) -> String {
    let total = results.len();
    let passed = results
        .iter()
        .filter(|r| r.status == TestStatus::Passed)
        .count();
    let failed = results
        .iter()
        .filter(|r| r.status == TestStatus::Failed || r.status == TestStatus::TimedOut)
        .count();
    let total_duration_ms: u128 = results.iter().map(|r| r.total_duration().as_millis()).sum();

    let count_by = |cat: TestCategory, passed_only: bool| -> usize {
        results
            .iter()
            .filter(|r| r.category == cat && (!passed_only || r.status == TestStatus::Passed))
            .count()
    };

    let trigger_total = count_by(TestCategory::Trigger, false);
    let trigger_passed = count_by(TestCategory::Trigger, true);
    let func_total = count_by(TestCategory::Functional, false);
    let func_passed = count_by(TestCategory::Functional, true);
    let perf_total = count_by(TestCategory::Perf, false);
    let perf_passed = count_by(TestCategory::Perf, true);

    let tests_json: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            let cat = match r.category {
                TestCategory::Trigger => "trigger",
                TestCategory::Functional => "functional",
                TestCategory::Perf => "perf",
            };
            let status = status_str(&r.status);
            let runs: Vec<serde_json::Value> = r
                .runs
                .iter()
                .map(|run| {
                    serde_json::json!({
                        "run": run.run,
                        "status": status_str(&run.status),
                        "duration_ms": run.duration.as_millis() as u64,
                    })
                })
                .collect();
            serde_json::json!({
                "name": r.name,
                "category": cat,
                "status": status,
                "runs": runs,
            })
        })
        .collect();

    let report = serde_json::json!({
        "skill": skill_name,
        "categories": {
            "trigger": { "total": trigger_total, "passed": trigger_passed, "failed": trigger_total - trigger_passed },
            "functional": { "total": func_total, "passed": func_passed, "failed": func_total - func_passed },
            "perf": { "total": perf_total, "passed": perf_passed, "failed": perf_total - perf_passed },
        },
        "tests": tests_json,
        "summary": { "total": total, "passed": passed, "failed": failed, "duration_ms": total_duration_ms as u64 },
    });

    serde_json::to_string(&report).unwrap_or_else(|_| "{}".to_string())
}

fn status_str(status: &TestStatus) -> &'static str {
    match status {
        TestStatus::Passed => "passed",
        TestStatus::Failed => "failed",
        TestStatus::Skipped => "skipped",
        TestStatus::TimedOut => "timed_out",
    }
}

// ── Markdown format ───────────────────────────────────────────────

fn format_markdown(skill_name: &str, results: &[TestResult]) -> String {
    let mut out = String::new();

    out.push_str(&format!("## Eval Results: {}\n\n", skill_name));

    let triggers: Vec<_> = results
        .iter()
        .filter(|r| r.category == TestCategory::Trigger)
        .collect();
    let functionals: Vec<_> = results
        .iter()
        .filter(|r| r.category == TestCategory::Functional)
        .collect();
    let perfs: Vec<_> = results
        .iter()
        .filter(|r| r.category == TestCategory::Perf)
        .collect();

    if !triggers.is_empty() {
        let passed = triggers
            .iter()
            .filter(|r| r.status == TestStatus::Passed)
            .count();
        out.push_str(&format!(
            "### Triggering ({}/{} passed)\n\n",
            passed,
            triggers.len()
        ));
        out.push_str("| Test | Status | Duration |\n");
        out.push_str("| ---- | ------ | -------- |\n");
        for r in &triggers {
            let icon = status_icon(&r.status);
            out.push_str(&format!(
                "| {} | {} | {:.1}s |\n",
                escape_md_cell(&r.name),
                icon,
                r.total_duration().as_secs_f64()
            ));
        }
        out.push('\n');
    }

    if !functionals.is_empty() {
        let passed = functionals
            .iter()
            .filter(|r| r.status == TestStatus::Passed)
            .count();
        out.push_str(&format!(
            "### Functional ({}/{} passed)\n\n",
            passed,
            functionals.len()
        ));
        out.push_str("| Test | Status | Duration | Runs |\n");
        out.push_str("| ---- | ------ | -------- | ---- |\n");
        for r in &functionals {
            let icon = status_icon(&r.status);
            out.push_str(&format!(
                "| {} | {} | {:.1}s | {}/{} |\n",
                escape_md_cell(&r.name),
                icon,
                r.total_duration().as_secs_f64(),
                r.passed_count(),
                r.runs.len()
            ));
        }
        out.push('\n');
    }

    if !perfs.is_empty() {
        let passed = perfs
            .iter()
            .filter(|r| r.status == TestStatus::Passed)
            .count();
        out.push_str(&format!(
            "### Performance ({}/{} passed)\n\n",
            passed,
            perfs.len()
        ));
        out.push_str("| Test | Status | Duration |\n");
        out.push_str("| ---- | ------ | -------- |\n");
        for r in &perfs {
            let icon = status_icon(&r.status);
            out.push_str(&format!(
                "| {} | {} | {:.1}s |\n",
                escape_md_cell(&r.name),
                icon,
                r.total_duration().as_secs_f64()
            ));
        }
        out.push('\n');
    }

    out
}

/// Escape a string for use in a Markdown table cell.
fn escape_md_cell(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

fn status_icon(status: &TestStatus) -> &'static str {
    match status {
        TestStatus::Passed => "✓ Pass",
        TestStatus::Failed => "✗ Fail",
        TestStatus::Skipped => "⊘ Skip",
        TestStatus::TimedOut => "⏱ Timeout",
    }
}
