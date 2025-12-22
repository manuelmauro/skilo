use super::{to_title_case, SkillTemplate, TemplateContext};
use crate::cli::ScriptLang;
use std::fs;
use std::path::Path;

pub struct ScriptBasedTemplate;

impl SkillTemplate for ScriptBasedTemplate {
    fn render(&self, ctx: &TemplateContext, output_dir: &Path) -> std::io::Result<()> {
        let skill_dir = output_dir.join(&ctx.name);
        fs::create_dir_all(&skill_dir)?;

        // Write SKILL.md
        let skill_md = self.render_skill_md(ctx);
        fs::write(skill_dir.join("SKILL.md"), skill_md)?;

        // Create scripts directory with multiple example scripts
        let scripts_dir = skill_dir.join("scripts");
        fs::create_dir_all(&scripts_dir)?;

        // Write multiple scripts
        for (name, content) in self.render_scripts(ctx) {
            let script_path = scripts_dir.join(name);
            fs::write(&script_path, content)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&script_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&script_path, perms)?;
            }
        }

        Ok(())
    }
}

impl ScriptBasedTemplate {
    fn render_skill_md(&self, ctx: &TemplateContext) -> String {
        let mut frontmatter = format!(
            "---\nname: {}\ndescription: {}\n",
            ctx.name,
            ctx.description.replace('\n', " ")
        );

        if let Some(license) = &ctx.license {
            frontmatter.push_str(&format!("license: {}\n", license));
        }

        frontmatter.push_str("---\n\n");

        let title = to_title_case(&ctx.name);
        let ext = ctx.lang.extension();

        let body = format!(
            r#"# {}

{}

## Scripts

This skill provides the following scripts:

- `scripts/setup.{}` - Initialize and configure
- `scripts/run.{}` - Execute the main functionality
- `scripts/cleanup.{}` - Clean up resources

## Usage

1. Run setup first:
   ```bash
   ./scripts/setup.{}
   ```

2. Execute the main script:
   ```bash
   ./scripts/run.{} [args]
   ```

3. Clean up when done:
   ```bash
   ./scripts/cleanup.{}
   ```
"#,
            title, ctx.description, ext, ext, ext, ext, ext, ext
        );

        frontmatter + &body
    }

    fn render_scripts(&self, ctx: &TemplateContext) -> Vec<(String, String)> {
        let ext = ctx.lang.extension();
        let shebang = ctx.lang.shebang();

        match ctx.lang {
            ScriptLang::Python => vec![
                (
                    format!("setup.{}", ext),
                    format!(
                        r#"{}
"""Setup script for {}."""

import os
import sys


def main():
    print("Setting up {}...")
    # Add setup logic here
    print("Setup complete!")


if __name__ == "__main__":
    main()
"#,
                        shebang, ctx.name, ctx.name
                    ),
                ),
                (
                    format!("run.{}", ext),
                    format!(
                        r#"{}
"""Main execution script for {}."""

import sys


def main():
    args = sys.argv[1:]
    print(f"Running {} with args: {{args}}")
    # Add main logic here


if __name__ == "__main__":
    main()
"#,
                        shebang, ctx.name, ctx.name
                    ),
                ),
                (
                    format!("cleanup.{}", ext),
                    format!(
                        r#"{}
"""Cleanup script for {}."""


def main():
    print("Cleaning up {}...")
    # Add cleanup logic here
    print("Cleanup complete!")


if __name__ == "__main__":
    main()
"#,
                        shebang, ctx.name, ctx.name
                    ),
                ),
            ],

            ScriptLang::Bash => vec![
                (
                    format!("setup.{}", ext),
                    format!(
                        r#"{}
# Setup script for {}.

set -euo pipefail

echo "Setting up {}..."
# Add setup logic here
echo "Setup complete!"
"#,
                        shebang, ctx.name, ctx.name
                    ),
                ),
                (
                    format!("run.{}", ext),
                    format!(
                        r#"{}
# Main execution script for {}.

set -euo pipefail

echo "Running {} with args: $@"
# Add main logic here
"#,
                        shebang, ctx.name, ctx.name
                    ),
                ),
                (
                    format!("cleanup.{}", ext),
                    format!(
                        r#"{}
# Cleanup script for {}.

set -euo pipefail

echo "Cleaning up {}..."
# Add cleanup logic here
echo "Cleanup complete!"
"#,
                        shebang, ctx.name, ctx.name
                    ),
                ),
            ],

            ScriptLang::Javascript => vec![
                (
                    format!("setup.{}", ext),
                    format!(
                        r#"{}
// Setup script for {}.

console.log("Setting up {}...");
// Add setup logic here
console.log("Setup complete!");
"#,
                        shebang, ctx.name, ctx.name
                    ),
                ),
                (
                    format!("run.{}", ext),
                    format!(
                        r#"{}
// Main execution script for {}.

const args = process.argv.slice(2);
console.log(`Running {} with args: ${{args.join(" ")}}`);
// Add main logic here
"#,
                        shebang, ctx.name, ctx.name
                    ),
                ),
                (
                    format!("cleanup.{}", ext),
                    format!(
                        r#"{}
// Cleanup script for {}.

console.log("Cleaning up {}...");
// Add cleanup logic here
console.log("Cleanup complete!");
"#,
                        shebang, ctx.name, ctx.name
                    ),
                ),
            ],

            ScriptLang::Typescript => vec![
                (
                    format!("setup.{}", ext),
                    format!(
                        r#"{}
// Setup script for {}.

console.log("Setting up {}...");
// Add setup logic here
console.log("Setup complete!");
"#,
                        shebang, ctx.name, ctx.name
                    ),
                ),
                (
                    format!("run.{}", ext),
                    format!(
                        r#"{}
// Main execution script for {}.

const args: string[] = process.argv.slice(2);
console.log(`Running {} with args: ${{args.join(" ")}}`);
// Add main logic here
"#,
                        shebang, ctx.name, ctx.name
                    ),
                ),
                (
                    format!("cleanup.{}", ext),
                    format!(
                        r#"{}
// Cleanup script for {}.

console.log("Cleaning up {}...");
// Add cleanup logic here
console.log("Cleanup complete!");
"#,
                        shebang, ctx.name, ctx.name
                    ),
                ),
            ],
        }
    }
}
