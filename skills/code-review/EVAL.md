---
name: code-review-eval
description: Evaluations for the code-review skill.
runs: 1
timeout: 180
---

# code-review Evaluations

## Trigger: review-request

- should_trigger: "Review my code changes for quality and correctness"

## Trigger: unrelated-question

- should_not_trigger: "What is the capital of France?"

## Test: checklist-awareness

Verify the skill is aware of the code review checklist.

### Prompt

```text
List the code review checklist items from the code-review skill. Just list them, don't run any commands.
```

### Expected

- Contains: "cargo test"
- Contains: "clippy"
- Exit code: 0
