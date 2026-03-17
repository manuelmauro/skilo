//! EVAL.md parsing — frontmatter and test case extraction.

use serde::Deserialize;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur when parsing an EVAL.md file.
#[derive(Debug, Error)]
pub enum EvalParseError {
    /// The EVAL.md file is missing the YAML frontmatter delimiter.
    #[error("EVAL.md must start with YAML frontmatter (---)")]
    MissingFrontmatter,

    /// The YAML frontmatter is not properly closed.
    #[error("Frontmatter is not closed (missing closing ---)")]
    UnclosedFrontmatter,

    /// The YAML frontmatter contains invalid YAML.
    #[error("Invalid YAML in frontmatter: {0}")]
    InvalidYaml(#[from] serde_yaml::Error),

    /// An I/O error occurred while reading the file.
    #[error("IO error reading {path}: {source}")]
    Io {
        /// The path that failed to read.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// A test section has an invalid format.
    #[error("Invalid test section '{name}': {reason}")]
    InvalidSection {
        /// Section name.
        name: String,
        /// Why it's invalid.
        reason: String,
    },
}

/// EVAL.md YAML frontmatter.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct EvalFrontmatter {
    /// Evaluation suite name.
    pub name: String,
    /// Description of what is being evaluated.
    pub description: String,
    /// Agent to use for evaluation (e.g., "pi-mono", "claude", "codex").
    /// If not specified, defaults to the configured agent or "pi-mono".
    pub agent: Option<String>,
    /// Model to pass to the agent's `--model` flag.
    pub model: Option<String>,
    /// Provider to pass to the agent's `--provider` flag.
    pub provider: Option<String>,
    /// Thinking level to pass to the agent's `--thinking` flag.
    pub thinking: Option<String>,
    /// Number of times to run each test.
    pub runs: u32,
    /// Per-test timeout in seconds.
    pub timeout: u64,
    /// Default grader strategy.
    pub grader: String,
}

impl Default for EvalFrontmatter {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            agent: None,
            model: None,
            provider: None,
            thinking: None,
            runs: 1,
            timeout: 60,
            grader: "contains".into(),
        }
    }
}

/// A parsed evaluation suite from EVAL.md.
#[derive(Debug, Clone)]
pub struct EvalSuite {
    /// Path to the EVAL.md file.
    pub path: PathBuf,
    /// Path to the skill directory (parent of EVAL.md).
    pub skill_path: PathBuf,
    /// Evaluation suite name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Agent to use (e.g., "pi-mono", "claude", "codex").
    pub agent: Option<String>,
    /// Model override.
    pub model: Option<String>,
    /// Provider override.
    pub provider: Option<String>,
    /// Thinking level override.
    pub thinking: Option<String>,
    /// Number of runs per test.
    pub runs: u32,
    /// Per-test timeout in seconds.
    pub timeout: u64,
    /// Default grader.
    pub default_grader: GraderKind,
    /// Triggering tests.
    pub trigger_tests: Vec<TriggerTest>,
    /// Functional tests.
    pub functional_tests: Vec<FunctionalTest>,
    /// Performance comparison tests.
    pub perf_tests: Vec<PerfTest>,
}

/// A triggering test case.
#[derive(Debug, Clone)]
pub struct TriggerTest {
    /// Test name.
    pub name: String,
    /// Prompt to send.
    pub prompt: String,
    /// Whether the skill should trigger.
    pub should_trigger: bool,
}

/// A functional test case.
#[derive(Debug, Clone)]
pub struct FunctionalTest {
    /// Test name.
    pub name: String,
    /// Prompt to send.
    pub prompt: String,
    /// Input fixture files.
    pub inputs: Vec<PathBuf>,
    /// Expected outcomes.
    pub expected: Vec<Expectation>,
    /// Grader override for this test.
    pub grader: Option<GraderKind>,
}

/// A performance comparison test case.
#[derive(Debug, Clone)]
pub struct PerfTest {
    /// Test name.
    pub name: String,
    /// Prompt to send.
    pub prompt: String,
    /// Performance metric assertions.
    pub metrics: Vec<PerfMetric>,
}

/// Grading strategy.
#[derive(Debug, Clone)]
pub enum GraderKind {
    /// Output must match expected value exactly.
    Exact,
    /// Output must contain all expected substrings.
    Contains,
    /// Output must match provided regex patterns.
    Regex,
    /// Use an LLM to judge output against rubric.
    Llm,
    /// Run a custom script.
    Script(PathBuf),
}

/// A single expectation within a test.
#[derive(Debug, Clone)]
pub enum Expectation {
    /// Output must contain this substring.
    Contains(String),
    /// Output must not contain this substring.
    NotContains(String),
    /// Output must match this regex.
    Regex(String),
    /// Process must exit with this code.
    ExitCode(i32),
    /// Rubric for LLM grading.
    Rubric(String),
}

/// A performance metric and its assertion.
#[derive(Debug, Clone)]
pub enum PerfMetric {
    /// Total tokens consumed.
    TokenUsage(PerfAssertion),
    /// Number of back-and-forth messages.
    MessageCount(PerfAssertion),
    /// Number of tool/MCP calls.
    ToolCallCount(PerfAssertion),
    /// Number of failed API/tool calls.
    ErrorCount(PerfAssertion),
}

/// How a performance metric should compare to the baseline.
#[derive(Debug, Clone)]
pub enum PerfAssertion {
    /// Must be less than the baseline value.
    LessThanBaseline,
    /// Must equal this specific value.
    Equals(u64),
}

impl EvalSuite {
    /// Parse an EVAL.md file.
    pub fn parse(path: &Path) -> Result<Self, EvalParseError> {
        let content = std::fs::read_to_string(path).map_err(|e| EvalParseError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        Self::parse_content(path, &content)
    }

    /// Parse from string content.
    pub fn parse_content(path: &Path, content: &str) -> Result<Self, EvalParseError> {
        let (fm_raw, body) = split_frontmatter(content)?;
        let fm: EvalFrontmatter = serde_yaml::from_str(&fm_raw)?;
        let default_grader = parse_grader_kind(&fm.grader, path)?;

        let skill_path = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        let mut trigger_tests = Vec::new();
        let mut functional_tests = Vec::new();
        let mut perf_tests = Vec::new();

        for section in parse_sections(&body) {
            match section.prefix.as_str() {
                "Trigger" => {
                    trigger_tests.push(parse_trigger_test(&section)?);
                }
                "Test" => {
                    functional_tests.push(parse_functional_test(
                        &section,
                        &skill_path,
                        &default_grader,
                    )?);
                }
                "Perf" => {
                    perf_tests.push(parse_perf_test(&section)?);
                }
                other => {
                    return Err(EvalParseError::InvalidSection {
                        name: section.heading.clone(),
                        reason: format!(
                            "Unknown section prefix '{}'. Expected 'Trigger', 'Test', or 'Perf'",
                            other
                        ),
                    });
                }
            }
        }

        Ok(EvalSuite {
            path: path.to_path_buf(),
            skill_path,
            name: fm.name,
            description: fm.description,
            agent: fm.agent,
            model: fm.model,
            provider: fm.provider,
            thinking: fm.thinking,
            runs: fm.runs,
            timeout: fm.timeout,
            default_grader,
            trigger_tests,
            functional_tests,
            perf_tests,
        })
    }

    /// Total number of tests in this suite.
    pub fn total_tests(&self) -> usize {
        self.trigger_tests.len() + self.functional_tests.len() + self.perf_tests.len()
    }
}

// ── Internal helpers ──────────────────────────────────────────────

fn split_frontmatter(content: &str) -> Result<(String, String), EvalParseError> {
    let content = content.trim_start();

    if !content.starts_with("---") {
        return Err(EvalParseError::MissingFrontmatter);
    }

    let after_open = &content[3..];
    let close_pos = after_open
        .find("\n---")
        .ok_or(EvalParseError::UnclosedFrontmatter)?;

    let frontmatter = after_open[..close_pos].trim().to_string();
    let body_start = 3 + close_pos + 4;
    let body = if body_start < content.len() {
        content[body_start..].trim_start().to_string()
    } else {
        String::new()
    };

    Ok((frontmatter, body))
}

/// A raw section extracted from the markdown body.
#[derive(Debug)]
struct Section {
    /// Full heading text (e.g., "Trigger: obvious-task").
    heading: String,
    /// Prefix before the colon (e.g., "Trigger").
    prefix: String,
    /// Name after the colon (e.g., "obvious-task").
    name: String,
    /// Body content of the section.
    body: String,
}

fn parse_sections(body: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut current: Option<(String, String, String, Vec<String>)> = None;

    for line in body.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            // Flush previous section.
            if let Some((h, prefix, name, lines)) = current.take() {
                sections.push(Section {
                    heading: h,
                    prefix,
                    name,
                    body: lines.join("\n"),
                });
            }

            let heading = heading.trim().to_string();
            if let Some((prefix, name)) = heading.split_once(':') {
                current = Some((
                    heading.clone(),
                    prefix.trim().to_string(),
                    name.trim().to_string(),
                    Vec::new(),
                ));
            }
        } else if let Some((_, _, _, ref mut lines)) = current {
            lines.push(line.to_string());
        }
    }

    // Flush last section.
    if let Some((h, prefix, name, lines)) = current {
        sections.push(Section {
            heading: h,
            prefix,
            name,
            body: lines.join("\n"),
        });
    }

    sections
}

fn parse_grader_kind(s: &str, skill_path: &Path) -> Result<GraderKind, EvalParseError> {
    match s {
        "exact" => Ok(GraderKind::Exact),
        "contains" => Ok(GraderKind::Contains),
        "regex" => Ok(GraderKind::Regex),
        "llm" => Ok(GraderKind::Llm),
        other => {
            // Treat as script path relative to skill directory.
            let script_path = skill_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(other);
            Ok(GraderKind::Script(script_path))
        }
    }
}

fn parse_trigger_test(section: &Section) -> Result<TriggerTest, EvalParseError> {
    let body = section.body.trim();

    for line in body.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("- should_trigger:") {
            let prompt = unquote(rest.trim());
            return Ok(TriggerTest {
                name: section.name.clone(),
                prompt,
                should_trigger: true,
            });
        }
        if let Some(rest) = line.strip_prefix("- should_not_trigger:") {
            let prompt = unquote(rest.trim());
            return Ok(TriggerTest {
                name: section.name.clone(),
                prompt,
                should_trigger: false,
            });
        }
    }

    Err(EvalParseError::InvalidSection {
        name: section.heading.clone(),
        reason: "Trigger section must contain '- should_trigger:' or '- should_not_trigger:'"
            .into(),
    })
}

fn parse_functional_test(
    section: &Section,
    skill_path: &Path,
    default_grader: &GraderKind,
) -> Result<FunctionalTest, EvalParseError> {
    let body = &section.body;

    let prompt = extract_subsection_code_block(body, "### Prompt").unwrap_or_default();
    let inputs = extract_subsection_list(body, "### Input")
        .into_iter()
        .map(|p| skill_path.join(unquote_backtick(&p)))
        .collect();
    let expected = parse_expectations(body, default_grader);

    Ok(FunctionalTest {
        name: section.name.clone(),
        prompt,
        inputs,
        expected,
        grader: None,
    })
}

fn parse_perf_test(section: &Section) -> Result<PerfTest, EvalParseError> {
    let body = &section.body;
    let prompt = extract_subsection_code_block(body, "### Prompt").unwrap_or_default();
    let metrics = parse_perf_metrics(body);

    Ok(PerfTest {
        name: section.name.clone(),
        prompt,
        metrics,
    })
}

fn parse_expectations(body: &str, _default_grader: &GraderKind) -> Vec<Expectation> {
    let mut expectations = Vec::new();

    let expected = match extract_subsection_body(body, "### Expected") {
        Some(s) => s,
        None => return expectations,
    };

    for line in expected.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("- Contains:") {
            expectations.push(Expectation::Contains(unquote(rest.trim())));
        } else if let Some(rest) = line.strip_prefix("- Not contains:") {
            expectations.push(Expectation::NotContains(unquote(rest.trim())));
        } else if let Some(rest) = line.strip_prefix("- Regex:") {
            expectations.push(Expectation::Regex(unquote(rest.trim())));
        } else if let Some(rest) = line.strip_prefix("- Exit code:") {
            if let Ok(code) = rest.trim().parse::<i32>() {
                expectations.push(Expectation::ExitCode(code));
            }
        } else if let Some(rubric) = line.strip_prefix("- ") {
            // Treat as rubric line for LLM grading.
            expectations.push(Expectation::Rubric(rubric.to_string()));
        }
    }

    expectations
}

fn parse_perf_metrics(body: &str) -> Vec<PerfMetric> {
    let mut metrics = Vec::new();

    let expected = match extract_subsection_body(body, "### Expected") {
        Some(s) => s,
        None => return metrics,
    };

    for line in expected.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("- token_usage:") {
            if let Some(assertion) = parse_perf_assertion(rest.trim()) {
                metrics.push(PerfMetric::TokenUsage(assertion));
            }
        } else if let Some(rest) = line.strip_prefix("- message_count:") {
            if let Some(assertion) = parse_perf_assertion(rest.trim()) {
                metrics.push(PerfMetric::MessageCount(assertion));
            }
        } else if let Some(rest) = line.strip_prefix("- tool_call_count:") {
            if let Some(assertion) = parse_perf_assertion(rest.trim()) {
                metrics.push(PerfMetric::ToolCallCount(assertion));
            }
        } else if let Some(rest) = line.strip_prefix("- error_count:") {
            if let Some(assertion) = parse_perf_assertion(rest.trim()) {
                metrics.push(PerfMetric::ErrorCount(assertion));
            }
        }
    }

    metrics
}

fn parse_perf_assertion(s: &str) -> Option<PerfAssertion> {
    let s = s.trim();
    if s == "< baseline" {
        Some(PerfAssertion::LessThanBaseline)
    } else if let Ok(n) = s.parse::<u64>() {
        Some(PerfAssertion::Equals(n))
    } else {
        None
    }
}

fn extract_subsection_code_block(body: &str, heading: &str) -> Option<String> {
    let sub = extract_subsection_body(body, heading)?;
    let start = sub.find("```")? + 3;
    let after_lang = &sub[start..];
    let code_start = after_lang.find('\n')? + 1;
    let code_body = &after_lang[code_start..];
    let end = code_body.find("```")?;
    Some(code_body[..end].trim().to_string())
}

fn extract_subsection_body(body: &str, heading: &str) -> Option<String> {
    let start = body.find(heading)?;
    let after_heading = &body[start + heading.len()..];

    // Find the end: next ### heading or end of string.
    let end = after_heading
        .find("\n### ")
        .or_else(|| after_heading.find("\n## "))
        .unwrap_or(after_heading.len());

    Some(after_heading[..end].to_string())
}

fn extract_subsection_list(body: &str, heading: &str) -> Vec<String> {
    let sub = match extract_subsection_body(body, heading) {
        Some(s) => s,
        None => return Vec::new(),
    };

    sub.lines()
        .filter_map(|line| {
            let line = line.trim();
            line.strip_prefix("- ").map(|s| s.trim().to_string())
        })
        .collect()
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn unquote_backtick(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('`') && s.ends_with('`') {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_eval_frontmatter() {
        let content = r#"---
name: my-eval
description: Test eval
model: claude-sonnet-4-20250514
runs: 3
timeout: 120
---

## Trigger: obvious-task

- should_trigger: "Help me set up a workspace"
"#;
        let suite = EvalSuite::parse_content(Path::new("my-skill/EVAL.md"), content).unwrap();
        assert_eq!(suite.name, "my-eval");
        assert_eq!(suite.runs, 3);
        assert_eq!(suite.timeout, 120);
        assert_eq!(suite.trigger_tests.len(), 1);
        assert!(suite.trigger_tests[0].should_trigger);
        assert_eq!(suite.trigger_tests[0].prompt, "Help me set up a workspace");
    }

    #[test]
    fn test_parse_trigger_should_not() {
        let content = r#"---
name: test
description: test
---

## Trigger: unrelated

- should_not_trigger: "What's the weather?"
"#;
        let suite = EvalSuite::parse_content(Path::new("s/EVAL.md"), content).unwrap();
        assert_eq!(suite.trigger_tests.len(), 1);
        assert!(!suite.trigger_tests[0].should_trigger);
    }

    #[test]
    fn test_parse_functional_test() {
        let content = r#"---
name: test
description: test
---

## Test: basic-usage

### Prompt

```
Apply the skill.
```

### Expected

- Contains: "hello"
- Not contains: "error"
- Exit code: 0
"#;
        let suite = EvalSuite::parse_content(Path::new("s/EVAL.md"), content).unwrap();
        assert_eq!(suite.functional_tests.len(), 1);

        let ft = &suite.functional_tests[0];
        assert_eq!(ft.name, "basic-usage");
        assert_eq!(ft.prompt, "Apply the skill.");
        assert_eq!(ft.expected.len(), 3);
        assert!(matches!(&ft.expected[0], Expectation::Contains(s) if s == "hello"));
        assert!(matches!(&ft.expected[1], Expectation::NotContains(s) if s == "error"));
        assert!(matches!(&ft.expected[2], Expectation::ExitCode(0)));
    }

    #[test]
    fn test_parse_perf_test() {
        let content = r#"---
name: test
description: test
---

## Perf: baseline

### Prompt

```
Do something complex.
```

### Expected

- token_usage: < baseline
- error_count: 0
"#;
        let suite = EvalSuite::parse_content(Path::new("s/EVAL.md"), content).unwrap();
        assert_eq!(suite.perf_tests.len(), 1);
        assert_eq!(suite.perf_tests[0].metrics.len(), 2);
    }

    #[test]
    fn test_total_tests() {
        let content = r#"---
name: test
description: test
---

## Trigger: t1

- should_trigger: "hello"

## Test: f1

### Prompt

```
do it
```

### Expected

- Contains: "ok"

## Perf: p1

### Prompt

```
do it
```

### Expected

- error_count: 0
"#;
        let suite = EvalSuite::parse_content(Path::new("s/EVAL.md"), content).unwrap();
        assert_eq!(suite.total_tests(), 3);
    }

    #[test]
    fn test_missing_frontmatter() {
        let result = EvalSuite::parse_content(Path::new("s/EVAL.md"), "# No frontmatter");
        assert!(matches!(result, Err(EvalParseError::MissingFrontmatter)));
    }
}
