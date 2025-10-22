use super::super::types::{RenderableData, RenderOptions, RenderError, RenderResult};
use std::fmt::Write;

/// Table formatter for structured data
pub struct TableFormatter;

impl TableFormatter {
    /// Format data as a table
    pub fn format(data: &RenderableData, options: &RenderOptions) -> RenderResult<String> {
        if data.is_empty() {
            return Ok("No data available".to_string());
        }

        let mut output = String::new();

        // Add title if present
        if let Some(title) = &data.title {
            writeln!(output, "{}", title)?;
            writeln!(output, "{}", "=".repeat(title.len()))?;
            writeln!(output)?;
        }

        // Calculate column widths
        let column_widths = Self::calculate_column_widths(data, options);

        // Format headers
        Self::format_row(&mut output, &data.headers, &column_widths, options)?;
        writeln!(output)?;

        // Format separator line
        Self::format_separator(&mut output, &column_widths)?;
        writeln!(output)?;

        // Format rows
        for row in &data.rows {
            Self::format_row(&mut output, row, &column_widths, options)?;
            writeln!(output)?;
        }

        Ok(output)
    }

    /// Calculate optimal column widths
    fn calculate_column_widths(data: &RenderableData, options: &RenderOptions) -> Vec<usize> {
        let mut widths: Vec<usize> = data.headers.iter().map(|h| h.len()).collect();

        for row in &data.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    let cell_width = if options.table_truncate {
                        std::cmp::min(cell.len(), options.table_max_column_width.unwrap_or(usize::MAX))
                    } else {
                        cell.len()
                    };
                    widths[i] = widths[i].max(cell_width);
                }
            }
        }

        // Apply max column width constraint
        if let Some(max_width) = options.table_max_column_width {
            for width in &mut widths {
                *width = std::cmp::min(*width, max_width);
            }
        }

        widths
    }

    /// Format a single row
    fn format_row(
        output: &mut String,
        row: &[String],
        column_widths: &[usize],
        options: &RenderOptions,
    ) -> RenderResult<()> {
        for (i, cell) in row.iter().enumerate() {
            if i < column_widths.len() {
                let width = column_widths[i];
                let cell_content = if options.table_truncate && cell.len() > width {
                    if width > 3 {
                        format!("{}...", &cell[..width - 3])
                    } else {
                        cell.chars().take(width).collect()
                    }
                } else {
                    cell.clone()
                };

                write!(output, "{:<width$}", cell_content, width = width)?;

                if i < row.len() - 1 {
                    write!(output, " ")?;
                }
            }
        }
        Ok(())
    }

    /// Format separator line
    fn format_separator(output: &mut String, column_widths: &[usize]) -> RenderResult<()> {
        for (i, width) in column_widths.iter().enumerate() {
            write!(output, "{}", "-".repeat(*width))?;
            if i < column_widths.len() - 1 {
                write!(output, " ")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_data() -> RenderableData {
        RenderableData::new(
            vec!["Name".to_string(), "Age".to_string(), "City".to_string()],
            vec![
                vec!["John".to_string(), "25".to_string(), "New York".to_string()],
                vec!["Jane".to_string(), "30".to_string(), "London".to_string()],
            ],
        )
    }

    #[test]
    fn test_format_basic_table() {
        let data = create_test_data();
        let options = RenderOptions::default();
        let result = TableFormatter::format(&data, &options).unwrap();

        assert!(result.contains("Name"));
        assert!(result.contains("Age"));
        assert!(result.contains("City"));
        assert!(result.contains("John"));
        assert!(result.contains("Jane"));
    }

    #[test]
    fn test_format_empty_data() {
        let data = RenderableData::new(vec![], vec![]);
        let options = RenderOptions::default();
        let result = TableFormatter::format(&data, &options).unwrap();

        assert_eq!(result, "No data available");
    }

    #[test]
    fn test_format_with_title() {
        let data = create_test_data().with_title("Test Data".to_string());
        let options = RenderOptions::default();
        let result = TableFormatter::format(&data, &options).unwrap();

        assert!(result.contains("Test Data"));
        assert!(result.contains("========="));
    }

    #[test]
    fn test_format_with_truncation() {
        let data = RenderableData::new(
            vec!["Name".to_string(), "Description".to_string()],
            vec![vec![
                "John".to_string(),
                "This is a very long description that should be truncated".to_string(),
            ]],
        );
        let mut options = RenderOptions::default();
        options.table_max_column_width = Some(10);
        options.table_truncate = true;

        let result = TableFormatter::format(&data, &options).unwrap();
        assert!(result.contains("This is..."));
    }
}
