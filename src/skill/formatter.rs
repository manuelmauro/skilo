//! Skill manifest formatting.
//!
//! Provides consistent formatting for SKILL.md files including
//! YAML frontmatter normalization and markdown table alignment.

use crate::skill::Manifest;
use comrak::nodes::NodeValue;
use comrak::{parse_document, Arena, Options};

/// Configuration for skill formatting.
#[derive(Debug, Clone)]
pub struct FormatterConfig {
    /// Whether to format markdown tables with aligned columns.
    pub format_tables: bool,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            format_tables: true,
        }
    }
}

/// Formats SKILL.md manifests for consistent presentation.
pub struct Formatter {
    config: FormatterConfig,
}

impl Formatter {
    /// Create a new formatter with the given configuration.
    pub fn new(config: FormatterConfig) -> Self {
        Self { config }
    }

    /// Format a manifest, returning the formatted content.
    pub fn format(&self, manifest: &Manifest) -> Result<String, serde_yaml::Error> {
        let yaml = manifest.frontmatter.to_yaml()?;

        let body = if self.config.format_tables {
            format_tables(&manifest.body)
        } else {
            manifest.body.clone()
        };

        Ok(format!("---\n{}---\n\n{}", yaml, body))
    }
}

impl From<&crate::config::FmtConfig> for FormatterConfig {
    fn from(config: &crate::config::FmtConfig) -> Self {
        Self {
            format_tables: config.format_tables,
        }
    }
}

/// Format all tables in a markdown string with aligned columns.
fn format_tables(markdown: &str) -> String {
    let arena = Arena::new();

    let mut options = Options::default();
    options.extension.table = true;

    let root = parse_document(&arena, markdown, &options);

    let lines: Vec<&str> = markdown.lines().collect();
    let ends_with_newline = markdown.ends_with('\n');

    // Collect table line ranges and their formatted replacements
    let mut replacements: Vec<(usize, usize, String)> = Vec::new();

    for node in root.descendants() {
        let node_data = node.data.borrow();
        if let NodeValue::Table(node_table) = &node_data.value {
            let start_line = node_data.sourcepos.start.line;
            let end_line = node_data.sourcepos.end.line;

            let table = extract_table(node, &node_table.alignments);
            let formatted = table.format();

            // Lines are 1-indexed in sourcepos
            replacements.push((start_line, end_line, formatted));
        }
    }

    if replacements.is_empty() {
        return markdown.to_string();
    }

    // Build result by replacing table lines
    let mut result = String::new();
    let mut current_line = 1;

    for (start_line, end_line, formatted) in &replacements {
        // Add lines before this table
        for line_num in current_line..*start_line {
            if line_num > 1 {
                result.push('\n');
            }
            if let Some(line) = lines.get(line_num - 1) {
                result.push_str(line);
            }
        }

        // Add the formatted table
        if *start_line > 1 {
            result.push('\n');
        }
        result.push_str(formatted);

        current_line = end_line + 1;
    }

    // Add remaining lines after last table
    for line_num in current_line..=lines.len() {
        result.push('\n');
        if let Some(line) = lines.get(line_num - 1) {
            result.push_str(line);
        }
    }

    // Preserve trailing newline if original had one
    if ends_with_newline && !result.ends_with('\n') {
        result.push('\n');
    }

    result
}

fn extract_table<'a>(
    table_node: &'a comrak::arena_tree::Node<'a, std::cell::RefCell<comrak::nodes::Ast>>,
    alignments: &[comrak::nodes::TableAlignment],
) -> Table {
    let mut rows: Vec<Vec<String>> = Vec::new();

    for child in table_node.children() {
        let child_data = child.data.borrow();
        if let NodeValue::TableRow(_) = &child_data.value {
            let row = extract_row(child);
            rows.push(row);
        }
    }

    Table {
        alignments: alignments.to_vec(),
        rows,
    }
}

fn extract_row<'a>(
    row_node: &'a comrak::arena_tree::Node<'a, std::cell::RefCell<comrak::nodes::Ast>>,
) -> Vec<String> {
    let mut cells: Vec<String> = Vec::new();

    for cell in row_node.children() {
        let cell_data = cell.data.borrow();
        if let NodeValue::TableCell = &cell_data.value {
            let content = extract_cell_content(cell);
            cells.push(content);
        }
    }

    cells
}

fn extract_cell_content<'a>(
    cell_node: &'a comrak::arena_tree::Node<'a, std::cell::RefCell<comrak::nodes::Ast>>,
) -> String {
    let mut content = String::new();

    for child in cell_node.descendants() {
        let child_data = child.data.borrow();
        match &child_data.value {
            NodeValue::Text(text) => {
                content.push_str(text);
            }
            NodeValue::Code(code) => {
                content.push('`');
                content.push_str(&code.literal);
                content.push('`');
            }
            NodeValue::SoftBreak => {
                content.push(' ');
            }
            NodeValue::Emph | NodeValue::Strong => {}
            _ => {}
        }
    }

    content.trim().to_string()
}

struct Table {
    alignments: Vec<comrak::nodes::TableAlignment>,
    rows: Vec<Vec<String>>,
}

impl Table {
    fn format(&self) -> String {
        if self.rows.is_empty() {
            return String::new();
        }

        let col_count = self.rows.iter().map(|r| r.len()).max().unwrap_or(0);
        if col_count == 0 {
            return String::new();
        }

        // Calculate column widths
        let mut widths: Vec<usize> = vec![3; col_count]; // minimum width of 3 for separator
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        let mut result = String::new();

        // Format header row
        if let Some(header) = self.rows.first() {
            result.push_str(&self.format_row(header, &widths));
            result.push('\n');

            // Format separator row
            result.push('|');
            for (i, width) in widths.iter().enumerate() {
                let alignment = self
                    .alignments
                    .get(i)
                    .copied()
                    .unwrap_or(comrak::nodes::TableAlignment::None);
                result.push_str(&self.format_separator(*width, alignment));
                result.push('|');
            }
            result.push('\n');
        }

        // Format data rows
        for row in self.rows.iter().skip(1) {
            result.push_str(&self.format_row(row, &widths));
            result.push('\n');
        }

        // Remove trailing newline to match original behavior
        if result.ends_with('\n') {
            result.pop();
        }

        result
    }

    fn format_row(&self, row: &[String], widths: &[usize]) -> String {
        let mut result = String::from("|");
        for (i, width) in widths.iter().enumerate() {
            let cell = row.get(i).map(|s| s.as_str()).unwrap_or("");
            let alignment = self
                .alignments
                .get(i)
                .copied()
                .unwrap_or(comrak::nodes::TableAlignment::None);
            result.push(' ');
            result.push_str(&self.pad_cell(cell, *width, alignment));
            result.push_str(" |");
        }
        result
    }

    fn pad_cell(
        &self,
        content: &str,
        width: usize,
        alignment: comrak::nodes::TableAlignment,
    ) -> String {
        use comrak::nodes::TableAlignment;

        let padding = width.saturating_sub(content.len());
        match alignment {
            TableAlignment::Right => format!("{:>width$}", content, width = width),
            TableAlignment::Center => {
                let left_pad = padding / 2;
                let right_pad = padding - left_pad;
                format!(
                    "{}{}{}",
                    " ".repeat(left_pad),
                    content,
                    " ".repeat(right_pad)
                )
            }
            TableAlignment::Left | TableAlignment::None => {
                format!("{:<width$}", content, width = width)
            }
        }
    }

    fn format_separator(&self, width: usize, alignment: comrak::nodes::TableAlignment) -> String {
        use comrak::nodes::TableAlignment;

        // Separator needs to match cell width + 2 spaces
        let total_width = width + 2;
        match alignment {
            TableAlignment::Left => format!(":{}", "-".repeat(total_width - 1)),
            TableAlignment::Right => format!("{}:", "-".repeat(total_width - 1)),
            TableAlignment::Center => format!(":{}:", "-".repeat(total_width.saturating_sub(2))),
            TableAlignment::None => "-".repeat(total_width),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_table() {
        let input = r#"| Name | Description |
|---|---|
| foo | A short one |
| barbaz | A longer description |"#;

        let expected = r#"| Name   | Description          |
|--------|----------------------|
| foo    | A short one          |
| barbaz | A longer description |"#;

        assert_eq!(format_tables(input), expected);
    }

    #[test]
    fn test_format_table_with_alignment() {
        let input = r#"| Left | Center | Right |
|:---|:---:|---:|
| a | b | c |
| longer | text | here |"#;

        let output = format_tables(input);
        // Check that alignment markers are preserved
        assert!(output.contains(":---"));
        assert!(output.contains("---:"));
    }

    #[test]
    fn test_preserves_text_around_table() {
        let input = r#"# Header

Some text before.

| Col1 | Col2 |
|---|---|
| a | b |

Some text after."#;

        let output = format_tables(input);
        assert!(output.contains("# Header"));
        assert!(output.contains("Some text before."));
        assert!(output.contains("Some text after."));
    }

    #[test]
    fn test_no_table() {
        let input = "Just some text without a table.";
        assert_eq!(format_tables(input), input);
    }

    #[test]
    fn test_code_in_cells() {
        let input = r#"| Command | Description |
|---|---|
| `foo` | Run foo |"#;

        let output = format_tables(input);
        assert!(output.contains("`foo`"));
    }

    #[test]
    fn test_preserves_trailing_newline() {
        let input = "| A | B |\n|---|---|\n| 1 | 2 |\n";
        let output = format_tables(input);
        assert!(output.ends_with('\n'), "Should preserve trailing newline");
    }

    #[test]
    fn test_no_trailing_newline_when_absent() {
        let input = "| A | B |\n|---|---|\n| 1 | 2 |";
        let output = format_tables(input);
        assert!(!output.ends_with('\n'), "Should not add trailing newline");
    }
}
