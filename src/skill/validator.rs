use crate::skill::manifest::Manifest;
use once_cell::sync::Lazy;
use regex::Regex;

static NAME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").unwrap());

static REF_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"`((?:scripts|references|assets)/[^`]+)`").unwrap());

#[derive(Debug, Default)]
pub struct ValidationResult {
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
}

impl ValidationResult {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn is_ok_strict(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }

    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub path: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub message: String,
    pub code: DiagnosticCode,
    pub fix_hint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCode {
    // Errors
    E001, // Invalid name format
    E002, // Name too long
    E003, // Name mismatch with directory
    E004, // Missing description
    E005, // Description too long
    E006, // Compatibility too long
    E007, // Invalid YAML
    E008, // Missing SKILL.md
    E009, // Referenced file not found

    // Warnings
    W001, // Body exceeds 500 lines
    W002, // Script not executable
    W003, // Script missing shebang
    W004, // Empty optional directory
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

pub struct Validator {
    pub max_body_lines: usize,
}

impl Default for Validator {
    fn default() -> Self {
        Self {
            max_body_lines: 500,
        }
    }
}

impl Validator {
    pub fn new(max_body_lines: usize) -> Self {
        Self { max_body_lines }
    }

    pub fn validate(&self, manifest: &Manifest) -> ValidationResult {
        let mut result = ValidationResult::default();

        // Validate name
        self.validate_name(manifest, &mut result);

        // Validate description
        self.validate_description(manifest, &mut result);

        // Validate compatibility
        self.validate_compatibility(manifest, &mut result);

        // Validate body length
        self.validate_body(manifest, &mut result);

        // Validate file references
        self.validate_references(manifest, &mut result);

        // Validate scripts
        self.validate_scripts(manifest, &mut result);

        result
    }

    fn validate_name(&self, manifest: &Manifest, result: &mut ValidationResult) {
        let name = &manifest.frontmatter.name;
        let path_str = manifest.path.display().to_string();

        // Check format
        if !NAME_REGEX.is_match(name) {
            result.errors.push(Diagnostic {
                path: path_str.clone(),
                line: Some(2),
                column: Some(7),
                message: format!(
                    "Invalid name '{}': must be lowercase alphanumeric with single hyphens",
                    name
                ),
                code: DiagnosticCode::E001,
                fix_hint: Some("Use only lowercase letters, numbers, and single hyphens".into()),
            });
        }

        // Check length
        if name.len() > 64 {
            result.errors.push(Diagnostic {
                path: path_str.clone(),
                line: Some(2),
                column: Some(7),
                message: format!("Name too long ({} chars, max 64)", name.len()),
                code: DiagnosticCode::E002,
                fix_hint: None,
            });
        }

        // Check directory match
        if let Some(parent) = manifest.path.parent() {
            if let Some(dir_name) = parent.file_name().and_then(|n| n.to_str()) {
                if dir_name != name {
                    result.errors.push(Diagnostic {
                        path: path_str,
                        line: Some(2),
                        column: Some(7),
                        message: format!(
                            "Name '{}' does not match directory name '{}'",
                            name, dir_name
                        ),
                        code: DiagnosticCode::E003,
                        fix_hint: Some(format!(
                            "Rename to '{}' or move to '{}/SKILL.md'",
                            dir_name, name
                        )),
                    });
                }
            }
        }
    }

    fn validate_description(&self, manifest: &Manifest, result: &mut ValidationResult) {
        let desc = &manifest.frontmatter.description;
        let path_str = manifest.path.display().to_string();

        if desc.is_empty() {
            result.errors.push(Diagnostic {
                path: path_str.clone(),
                line: Some(3),
                column: Some(14),
                message: "Description cannot be empty".into(),
                code: DiagnosticCode::E004,
                fix_hint: None,
            });
        }

        if desc.len() > 1024 {
            result.errors.push(Diagnostic {
                path: path_str,
                line: Some(3),
                column: Some(14),
                message: format!("Description too long ({} chars, max 1024)", desc.len()),
                code: DiagnosticCode::E005,
                fix_hint: None,
            });
        }
    }

    fn validate_compatibility(&self, manifest: &Manifest, result: &mut ValidationResult) {
        if let Some(compat) = &manifest.frontmatter.compatibility {
            if compat.len() > 500 {
                result.errors.push(Diagnostic {
                    path: manifest.path.display().to_string(),
                    line: None,
                    column: None,
                    message: format!("Compatibility too long ({} chars, max 500)", compat.len()),
                    code: DiagnosticCode::E006,
                    fix_hint: None,
                });
            }
        }
    }

    fn validate_body(&self, manifest: &Manifest, result: &mut ValidationResult) {
        let line_count = manifest.body.lines().count();
        if line_count > self.max_body_lines {
            result.warnings.push(Diagnostic {
                path: manifest.path.display().to_string(),
                line: Some(manifest.body_start_line + self.max_body_lines),
                column: None,
                message: format!(
                    "Body exceeds recommended {} lines ({} lines). Consider using references/",
                    self.max_body_lines, line_count
                ),
                code: DiagnosticCode::W001,
                fix_hint: Some("Move detailed content to references/ directory".into()),
            });
        }
    }

    fn validate_references(&self, manifest: &Manifest, result: &mut ValidationResult) {
        let Some(skill_dir) = manifest.path.parent() else {
            return;
        };

        for cap in REF_REGEX.captures_iter(&manifest.body) {
            let ref_path = &cap[1];
            let full_path = skill_dir.join(ref_path);

            if !full_path.exists() {
                result.errors.push(Diagnostic {
                    path: manifest.path.display().to_string(),
                    line: None,
                    column: None,
                    message: format!("Referenced file not found: {}", ref_path),
                    code: DiagnosticCode::E009,
                    fix_hint: Some(format!("Create {} or remove the reference", ref_path)),
                });
            }
        }
    }

    fn validate_scripts(&self, manifest: &Manifest, result: &mut ValidationResult) {
        let Some(skill_dir) = manifest.path.parent() else {
            return;
        };
        let scripts_dir = skill_dir.join("scripts");

        if !scripts_dir.exists() {
            return;
        }

        let Ok(entries) = std::fs::read_dir(&scripts_dir) else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            // Check executable permission (Unix only)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = path.metadata() {
                    if meta.permissions().mode() & 0o111 == 0 {
                        result.warnings.push(Diagnostic {
                            path: path.display().to_string(),
                            line: None,
                            column: None,
                            message: "Script is not executable".into(),
                            code: DiagnosticCode::W002,
                            fix_hint: Some(format!("Run: chmod +x {}", path.display())),
                        });
                    }
                }
            }

            // Check shebang
            if let Ok(content) = std::fs::read_to_string(&path) {
                if !content.starts_with("#!") {
                    result.warnings.push(Diagnostic {
                        path: path.display().to_string(),
                        line: Some(1),
                        column: Some(1),
                        message: "Script missing shebang line".into(),
                        code: DiagnosticCode::W003,
                        fix_hint: Some("Add #!/usr/bin/env <interpreter> as first line".into()),
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_names() {
        assert!(NAME_REGEX.is_match("my-skill"));
        assert!(NAME_REGEX.is_match("skill123"));
        assert!(NAME_REGEX.is_match("a"));
        assert!(NAME_REGEX.is_match("my-cool-skill"));
    }

    #[test]
    fn test_invalid_names() {
        assert!(!NAME_REGEX.is_match("My-Skill")); // uppercase
        assert!(!NAME_REGEX.is_match("-skill")); // leading hyphen
        assert!(!NAME_REGEX.is_match("skill-")); // trailing hyphen
        assert!(!NAME_REGEX.is_match("my--skill")); // consecutive hyphens
        assert!(!NAME_REGEX.is_match("my_skill")); // underscore
        assert!(!NAME_REGEX.is_match("")); // empty
    }
}
