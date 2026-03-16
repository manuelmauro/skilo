//! `eval init` scaffolding — creates EVAL.md and evals/ directory.

use std::path::Path;

/// Scaffold eval files for a skill.
///
/// Returns a list of created paths.
pub fn scaffold_eval(skill_path: &Path) -> Result<Vec<String>, std::io::Error> {
    let mut created = Vec::new();

    let eval_path = skill_path.join("EVAL.md");
    if !eval_path.exists() {
        let skill_name = skill_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my-skill");

        let content = format!(
            r#"---
name: {skill_name}-eval
description: Evaluations for the {skill_name} skill.
runs: 1
timeout: 60
---

# {skill_name} Evaluations

## Trigger: obvious-task

- should_trigger: "Help me with {skill_name}"

## Test: basic-usage

Verify the skill produces correct output for a simple input.

### Prompt

```
Apply the {skill_name} instructions.
```

### Expected

- Exit code: 0
"#,
            skill_name = skill_name
        );

        std::fs::write(&eval_path, content)?;
        created.push(eval_path.display().to_string());
    }

    let fixtures_path = skill_path.join("evals").join("fixtures");
    if !fixtures_path.exists() {
        std::fs::create_dir_all(&fixtures_path)?;
        created.push(fixtures_path.display().to_string());
    }

    let graders_path = skill_path.join("evals").join("graders");
    if !graders_path.exists() {
        std::fs::create_dir_all(&graders_path)?;
        created.push(graders_path.display().to_string());
    }

    Ok(created)
}
