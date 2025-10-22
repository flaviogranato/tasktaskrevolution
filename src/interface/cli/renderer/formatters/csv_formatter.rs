use super::super::types::{RenderableData, RenderOptions, RenderError, RenderResult};

/// CSV formatter for structured data
pub struct CsvFormatter;

impl CsvFormatter {
    /// Format data as CSV
    pub fn format(data: &RenderableData, options: &RenderOptions) -> RenderResult<String> {
        if data.is_empty() {
            return Ok("No data available".to_string());
        }

        let mut output = String::new();

        // Add headers if requested
        if options.csv_include_headers {
            Self::format_row(&mut output, &data.headers)?;
        }

        // Add data rows
        for row in &data.rows {
            Self::format_row(&mut output, row)?;
        }

        Ok(output)
    }

    /// Format a single row as CSV
    fn format_row(output: &mut String, row: &[String]) -> RenderResult<()> {
        let mut formatted_cells = Vec::new();

        for cell in row {
            let escaped_cell = Self::escape_csv_cell(cell);
            formatted_cells.push(escaped_cell);
        }

        output.push_str(&formatted_cells.join(","));
        output.push('\n');

        Ok(())
    }

    /// Escape a CSV cell according to RFC 4180
    fn escape_csv_cell(cell: &str) -> String {
        // Check if cell needs escaping
        let needs_escaping = cell.contains(',') 
            || cell.contains('"') 
            || cell.contains('\n') 
            || cell.contains('\r')
            || cell.starts_with(' ')
            || cell.ends_with(' ');

        if needs_escaping {
            // Escape quotes by doubling them and wrap in quotes
            let escaped = cell.replace('"', "\"\"");
            format!("\"{}\"", escaped)
        } else {
            cell.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_format_basic_csv() {
        let data = create_test_data();
        let options = RenderOptions::default();
        let result = CsvFormatter::format(&data, &options).unwrap();

        assert!(result.contains("Name,Age,City"));
        assert!(result.contains("John,25,New York"));
        assert!(result.contains("Jane,30,London"));
    }

    #[test]
    fn test_format_csv_without_headers() {
        let data = create_test_data();
        let mut options = RenderOptions::default();
        options.csv_include_headers = false;
        
        let result = CsvFormatter::format(&data, &options).unwrap();

        assert!(!result.contains("Name,Age,City"));
        assert!(result.contains("John,25,New York"));
        assert!(result.contains("Jane,30,London"));
    }

    #[test]
    fn test_format_empty_data() {
        let data = RenderableData::new(vec![], vec![]);
        let options = RenderOptions::default();
        let result = CsvFormatter::format(&data, &options).unwrap();

        assert_eq!(result, "No data available");
    }

    #[test]
    fn test_escape_csv_cell() {
        // Test cells that need escaping
        assert_eq!(CsvFormatter::escape_csv_cell("Hello, World"), "\"Hello, World\"");
        assert_eq!(CsvFormatter::escape_csv_cell("He said \"Hello\""), "\"He said \"\"Hello\"\"\"");
        assert_eq!(CsvFormatter::escape_csv_cell("Line1\nLine2"), "\"Line1\nLine2\"");
        assert_eq!(CsvFormatter::escape_csv_cell(" Leading space"), "\" Leading space\"");
        assert_eq!(CsvFormatter::escape_csv_cell("Trailing space "), "\"Trailing space \"");

        // Test cells that don't need escaping
        assert_eq!(CsvFormatter::escape_csv_cell("Simple"), "Simple");
        assert_eq!(CsvFormatter::escape_csv_cell("123"), "123");
        assert_eq!(CsvFormatter::escape_csv_cell(""), "");
    }

    #[test]
    fn test_format_csv_with_special_characters() {
        let data = RenderableData::new(
            vec!["Name".to_string(), "Description".to_string()],
            vec![
                vec!["John".to_string(), "A person with, comma".to_string()],
                vec!["Jane".to_string(), "A person with \"quotes\"".to_string()],
                vec!["Bob".to_string(), "A person with\nnewline".to_string()],
            ],
        );
        let options = RenderOptions::default();
        let result = CsvFormatter::format(&data, &options).unwrap();

        assert!(result.contains("\"A person with, comma\""));
        assert!(result.contains("\"A person with \"\"quotes\"\"\""));
        assert!(result.contains("\"A person with\nnewline\""));
    }
}


