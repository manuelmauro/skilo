//! Skill templates for scaffolding new skills.
//!
//! This module provides different templates for creating new Agent Skills,
//! ranging from minimal single-file skills to full-featured skills with
//! scripts, references, and assets.

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

/// Context for rendering skill templates.
///
/// Contains all the information needed to generate a new skill from a template.
pub struct TemplateContext {
    /// The skill name (kebab-case identifier).
    pub name: String,
    /// A brief description of the skill.
    pub description: String,
    /// Optional license identifier.
    pub license: Option<String>,
    /// The scripting language for generated scripts.
    pub lang: ScriptLang,
    /// Whether to include optional directories (references, assets).
    pub include_optional_dirs: bool,
    /// Whether to include script files.
    pub include_scripts: bool,
}

/// Trait for skill templates that generate new skill structures.
pub trait SkillTemplate {
    /// Render the template to the given output directory.
    ///
    /// Creates the skill directory structure, SKILL.md file, and any
    /// additional files based on the template type and context.
    fn render(&self, ctx: &TemplateContext, output_dir: &Path) -> std::io::Result<()>;
}

/// Get a template implementation for the given template type.
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
