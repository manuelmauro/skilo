//! Error types for the skilo crate.

use miette::Diagnostic;
use thiserror::Error;

/// Errors that can occur during skilo operations.
#[derive(Error, Diagnostic, Debug)]
pub enum SkiloError {
    /// A skill with the given name already exists.
    #[error("Skill '{name}' already exists at {path}")]
    #[diagnostic(code(skilo::skill_exists))]
    SkillExists {
        /// The skill name.
        name: String,
        /// The path where the skill exists.
        path: String,
    },

    /// The skill name is invalid.
    #[error(
        "Invalid skill name '{0}': must be 1-64 lowercase alphanumeric chars with single hyphens"
    )]
    #[diagnostic(code(skilo::invalid_name))]
    InvalidName(String),

    /// No skills were found at the given path.
    #[error("No skills found in {path}")]
    #[diagnostic(code(skilo::no_skills))]
    NoSkillsFound {
        /// The path that was searched.
        path: String,
    },

    /// A configuration error occurred.
    #[error("Configuration error: {0}")]
    #[diagnostic(code(skilo::config))]
    Config(String),

    /// Validation failed with the given number of errors.
    #[error("Validation failed with {0} error(s)")]
    #[diagnostic(code(skilo::validation_failed))]
    ValidationFailed(usize),

    /// Format check failed with the given number of files needing formatting.
    #[error("Format check failed: {0} file(s) need formatting")]
    #[diagnostic(code(skilo::format_failed))]
    FormatCheckFailed(usize),

    /// An error occurred while parsing a manifest.
    #[error("Manifest error: {0}")]
    #[diagnostic(code(skilo::manifest))]
    Manifest(#[from] crate::skill::manifest::ManifestError),

    /// An I/O error occurred.
    #[error("IO error: {0}")]
    #[diagnostic(code(skilo::io))]
    Io(#[from] std::io::Error),

    /// Invalid source format for the add command.
    #[error("Invalid source format: {0}. {1}")]
    #[diagnostic(code(skilo::invalid_source))]
    InvalidSource(String, String),

    /// Git operation failed.
    #[error("Git error: {message}")]
    #[diagnostic(code(skilo::git))]
    Git {
        /// The error message.
        message: String,
    },

    /// Repository not found.
    #[error("Repository not found: {url}")]
    #[diagnostic(code(skilo::repo_not_found))]
    RepoNotFound {
        /// The repository URL.
        url: String,
    },

    /// Network error.
    #[error("Network error: {message}")]
    #[diagnostic(code(skilo::network))]
    Network {
        /// The error message.
        message: String,
    },

    /// User cancelled the operation.
    #[error("Operation cancelled by user")]
    #[diagnostic(code(skilo::cancelled))]
    Cancelled,
}

/// A specialized Result type for skilo operations.
pub type Result<T> = std::result::Result<T, SkiloError>;
