mod full;
mod hello_world;
mod minimal;
mod script_based;

use crate::cli::{ScriptLang, Template};
use std::path::Path;

pub use full::FullTemplate;
pub use hello_world::HelloWorldTemplate;
pub use minimal::MinimalTemplate;
pub use script_based::ScriptBasedTemplate;

pub struct TemplateContext {
    pub name: String,
    pub description: String,
    pub license: Option<String>,
    pub lang: ScriptLang,
    pub include_optional_dirs: bool,
    pub include_scripts: bool,
}

pub trait SkillTemplate {
    fn render(&self, ctx: &TemplateContext, output_dir: &Path) -> std::io::Result<()>;
}

pub fn get_template(template: Template) -> Box<dyn SkillTemplate> {
    match template {
        Template::HelloWorld => Box::new(HelloWorldTemplate),
        Template::Minimal => Box::new(MinimalTemplate),
        Template::Full => Box::new(FullTemplate),
        Template::ScriptBased => Box::new(ScriptBasedTemplate),
    }
}

/// Convert a kebab-case name to Title Case
pub fn to_title_case(name: &str) -> String {
    name.split('-')
        .map(|s| {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().chain(c).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
