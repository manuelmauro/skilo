//! Validates the length of the compatibility field in frontmatter.

use crate::skill::manifest::Manifest;
use crate::skill::rules::Rule;
use crate::skill::validator::{Diagnostic, DiagnosticCode};

/// E006: Validates compatibility field length.
pub struct CompatibilityLengthRule {
    /// Maximum allowed compatibility string length.
    max_length: usize,
}

impl CompatibilityLengthRule {
    /// Create a new compatibility length rule with the specified maximum.
    pub fn new(max_length: usize) -> Self {
        Self { max_length }
    }
}

impl Rule for CompatibilityLengthRule {
    fn name(&self) -> &'static str {
        "compatibility-length"
    }

    fn check(&self, manifest: &Manifest) -> Vec<Diagnostic> {
        let Some(compat) = &manifest.frontmatter.compatibility else {
            return Vec::new();
        };

        if compat.len() <= self.max_length {
            return Vec::new();
        }

        vec![Diagnostic {
            path: manifest.path.display().to_string(),
            line: None,
            column: None,
            message: format!(
                "Compatibility too long ({} chars, max {})",
                compat.len(),
                self.max_length
            ),
            code: DiagnosticCode::E006,
            fix_hint: None,
        }]
    }
}
