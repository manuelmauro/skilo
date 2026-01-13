//! Output formatting for command results.

mod json;
mod sarif;
mod text;

use crate::cli::OutputFormat;
use crate::skill::ValidationResult;

pub use json::JsonFormatter;
pub use sarif::SarifFormatter;
pub use text::TextFormatter;

/// Trait for formatting command output.
pub trait OutputFormatter {
    /// Format validation results.
    fn format_validation(&self, results: &[(String, ValidationResult)]) -> String;
    /// Format an informational message.
    fn format_message(&self, message: &str);
    /// Format an error message.
    fn format_error(&self, message: &str);
    /// Format a success message.
    fn format_success(&self, message: &str);
}

/// Get a formatter for the given output format.
pub fn get_formatter(format: OutputFormat, quiet: bool) -> Box<dyn OutputFormatter> {
    match format {
        OutputFormat::Text => Box::new(TextFormatter::new(quiet)),
        OutputFormat::Json => Box::new(JsonFormatter::new(quiet)),
        OutputFormat::Sarif => Box::new(SarifFormatter::new(quiet)),
    }
}
