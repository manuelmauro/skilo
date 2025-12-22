mod json;
mod sarif;
mod text;

use crate::cli::OutputFormat;
use crate::skill::ValidationResult;

pub use json::JsonFormatter;
pub use sarif::SarifFormatter;
pub use text::TextFormatter;

pub trait OutputFormatter {
    fn format_validation(&self, results: &[(String, ValidationResult)]) -> String;
    fn format_message(&self, message: &str);
    fn format_error(&self, message: &str);
    fn format_success(&self, message: &str);
}

pub fn get_formatter(format: OutputFormat, quiet: bool) -> Box<dyn OutputFormatter> {
    match format {
        OutputFormat::Text => Box::new(TextFormatter::new(quiet)),
        OutputFormat::Json => Box::new(JsonFormatter::new(quiet)),
        OutputFormat::Sarif => Box::new(SarifFormatter::new(quiet)),
    }
}
