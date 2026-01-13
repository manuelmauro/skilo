//! Creates a skill with a simple greeting script, suitable for
//! getting started with Agent Skills development.

use super::{to_title_case, SkillTemplate, TemplateContext};
use crate::cli::ScriptLang;
use std::fs;
use std::path::Path;

/// Template that creates a hello world skill with a greeting script.
pub struct HelloWorldTemplate;

impl SkillTemplate for HelloWorldTemplate {
    fn render(&self, ctx: &TemplateContext, output_dir: &Path) -> std::io::Result<()> {
        let skill_dir = output_dir.join(&ctx.name);
        fs::create_dir_all(&skill_dir)?;

        // Write SKILL.md
        let skill_md = self.render_skill_md(ctx);
        fs::write(skill_dir.join("SKILL.md"), skill_md)?;

        // Write script
        if ctx.include_scripts {
            let scripts_dir = skill_dir.join("scripts");
            fs::create_dir_all(&scripts_dir)?;

            let script_name = ctx.lang.file_name("greet");
            let script_content = self.render_script(ctx);
            let script_path = scripts_dir.join(&script_name);

            fs::write(&script_path, script_content)?;

            // Make executable on Unix
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

impl HelloWorldTemplate {
    /// Render the SKILL.md content for a hello world skill.
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

        let body = format!(
            r#"# {}

This skill provides a simple greeting functionality.

## Usage

Run the greeting script to display a personalized message.

## Scripts

- `scripts/greet.{}` - Outputs a greeting message

## Example

```bash
./scripts/greet.{} World
# Output: Hello, World!
```
"#,
            title,
            ctx.lang.extension(),
            ctx.lang.extension()
        );

        frontmatter + &body
    }

    /// Render the greeting script content for the selected language.
    fn render_script(&self, ctx: &TemplateContext) -> String {
        match ctx.lang {
            ScriptLang::Python => format!(
                r#"{}
"""A simple greeting script."""

import sys


def main():
    name = sys.argv[1] if len(sys.argv) > 1 else "World"
    print(f"Hello, {{name}}!")


if __name__ == "__main__":
    main()
"#,
                ctx.lang.shebang()
            ),

            ScriptLang::Bash => format!(
                r#"{}
# A simple greeting script.

set -euo pipefail

name="${{1:-World}}"
echo "Hello, ${{name}}!"
"#,
                ctx.lang.shebang()
            ),

            ScriptLang::Javascript => format!(
                r#"{}
// A simple greeting script.

const name = process.argv[2] || "World";
console.log(`Hello, ${{name}}!`);
"#,
                ctx.lang.shebang()
            ),

            ScriptLang::Typescript => format!(
                r#"{}
// A simple greeting script.

const name: string = process.argv[2] || "World";
console.log(`Hello, ${{name}}!`);
"#,
                ctx.lang.shebang()
            ),
        }
    }
}
