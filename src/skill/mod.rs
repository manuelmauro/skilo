//! Skill parsing, validation, and formatting.
//!
//! This module provides types for working with Agent Skills:
//! - [`Manifest`] - Parse SKILL.md files
//! - [`Frontmatter`] - Skill metadata (name, description, etc.)
//! - [`Discovery`] - Find skills in directories
//! - [`Validator`] - Validate skills against the specification

pub mod discovery;
pub mod formatter;
pub mod frontmatter;
pub mod manifest;
pub mod rules;
pub mod validator;

pub use discovery::Discovery;
pub use formatter::{Formatter, FormatterConfig};
pub use frontmatter::Frontmatter;
pub use manifest::Manifest;
pub use validator::{Diagnostic, DiagnosticCode, ValidationResult, Validator};
