//! Creates a complete skill structure with scripts, references,
//! and assets directories, suitable for feature-rich skills.

use super::{to_title_case, SkillTemplate, TemplateContext};
use crate::cli::ScriptLang;
use std::fs;
use std::path::Path;

/// Template that creates a full skill with all directories and example files.
pub struct FullTemplate;

impl SkillTemplate for FullTemplate {
    fn render(&self, ctx: &TemplateContext, output_dir: &Path) -> std::io::Result<()> {
        let skill_dir = output_dir.join(&ctx.name);
        fs::create_dir_all(&skill_dir)?;

        // Write SKILL.md
        let skill_md = self.render_skill_md(ctx);
        fs::write(skill_dir.join("SKILL.md"), skill_md)?;

        // Create all optional directories
        let scripts_dir = skill_dir.join("scripts");
        let references_dir = skill_dir.join("references");
        let assets_dir = skill_dir.join("assets");

        fs::create_dir_all(&scripts_dir)?;
        fs::create_dir_all(&references_dir)?;
        fs::create_dir_all(&assets_dir)?;

        // Write example script
        let script_name = ctx.lang.file_name("main");
        let script_content = self.render_script(ctx);
        let script_path = scripts_dir.join(&script_name);
        fs::write(&script_path, script_content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms)?;
        }

        // Write reference document
        fs::write(
            references_dir.join("REFERENCE.md"),
            self.render_reference(ctx),
        )?;

        // Write placeholder asset
        fs::write(assets_dir.join(".gitkeep"), "")?;

        Ok(())
    }
}

impl FullTemplate {
    /// Render the SKILL.md content for a full skill.
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

{}

## Usage

See `references/REFERENCE.md` for detailed documentation.

## Scripts

- `scripts/main.{}` - Main entry point

## References

- `references/REFERENCE.md` - Detailed documentation

## Assets

Static assets are stored in the `assets/` directory.
"#,
            title,
            ctx.description,
            ctx.lang.extension()
        );

        frontmatter + &body
    }

    /// Render the main script content for the selected language.
    fn render_script(&self, ctx: &TemplateContext) -> String {
        match ctx.lang {
            ScriptLang::Python => format!(
                r#"{}
"""Main entry point for {}."""

import argparse
import sys


def main():
    parser = argparse.ArgumentParser(description="{}")
    parser.add_argument("--verbose", "-v", action="store_true", help="Enable verbose output")
    args = parser.parse_args()

    if args.verbose:
        print("Verbose mode enabled")

    print("Hello from {}!")


if __name__ == "__main__":
    main()
"#,
                ctx.lang.shebang(),
                ctx.name,
                ctx.description,
                ctx.name
            ),

            ScriptLang::Bash => format!(
                r#"{}
# Main entry point for {}.

set -euo pipefail

VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

if [ "$VERBOSE" = true ]; then
    echo "Verbose mode enabled"
fi

echo "Hello from {}!"
"#,
                ctx.lang.shebang(),
                ctx.name,
                ctx.name
            ),

            ScriptLang::Javascript => format!(
                r#"{}
// Main entry point for {}.

const args = process.argv.slice(2);
const verbose = args.includes("-v") || args.includes("--verbose");

if (verbose) {{
    console.log("Verbose mode enabled");
}}

console.log("Hello from {}!");
"#,
                ctx.lang.shebang(),
                ctx.name,
                ctx.name
            ),

            ScriptLang::Typescript => format!(
                r#"{}
// Main entry point for {}.

const args: string[] = process.argv.slice(2);
const verbose: boolean = args.includes("-v") || args.includes("--verbose");

if (verbose) {{
    console.log("Verbose mode enabled");
}}

console.log("Hello from {}!");
"#,
                ctx.lang.shebang(),
                ctx.name,
                ctx.name
            ),
        }
    }

    /// Render the reference documentation content.
    fn render_reference(&self, ctx: &TemplateContext) -> String {
        let title = to_title_case(&ctx.name);
        format!(
            r#"# {} Reference

## Overview

{}

## Configuration

This skill does not require any configuration.

## API

### Scripts

#### `scripts/main.{}`

Main entry point for the skill.

**Arguments:**

- `--verbose`, `-v`: Enable verbose output

**Exit codes:**

- `0`: Success
- `1`: Error

## Examples

```bash
./scripts/main.{}
./scripts/main.{} --verbose
```
"#,
            title,
            ctx.description,
            ctx.lang.extension(),
            ctx.lang.extension(),
            ctx.lang.extension()
        )
    }
}
