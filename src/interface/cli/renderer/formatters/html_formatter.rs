use super::super::types::{RenderableData, RenderOptions, RenderError, RenderResult};
use std::fmt::Write;

/// HTML formatter for structured data
pub struct HtmlFormatter;

impl HtmlFormatter {
    /// Format data as HTML table
    pub fn format(data: &RenderableData, options: &RenderOptions) -> RenderResult<String> {
        if data.is_empty() {
            return Ok(Self::format_empty_table());
        }

        let mut output = String::new();

        // Start HTML document
        output.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        Self::write_head(&mut output, options)?;
        output.push_str("</head>\n<body>\n");

        // Add title if present
        if let Some(title) = &data.title {
            writeln!(output, "<h1>{}</h1>", Self::escape_html(title))?;
        }

        // Start table
        let table_class = options.html_table_class.as_deref().unwrap_or("ttr-table");
        writeln!(output, "<table class=\"{}\">", table_class)?;

        // Add headers
        output.push_str("<thead>\n<tr>\n");
        for header in &data.headers {
            writeln!(output, "<th>{}</th>", Self::escape_html(header))?;
        }
        output.push_str("</tr>\n</thead>\n");

        // Add data rows
        output.push_str("<tbody>\n");
        for row in &data.rows {
            output.push_str("<tr>\n");
            for cell in row {
                writeln!(output, "<td>{}</td>", Self::escape_html(cell))?;
            }
            output.push_str("</tr>\n");
        }
        output.push_str("</tbody>\n");

        // End table
        output.push_str("</table>\n");

        // Add metadata if present
        if let Some(metadata) = &data.metadata {
            Self::write_metadata(&mut output, metadata)?;
        }

        // End HTML document
        output.push_str("</body>\n</html>");

        Ok(output)
    }

    /// Write HTML head section
    fn write_head(output: &mut String, options: &RenderOptions) -> RenderResult<()> {
        writeln!(output, "<meta charset=\"UTF-8\">")?;
        writeln!(output, "<title>TTR Output</title>")?;

        if options.html_inline_styles {
            Self::write_inline_styles(output)?;
        }

        Ok(())
    }

    /// Write inline CSS styles
    fn write_inline_styles(output: &mut String) -> RenderResult<()> {
        writeln!(output, "<style>")?;
        writeln!(output, "  .ttr-table {{")?;
        writeln!(output, "    border-collapse: collapse;")?;
        writeln!(output, "    width: 100%;")?;
        writeln!(output, "    margin: 20px 0;")?;
        writeln!(output, "  }}")?;
        writeln!(output, "  .ttr-table th, .ttr-table td {{")?;
        writeln!(output, "    border: 1px solid #ddd;")?;
        writeln!(output, "    padding: 8px;")?;
        writeln!(output, "    text-align: left;")?;
        writeln!(output, "  }}")?;
        writeln!(output, "  .ttr-table th {{")?;
        writeln!(output, "    background-color: #f2f2f2;")?;
        writeln!(output, "    font-weight: bold;")?;
        writeln!(output, "  }}")?;
        writeln!(output, "  .ttr-table tr:nth-child(even) {{")?;
        writeln!(output, "    background-color: #f9f9f9;")?;
        writeln!(output, "  }}")?;
        writeln!(output, "  .ttr-table tr:hover {{")?;
        writeln!(output, "    background-color: #f5f5f5;")?;
        writeln!(output, "  }}")?;
        writeln!(output, "</style>")?;
        Ok(())
    }

    /// Write metadata section
    fn write_metadata(output: &mut String, metadata: &std::collections::HashMap<String, serde_json::Value>) -> RenderResult<()> {
        writeln!(output, "<div class=\"metadata\">")?;
        writeln!(output, "<h3>Metadata</h3>")?;
        writeln!(output, "<ul>")?;
        
        for (key, value) in metadata {
            writeln!(output, "<li><strong>{}:</strong> {}</li>", 
                Self::escape_html(key), 
                Self::escape_html(&value.to_string())
            )?;
        }
        
        writeln!(output, "</ul>")?;
        writeln!(output, "</div>")?;
        Ok(())
    }

    /// Format empty table
    fn format_empty_table() -> String {
        "<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"UTF-8\">\n<title>TTR Output</title>\n</head>\n<body>\n<p>No data available</p>\n</body>\n</html>".to_string()
    }

    /// Escape HTML special characters
    fn escape_html(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_data() -> RenderableData {
        RenderableData::new(
            vec!["Name".to_string(), "Age".to_string()],
            vec![
                vec!["John".to_string(), "25".to_string()],
                vec!["Jane".to_string(), "30".to_string()],
            ],
        )
    }

    #[test]
    fn test_format_basic_html() {
        let data = create_test_data();
        let options = RenderOptions::default();
        let result = HtmlFormatter::format(&data, &options).unwrap();

        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("<table class=\"ttr-table\">"));
        assert!(result.contains("<th>Name</th>"));
        assert!(result.contains("<th>Age</th>"));
        assert!(result.contains("<td>John</td>"));
        assert!(result.contains("<td>25</td>"));
        assert!(result.contains("<td>Jane</td>"));
        assert!(result.contains("<td>30</td>"));
    }

    #[test]
    fn test_format_html_with_title() {
        let data = create_test_data().with_title("Test Data".to_string());
        let options = RenderOptions::default();
        let result = HtmlFormatter::format(&data, &options).unwrap();

        assert!(result.contains("<h1>Test Data</h1>"));
    }

    #[test]
    fn test_format_html_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String("test".to_string()));
        metadata.insert("version".to_string(), serde_json::Value::String("1.0".to_string()));

        let data = create_test_data().with_metadata_map(metadata);
        let options = RenderOptions::default();
        let result = HtmlFormatter::format(&data, &options).unwrap();

        assert!(result.contains("<div class=\"metadata\">"));
        assert!(result.contains("<h3>Metadata</h3>"));
        assert!(result.contains("<li><strong>source:</strong> &quot;test&quot;</li>"));
        assert!(result.contains("<li><strong>version:</strong> &quot;1.0&quot;</li>"));
    }

    #[test]
    fn test_format_empty_html() {
        let data = RenderableData::new(vec![], vec![]);
        let options = RenderOptions::default();
        let result = HtmlFormatter::format(&data, &options).unwrap();

        assert!(result.contains("No data available"));
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(HtmlFormatter::escape_html("Hello & World"), "Hello &amp; World");
        assert_eq!(HtmlFormatter::escape_html("A < B"), "A &lt; B");
        assert_eq!(HtmlFormatter::escape_html("C > D"), "C &gt; D");
        assert_eq!(HtmlFormatter::escape_html("He said \"Hello\""), "He said &quot;Hello&quot;");
        assert_eq!(HtmlFormatter::escape_html("It's a test"), "It&#x27;s a test");
    }

    #[test]
    fn test_format_html_with_custom_class() {
        let data = create_test_data();
        let mut options = RenderOptions::default();
        options.html_table_class = Some("custom-table".to_string());
        
        let result = HtmlFormatter::format(&data, &options).unwrap();

        assert!(result.contains("<table class=\"custom-table\">"));
    }

    #[test]
    fn test_format_html_includes_styles() {
        let data = create_test_data();
        let options = RenderOptions::default();
        let result = HtmlFormatter::format(&data, &options).unwrap();

        assert!(result.contains("<style>"));
        assert!(result.contains(".ttr-table"));
        assert!(result.contains("border-collapse: collapse"));
    }
}
