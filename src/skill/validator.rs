//! Skill validation.

use crate::config::LintConfig;
use crate::skill::manifest::Manifest;
use crate::skill::rules::{
    BodyLengthRule, CompatibilityLengthRule, DescriptionLengthRule, DescriptionRequiredRule,
    NameDirectoryRule, NameFormatRule, NameLengthRule, ReferencesExistRule, Rule,
    ScriptExecutableRule, ScriptShebangRule,
};

/// Result of validating a skill.
#[derive(Debug, Default)]
pub struct ValidationResult {
    /// Validation errors.
    pub errors: Vec<Diagnostic>,
    /// Validation warnings.
    pub warnings: Vec<Diagnostic>,
}

impl ValidationResult {
    /// Returns true if there are no errors.
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns true if there are no errors or warnings.
    pub fn is_ok_strict(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }

    /// Merge another result into this one.
    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

/// A validation diagnostic (error or warning).
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Path to the file with the issue.
    pub path: String,
    /// Line number (if applicable).
    pub line: Option<usize>,
    /// Column number (if applicable).
    pub column: Option<usize>,
    /// Human-readable message.
    pub message: String,
    /// Diagnostic code.
    pub code: DiagnosticCode,
    /// Optional hint for fixing the issue.
    pub fix_hint: Option<String>,
}

/// Diagnostic codes for validation issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCode {
    /// Invalid name format.
    E001,
    /// Name too long.
    E002,
    /// Name mismatch with directory.
    E003,
    /// Missing description.
    E004,
    /// Description too long.
    E005,
    /// Compatibility too long.
    E006,
    /// Invalid YAML.
    E007,
    /// Missing SKILL.md.
    E008,
    /// Referenced file not found.
    E009,

    /// Body exceeds max lines.
    W001,
    /// Script not executable.
    W002,
    /// Script missing shebang.
    W003,
    /// Empty optional directory.
    W004,
}

impl std::fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::E001 => write!(f, "E001"),
            Self::E002 => write!(f, "E002"),
            Self::E003 => write!(f, "E003"),
            Self::E004 => write!(f, "E004"),
            Self::E005 => write!(f, "E005"),
            Self::E006 => write!(f, "E006"),
            Self::E007 => write!(f, "E007"),
            Self::E008 => write!(f, "E008"),
            Self::E009 => write!(f, "E009"),
            Self::W001 => write!(f, "W001"),
            Self::W002 => write!(f, "W002"),
            Self::W003 => write!(f, "W003"),
            Self::W004 => write!(f, "W004"),
        }
    }
}

impl DiagnosticCode {
    /// Returns true if this is an error (not a warning).
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            Self::E001
                | Self::E002
                | Self::E003
                | Self::E004
                | Self::E005
                | Self::E006
                | Self::E007
                | Self::E008
                | Self::E009
        )
    }
}

/// Skill validator with configurable rules.
pub struct Validator {
    rules: Vec<Box<dyn Rule>>,
}

impl Default for Validator {
    fn default() -> Self {
        Self::new(&LintConfig::default())
    }
}

impl Validator {
    /// Create a new validator with the given configuration.
    pub fn new(config: &LintConfig) -> Self {
        let mut rules: Vec<Box<dyn Rule>> = Vec::new();

        if config.rules.name_format {
            rules.push(Box::new(NameFormatRule));
        }
        if let Some(max) = config.rules.name_length.resolve(64) {
            rules.push(Box::new(NameLengthRule::new(max)));
        }
        if config.rules.name_directory {
            rules.push(Box::new(NameDirectoryRule));
        }
        if config.rules.description_required {
            rules.push(Box::new(DescriptionRequiredRule));
        }
        if let Some(max) = config.rules.description_length.resolve(1024) {
            rules.push(Box::new(DescriptionLengthRule::new(max)));
        }
        if let Some(max) = config.rules.compatibility_length.resolve(500) {
            rules.push(Box::new(CompatibilityLengthRule::new(max)));
        }
        if config.rules.references_exist {
            rules.push(Box::new(ReferencesExistRule));
        }
        if let Some(max) = config.rules.body_length.resolve(500) {
            rules.push(Box::new(BodyLengthRule::new(max)));
        }
        if config.rules.script_executable {
            rules.push(Box::new(ScriptExecutableRule));
        }
        if config.rules.script_shebang {
            rules.push(Box::new(ScriptShebangRule));
        }

        Self { rules }
    }

    /// Validate a skill manifest.
    pub fn validate(&self, manifest: &Manifest) -> ValidationResult {
        let mut result = ValidationResult::default();

        for rule in &self.rules {
            let diagnostics = rule.check(manifest);
            for diag in diagnostics {
                if diag.code.is_error() {
                    result.errors.push(diag);
                } else {
                    result.warnings.push(diag);
                }
            }
        }

        result
    }
}
