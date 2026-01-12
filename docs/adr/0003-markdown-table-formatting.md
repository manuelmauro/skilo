# ADR 0003: Markdown Table Formatting

## Status

Proposed

## Context

The `skilo fmt` command currently only formats YAML frontmatter in SKILL.md files. The markdown body is preserved as-is without any normalization.

Markdown tables in skill documentation often have inconsistent column widths:

```markdown
| Name | Description | Type |
|---|---|---|
| foo | A short one | string |
| barbaz | A longer description here | number |
```

This reduces readability and creates noisy diffs when contributors format tables differently.

The `comrak` crate provides robust CommonMark + GFM parsing with table support. Leveraging it for table detection and formatting would provide consistent table presentation across all skills.

## Decision

We will extend `skilo fmt` to format markdown tables with aligned columns.

### Formatting Rules

1. **Column width**: Each column is padded to the width of its longest cell
2. **Alignment preservation**: Column alignment markers (`:---`, `:---:`, `---:`) are preserved
3. **Separator row**: Dashes are extended to match column width
4. **Whitespace**: Single space padding inside each cell

### Example

Input:
```markdown
| Name | Description | Type |
|---|---|---|
| foo | A short one | string |
| barbaz | A longer description here | number |
```

Output:
```markdown
| Name   | Description               | Type   |
|--------|---------------------------|--------|
| foo    | A short one               | string |
| barbaz | A longer description here | number |
```

### Configuration

Add to `FmtConfig` in `.skilorc.toml`:

```toml
[fmt]
format_tables = true  # default: true
```

### Implementation

1. **Table detection**: Use `comrak` to parse markdown and identify table nodes in the AST
2. **Table extraction**: Walk the AST to collect rows and cells while tracking alignment
3. **Width calculation**: Determine maximum width for each column
4. **Reformatting**: Rebuild table with padded cells and aligned separators
5. **Reconstruction**: Replace original table lines with formatted version using source positions

### Module Structure

```
src/skill/
├── mod.rs
├── table_fmt.rs    # New: table formatting logic
└── ...
```

### API

```rust
// src/skill/table_fmt.rs
pub fn format_tables(markdown: &str) -> String;
```

### Integration

Modify `Manifest::to_string_formatted()` to optionally apply table formatting to the body content before reconstruction.

## Consequences

### Positive

- Consistent table presentation across all skills
- Improved readability of SKILL.md files
- Cleaner diffs when multiple contributors edit tables
- Uses battle-tested `comrak` crate (used by crates.io, GitLab, docs.rs)

### Negative

- Increased processing time for `fmt` command (parsing full markdown)
- May reformat tables that were intentionally formatted differently
- Additional code complexity in formatting pipeline

### Neutral

- Opt-out available via configuration
- Only affects tables, other markdown elements unchanged

## References

- [comrak Documentation](https://docs.rs/comrak/latest/comrak/)
- [GitHub Flavored Markdown Tables](https://github.github.com/gfm/#tables-extension-)
