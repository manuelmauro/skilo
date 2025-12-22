use super::{to_title_case, SkillTemplate, TemplateContext};
use std::fs;
use std::path::Path;

pub struct MinimalTemplate;

impl SkillTemplate for MinimalTemplate {
    fn render(&self, ctx: &TemplateContext, output_dir: &Path) -> std::io::Result<()> {
        let skill_dir = output_dir.join(&ctx.name);
        fs::create_dir_all(&skill_dir)?;

        // Write SKILL.md only
        let skill_md = self.render_skill_md(ctx);
        fs::write(skill_dir.join("SKILL.md"), skill_md)?;

        Ok(())
    }
}

impl MinimalTemplate {
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
"#,
            title, ctx.description
        );

        frontmatter + &body
    }
}
