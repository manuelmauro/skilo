# ADR 0008: Eval Subcommand

## Status

Proposed

## Context

Evals, short for evaluations, are systematic methods used to measure the performance and effectiveness of AI systems, particularly large language models (LLMs). They help ensure that these models meet predefined criteria and function reliably in real-world applications.

Agent Skills are instructions and tooling that shape how an AI coding agent behaves. However, there is currently no built-in mechanism to verify that a skill actually produces the desired outcomes when consumed by an agent. Skill authors must resort to manual, ad-hoc testing — running the skill against an agent and visually inspecting the results — which is:

1. **Time-consuming**: Each change requires a full manual review cycle
2. **Non-reproducible**: Results depend on the tester, the model version, and ambient context
3. **Not automatable**: Cannot be integrated into CI/CD pipelines
4. **Subjective**: No quantifiable pass/fail criteria

Anthropic's *Complete Guide to Building Skills for Claude* identifies three levels of testing rigor for skills:

- **Manual testing in Claude.ai** — Run queries directly and observe behavior. Fast iteration, no setup required.
- **Scripted testing in Claude Code** — Automate test cases for repeatable validation across changes.
- **Programmatic testing via the skills API** — Build evaluation suites that run systematically against defined test sets.

The guide further recommends that effective skill testing covers three areas:

1. **Triggering tests** — Ensure the skill loads at the right times (triggers on obvious tasks and paraphrased requests, does *not* trigger on unrelated topics).
2. **Functional tests** — Verify the skill produces correct outputs (valid outputs, successful API calls, error handling, edge cases).
3. **Performance comparison** — Prove the skill improves results versus a no-skill baseline (fewer messages, lower token consumption, fewer API failures).

These categories, along with quantitative success criteria (e.g., "skill triggers on 90% of relevant queries", "0 failed API calls per workflow") and qualitative metrics (e.g., "users don't need to prompt Claude about next steps", "consistent results across sessions"), provide a strong foundation for a structured eval framework.

Today, `skilo` has no way to encode, run, or report on any of these test categories. Other ecosystems have standardized evaluation frameworks (e.g., OpenAI Evals, LangSmith, Inspect AI), but none are tailored for the Agent Skills specification or integrate with the `skilo` workflow.

## Decision

We will add a `skilo eval` subcommand that allows skill authors to define, run, and report on evaluations for their skills. The design directly encodes the three test categories recommended by Anthropic's guide — triggering, functional, and performance — as first-class concepts, and supports the "iterate on a single task first, then expand" workflow.

### Test Categories

Following the official Anthropic guidance, every eval suite is organized around three test categories. Authors can use any combination of them.

#### 1. Triggering Tests

Verify that the skill's `description` frontmatter causes the model to activate the skill at the right times.

| Assertion        | Description                                            |
| ---------------- | ------------------------------------------------------ |
| `should_trigger` | Given this prompt, the skill **should** be activated   |
| `should_not_trigger` | Given this prompt, the skill should **not** activate |

Example in `EVAL.md`:

```markdown
## Trigger: obvious-task
- should_trigger: "Help me set up a new ProjectHub workspace"

## Trigger: paraphrased
- should_trigger: "I need to create a project in ProjectHub"

## Trigger: unrelated
- should_not_trigger: "What's the weather in San Francisco?"
```

#### 2. Functional Tests

Verify that the skill produces correct, complete outputs. These are the primary test type and map to the `## Test:` sections described below.

#### 3. Performance Comparison Tests

Compare with-skill vs. without-skill execution on the same prompt, measuring quantitative metrics.

| Metric              | Description                                     |
| ------------------- | ----------------------------------------------- |
| `token_usage`       | Total tokens consumed                           |
| `message_count`     | Number of back-and-forth messages               |
| `tool_call_count`   | Number of tool/MCP calls                        |
| `error_count`       | Number of failed API/tool calls                 |

Example in `EVAL.md`:

```markdown
## Perf: baseline-comparison

### Prompt

```
Set up a new project workspace with 5 tasks
```

### Baseline

Run the same prompt **without** the skill enabled.

### Expected

- token_usage: < baseline
- message_count: < baseline
- error_count: 0
```

### Eval Specification

Evals are defined in an `EVAL.md` file placed inside the skill directory alongside `SKILL.md`:

```
my-skill/
├── SKILL.md
├── EVAL.md            # Evaluation definitions
├── evals/             # Eval-specific assets
│   ├── fixtures/      # Input fixtures (files, prompts, context)
│   └── graders/       # Custom grading scripts
├── scripts/
├── references/
└── assets/
```

### EVAL.md Format

```markdown
---
name: my-skill-eval
description: Evaluations for the my-skill skill.
model: claude-sonnet-4-20250514
runs: 3
timeout: 120
---

# My Skill Evaluations

<!-- ── Triggering tests ─────────────────────────────────── -->

## Trigger: obvious-task

- should_trigger: "Help me set up a new workspace"

## Trigger: paraphrased-request

- should_trigger: "I need to initialize a project"

## Trigger: unrelated-topic

- should_not_trigger: "What's the weather today?"

<!-- ── Functional tests ─────────────────────────────────── -->

## Test: basic-usage

Verify the skill produces correct output for a simple input.

### Prompt

```
Given the following code, apply the my-skill instructions.
```

### Input

- `evals/fixtures/basic-input.txt`

### Expected

- Contains: "expected output substring"
- Not contains: "error"
- Exit code: 0

## Test: edge-case-empty-input

Verify the skill handles empty input gracefully.

### Prompt

```
Apply the my-skill instructions to an empty file.
```

### Input

- `evals/fixtures/empty.txt`

### Expected

- Contains: "no input provided"
- Exit code: 0

<!-- ── Performance comparison tests ─────────────────────── -->

## Perf: baseline-comparison

### Prompt

```
Set up a complete project workspace with 5 tasks.
```

### Baseline

Run the same prompt without the skill enabled.

### Expected

- token_usage: < baseline
- message_count: < baseline
- error_count: 0
```

### EVAL.md Frontmatter

| Field         | Type     | Required | Description                                       |
| ------------- | -------- | -------- | ------------------------------------------------- |
| `name`        | string   | yes      | Evaluation suite name (1-64 chars)                |
| `description` | string   | yes      | What is being evaluated (1-1024 chars)            |
| `model`       | string   | no       | Default model to evaluate against                 |
| `runs`        | integer  | no       | Number of times to run each test (default: 1)     |
| `timeout`     | integer  | no       | Per-test timeout in seconds (default: 60)         |
| `grader`      | string   | no       | Default grader: `exact`, `contains`, `llm`, `script` (default: `contains`) |

### Section Prefixes

Each `## ` heading in the EVAL.md body uses a prefix to declare its test category:

| Prefix       | Category               | Description                                    |
| ------------ | ---------------------- | ---------------------------------------------- |
| `Trigger:`   | Triggering test        | Asserts skill activation / non-activation      |
| `Test:`      | Functional test        | Asserts output correctness                     |
| `Perf:`      | Performance comparison | Compares with-skill vs. without-skill metrics  |

### Success Criteria

Anthropic's guide recommends both quantitative and qualitative success criteria. `skilo eval` encodes the quantitative ones as assertions and surfaces the qualitative ones in the report for human review.

**Quantitative (automated)**:

| Criterion                             | Assertion example                              |
| ------------------------------------- | ---------------------------------------------- |
| Skill triggers on ≥ 90% of queries   | `Trigger:` tests pass rate ≥ threshold         |
| Completes workflow in ≤ N tool calls  | `tool_call_count: <= 5`                        |
| 0 failed API calls per workflow       | `error_count: 0`                               |

**Qualitative (reported for human review)**:

| Criterion                                          | How it surfaces                                       |
| -------------------------------------------------- | ----------------------------------------------------- |
| Users don't need to prompt about next steps        | `--verbose` output shows full conversation transcript  |
| Workflow completes without user correction          | `message_count` metric in `Perf:` tests               |
| Consistent results across sessions                 | Multi-run variance shown in report                     |

### Grading Strategies

| Grader     | Description                                                  |
| ---------- | ------------------------------------------------------------ |
| `exact`    | Output must match expected value exactly                     |
| `contains` | Output must contain all expected substrings                  |
| `regex`    | Output must match provided regex patterns                    |
| `llm`      | Use an LLM to judge output against rubric                    |
| `script`   | Run a custom script in `evals/graders/` that returns pass/fail |

#### Script Grader

Custom grading scripts receive the test output on stdin and must exit with code `0` for pass, non-zero for fail:

```python
#!/usr/bin/env python3
"""Custom grader for JSON output validation."""

import json
import sys

output = sys.stdin.read()
try:
    data = json.loads(output)
    assert "result" in data
    assert data["result"] is not None
    sys.exit(0)
except (json.JSONDecodeError, AssertionError) as e:
    print(f"FAIL: {e}", file=sys.stderr)
    sys.exit(1)
```

#### LLM Grader

When using the `llm` grader, an `### Expected` section serves as a rubric for the judging model:

```markdown
### Expected

- The output correctly summarizes the input document
- Key facts are preserved without hallucination
- The tone is professional and concise
```

### Subcommand Details

#### `skilo eval [path]`

Run evaluations for skill(s) at the given path.

```bash
# Run evals for a single skill
skilo eval my-skill/

# Run evals for all skills in current directory
skilo eval .

# Run a specific test
skilo eval my-skill/ --test basic-usage

# Run with a specific model
skilo eval my-skill/ --model claude-sonnet-4-20250514

# Run multiple times for statistical confidence
skilo eval my-skill/ --runs 5

# Output results as JSON
skilo eval my-skill/ --format json
```

Options:

| Flag                  | Description                                        | Default     |
| --------------------- | -------------------------------------------------- | ----------- |
| `--test <name>`       | Run only the specified test by name                | all tests   |
| `--category <cat>`    | Run only tests of this category: `trigger`, `functional`, `perf` | all |
| `--model <model>`     | Override the model used for evaluation             | from EVAL.md |
| `--runs <n>`          | Override number of runs per test                   | from EVAL.md |
| `--timeout <secs>`    | Override per-test timeout                          | from EVAL.md |
| `--format <fmt>`      | Output format: `text`, `json`, `markdown`          | `text`      |
| `--fail-fast`         | Stop on first failure                              | `false`     |
| `--grader <type>`     | Override grader for all functional tests           | from EVAL.md |
| `--verbose`           | Show full model input/output                       | `false`     |
| `--dry-run`           | Parse and validate evals without running them      | `false`     |

#### `skilo eval init [path]`

Scaffold an `EVAL.md` and `evals/` directory for an existing skill:

```bash
$ skilo eval init my-skill/
Created my-skill/EVAL.md
Created my-skill/evals/fixtures/
Created my-skill/evals/graders/
```

### Output Format

#### Text (default)

```
Evaluating: my-skill (6 tests, 1 run each)

Triggering:
  ✓ obvious-task ............. triggered (0.4s)
  ✓ paraphrased-request ...... triggered (0.5s)
  ✓ unrelated-topic .......... not triggered (0.3s)

Functional:
  ✓ basic-usage .............. passed (1.2s)
  ✓ edge-case-empty-input .... passed (0.8s)
  ✗ complex-scenario ......... FAILED (2.1s)
    Expected contains: "summary complete"
    Got: "I was unable to process the input..."

Performance:
  ✓ baseline-comparison ...... improved (3.4s)
    token_usage:   6120 vs 12040 baseline (↓ 49%)
    message_count: 2 vs 15 baseline (↓ 87%)
    error_count:   0 (target: 0)

Results: 5 passed, 1 failed, 6 total (8.7s)
```

#### JSON

```json
{
  "skill": "my-skill",
  "model": "claude-sonnet-4-20250514",
  "timestamp": "2026-03-11T15:09:55Z",
  "categories": {
    "trigger": {
      "total": 3,
      "passed": 3,
      "failed": 0
    },
    "functional": {
      "total": 2,
      "passed": 1,
      "failed": 1
    },
    "perf": {
      "total": 1,
      "passed": 1,
      "failed": 0
    }
  },
  "tests": [
    {
      "name": "basic-usage",
      "category": "functional",
      "status": "passed",
      "runs": [
        {
          "run": 1,
          "status": "passed",
          "duration_ms": 1200,
          "output": "..."
        }
      ]
    },
    {
      "name": "baseline-comparison",
      "category": "perf",
      "status": "passed",
      "metrics": {
        "token_usage": { "skill": 6120, "baseline": 12040 },
        "message_count": { "skill": 2, "baseline": 15 },
        "error_count": { "skill": 0, "baseline": 3 }
      }
    }
  ],
  "summary": {
    "total": 6,
    "passed": 5,
    "failed": 1,
    "duration_ms": 8700
  }
}
```

#### Markdown

Produces a summary table suitable for inclusion in PRs or documentation:

```markdown
## Eval Results: my-skill

### Triggering (3/3 passed)

| Test                  | Assertion          | Status | Duration |
| --------------------- | ------------------ | ------ | -------- |
| obvious-task          | should_trigger     | ✓ Pass | 0.4s     |
| paraphrased-request   | should_trigger     | ✓ Pass | 0.5s     |
| unrelated-topic       | should_not_trigger | ✓ Pass | 0.3s     |

### Functional (1/2 passed)

| Test                  | Status | Duration | Runs |
| --------------------- | ------ | -------- | ---- |
| basic-usage           | ✓ Pass | 1.2s     | 1/1  |
| complex-scenario      | ✗ Fail | 2.1s     | 0/1  |

### Performance (1/1 passed)

| Test                  | Metric        | Skill | Baseline | Delta |
| --------------------- | ------------- | ----- | -------- | ----- |
| baseline-comparison   | token_usage   | 6120  | 12040    | ↓ 49% |
|                       | message_count | 2     | 15       | ↓ 87% |
|                       | error_count   | 0     | 3        | ↓100% |
```

### Iteration Workflow

Anthropic recommends: *"Iterate on a single challenging task until Claude succeeds, then extract the winning approach into a skill."* The eval subcommand supports this workflow:

1. **Start small**: Write a single `Test:` for the hardest task your skill should handle.
2. **Run repeatedly**: Use `--runs 3-5` to surface non-deterministic failures.
3. **Expand coverage**: Once the core test passes reliably, add triggering tests and edge cases.
4. **Add performance baselines**: When functional tests are green, add `Perf:` tests to quantify improvement.
5. **Monitor regressions**: Run `skilo eval` in CI to catch breakages from skill edits or model updates.

The eval report also surfaces **iteration signals** from the Anthropic guide:

| Signal              | Detected by              | Suggested action                                     |
| ------------------- | ------------------------ | ---------------------------------------------------- |
| Under-triggering    | `Trigger:` tests failing | Add keywords / trigger phrases to `description`      |
| Over-triggering     | `should_not_trigger` failing | Add negative triggers, be more specific           |
| Execution issues    | `Test:` failures         | Improve instructions, add error handling in skill    |
| Performance parity  | `Perf:` no improvement   | Refactor skill instructions, reduce steps            |

### Configuration

In `.skilorc.toml`:

```toml
[eval]
# Default model for evaluations
default_model = "claude-sonnet-4-20250514"

# Default number of runs per test
default_runs = 1

# Default timeout in seconds
default_timeout = 60

# Default output format
default_format = "text"

# Fail the command if any test fails
fail_on_error = true
```

### Integration with `skilo check`

The `skilo check` command will gain an optional `--eval` flag:

```bash
# Run lint + format check + evals
skilo check --eval .
```

This allows evals to be included in CI pipelines alongside existing validation:

```yaml
- name: Validate and evaluate skills
  run: |
    skilo check --eval --strict .
```

### Exit Codes

| Code | Meaning                              |
| ---- | ------------------------------------ |
| 0    | All evaluations passed               |
| 1    | One or more evaluations failed       |
| 2    | Invalid arguments or configuration   |
| 3    | I/O error (file not found, timeout)  |
| 4    | Model/API error (unreachable, auth)  |

## Consequences

### Positive

- Skill authors can verify effectiveness before publishing
- Reproducible, automated quality assurance for skills
- CI/CD integration enables regression testing across model updates
- Multiple grading strategies accommodate diverse skill types
- Familiar testing UX (inspired by `cargo test`, `pytest`)
- Statistical confidence through repeated runs

### Negative

- Requires API access/credentials for model-based evaluation
- Eval runs incur cost and latency (LLM API calls)
- Non-deterministic model outputs make exact matching fragile
- Additional file (`EVAL.md`) and directory (`evals/`) to maintain
- LLM-as-judge grading introduces its own reliability questions

### Neutral

- Evals are optional — skills work without them
- Does not prescribe a specific LLM provider; model string is opaque to skilo
- Eval results are ephemeral by default (no persistent history)
- Complements but does not replace manual skill review

## Implementation Notes

### Module Structure

```
src/
├── commands/
│   ├── eval.rs          # Eval subcommand entry point
│   └── ...
├── eval/
│   ├── mod.rs           # Public API
│   ├── parser.rs        # EVAL.md parsing (frontmatter + test cases)
│   ├── runner.rs        # Test execution orchestration
│   ├── categories/
│   │   ├── mod.rs
│   │   ├── trigger.rs   # Triggering test runner
│   │   ├── functional.rs # Functional test runner
│   │   └── perf.rs      # Performance comparison runner
│   ├── grader.rs        # Grading strategy dispatch
│   ├── graders/
│   │   ├── mod.rs
│   │   ├── exact.rs     # Exact match grader
│   │   ├── contains.rs  # Substring match grader
│   │   ├── regex.rs     # Regex pattern grader
│   │   ├── llm.rs       # LLM-as-judge grader
│   │   └── script.rs    # External script grader
│   ├── report.rs        # Output formatting (text, json, markdown)
│   └── scaffold.rs      # `eval init` scaffolding
└── ...
```

### Key Types

```rust
/// Parsed evaluation suite from EVAL.md.
pub struct EvalSuite {
    pub name: String,
    pub description: String,
    pub model: Option<String>,
    pub runs: u32,
    pub timeout: u64,
    pub default_grader: GraderKind,
    pub trigger_tests: Vec<TriggerTest>,
    pub functional_tests: Vec<FunctionalTest>,
    pub perf_tests: Vec<PerfTest>,
}

/// Test category tag.
pub enum TestCategory {
    Trigger,
    Functional,
    Perf,
}

// ── Triggering tests ──────────────────────────────────

/// A triggering test case.
pub struct TriggerTest {
    pub name: String,
    pub prompt: String,
    pub should_trigger: bool,
}

// ── Functional tests ──────────────────────────────────

/// A functional test case.
pub struct FunctionalTest {
    pub name: String,
    pub prompt: String,
    pub inputs: Vec<PathBuf>,
    pub expected: Vec<Expectation>,
    pub grader: Option<GraderKind>,
}

/// Grading strategy.
pub enum GraderKind {
    Exact,
    Contains,
    Regex,
    Llm,
    Script(PathBuf),
}

/// A single expectation within a test.
pub enum Expectation {
    Contains(String),
    NotContains(String),
    Regex(String),
    ExitCode(i32),
    Rubric(String),
}

// ── Performance comparison tests ──────────────────────

/// A performance comparison test case.
pub struct PerfTest {
    pub name: String,
    pub prompt: String,
    pub metrics: Vec<PerfMetric>,
}

/// A performance metric and its assertion.
pub enum PerfMetric {
    TokenUsage(PerfAssertion),
    MessageCount(PerfAssertion),
    ToolCallCount(PerfAssertion),
    ErrorCount(PerfAssertion),
}

/// How a performance metric should compare to the baseline.
pub enum PerfAssertion {
    LessThanBaseline,
    Equals(u64),
}

// ── Results ───────────────────────────────────────────

/// Result of a single test run.
pub struct TestRunResult {
    pub run: u32,
    pub status: TestStatus,
    pub duration: Duration,
    pub output: String,
    pub error: Option<String>,
}

/// Aggregated result of a test across all runs.
pub struct TestResult {
    pub name: String,
    pub category: TestCategory,
    pub runs: Vec<TestRunResult>,
    pub status: TestStatus,
}

pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    TimedOut,
}
```

### Execution Flow

```
1. Discover skills at given path
2. For each skill with an EVAL.md:
   a. Parse EVAL.md frontmatter and test definitions
   b. Validate test structure (inputs exist, graders available)
   c. For each test:
      i.   Prepare prompt (skill instructions + test prompt + input fixtures)
      ii.  Send to model (repeat for `runs` count)
      iii. Grade each response against expectations
      iv.  Collect results
   d. Aggregate and report results
3. Exit with appropriate code
```

## Future Extensions

- **Eval history**: Persist results to track performance over time and across model versions
- **Baseline comparisons**: Compare current results against a saved baseline
- **Parallel execution**: Run independent tests concurrently
- **Eval sharing**: Publish eval suites alongside skills for community benchmarking
- **Provider abstraction**: Pluggable model backends (OpenAI, Anthropic, local models)
- **Coverage metrics**: Measure which skill instructions are exercised by the eval suite

## References

- [The Complete Guide to Building Skills for Claude](https://resources.anthropic.com/hubfs/The-Complete-Guide-to-Building-Skill-for-Claude.pdf) — Primary source for the three test categories and success criteria
- [OpenAI Evals](https://github.com/openai/evals)
- [Anthropic Evaluations](https://docs.anthropic.com/en/docs/test-and-evaluate/strengthen-guardrails/reduce-hallucinations)
- [Inspect AI](https://inspect.ai-safety-institute.org.uk/)
- [LangSmith Evaluation](https://docs.smith.langchain.com/evaluation)
- [Agent Skills Specification](https://agentskills.io/specification)
- [Braintrust Evals](https://www.braintrust.dev/docs/guides/evals)
