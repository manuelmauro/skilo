use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    /// Skill name (required, 1-64 chars, lowercase alphanumeric + hyphens)
    pub name: String,

    /// Skill description (required, 1-1024 chars)
    pub description: String,

    /// License identifier or file reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Compatibility requirements (max 500 chars)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<String>,

    /// Additional metadata key-value pairs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// Pre-approved tools (space-delimited)
    #[serde(rename = "allowed-tools", skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<String>,
}

impl Frontmatter {
    /// Canonical key ordering for formatting
    pub const KEY_ORDER: &'static [&'static str] = &[
        "name",
        "description",
        "license",
        "compatibility",
        "metadata",
        "allowed-tools",
    ];

    /// Serialize to YAML with canonical key ordering
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }
}
