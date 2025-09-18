use std::fmt;

/// Table formatter for Kubernetes-style output
pub struct TableFormatter {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    column_widths: Vec<usize>,
}

impl TableFormatter {
    /// Create a new table formatter
    pub fn new(headers: Vec<String>) -> Self {
        let column_widths = headers.iter().map(|h| h.len()).collect();
        Self {
            headers,
            rows: Vec::new(),
            column_widths,
        }
    }

    /// Add a row to the table
    pub fn add_row(&mut self, row: Vec<String>) {
        // Update column widths based on new row
        for (i, cell) in row.iter().enumerate() {
            if i < self.column_widths.len() {
                self.column_widths[i] = self.column_widths[i].max(cell.len());
            }
        }
        self.rows.push(row);
    }

    /// Format the table as a string
    pub fn format(&self) -> String {
        let mut output = String::new();

        // Print headers
        output.push_str(&self.format_row(&self.headers));
        output.push('\n');

        // Print separator line
        output.push_str(&self.format_separator());
        output.push('\n');

        // Print rows
        for row in &self.rows {
            output.push_str(&self.format_row(row));
            output.push('\n');
        }

        output
    }

    /// Format a single row
    fn format_row(&self, row: &[String]) -> String {
        let mut formatted = String::new();

        for (i, cell) in row.iter().enumerate() {
            if i < self.column_widths.len() {
                let width = self.column_widths[i];
                let padded = format!("{:<width$}", cell, width = width);
                formatted.push_str(&padded);

                if i < row.len() - 1 {
                    formatted.push(' ');
                }
            }
        }

        formatted
    }

    /// Format separator line
    fn format_separator(&self) -> String {
        let mut separator = String::new();

        for (i, &width) in self.column_widths.iter().enumerate() {
            separator.push_str(&"-".repeat(width));

            if i < self.column_widths.len() - 1 {
                separator.push(' ');
            }
        }

        separator
    }
}

impl fmt::Display for TableFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_formatter_basic() {
        let mut table = TableFormatter::new(vec!["NAME".to_string(), "CODE".to_string(), "STATUS".to_string()]);

        table.add_row(vec![
            "Test Company".to_string(),
            "TEST-001".to_string(),
            "Active".to_string(),
        ]);

        table.add_row(vec![
            "Another Company".to_string(),
            "TEST-002".to_string(),
            "Inactive".to_string(),
        ]);

        let output = table.format();
        assert!(output.contains("NAME"));
        assert!(output.contains("CODE"));
        assert!(output.contains("STATUS"));
        assert!(output.contains("Test Company"));
        assert!(output.contains("Another Company"));
    }

    #[test]
    fn test_table_formatter_empty() {
        let table = TableFormatter::new(vec!["NAME".to_string(), "CODE".to_string()]);

        let output = table.format();
        assert!(output.contains("NAME"));
        assert!(output.contains("CODE"));
        assert!(!output.contains("Test Company"));
    }
}
