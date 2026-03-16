//! Evaluation framework for Agent Skills.
//!
//! Parses `EVAL.md` files and orchestrates test execution via the Pi agent.

mod agent;
mod parser;
mod report;
mod runner;
mod scaffold;

pub mod categories;
pub mod graders;

pub use agent::AgentConfig;
pub use parser::{
    EvalSuite, Expectation, FunctionalTest, GraderKind, PerfAssertion, PerfMetric, PerfTest,
    TriggerTest,
};
pub use report::{format_results, ReportFormat};
pub use runner::{run_suite, RunOptions, TestResult, TestRunResult, TestStatus};
pub use scaffold::scaffold_eval;
