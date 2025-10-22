use super::types::{OutputFormat, RenderableData, RenderOptions, RenderError, RenderResult};
use super::formatters::{TableFormatter, JsonFormatter, CsvFormatter, HtmlFormatter};

/// Unified renderer for all output formats
pub struct UnifiedRenderer {
    format: OutputFormat,
    options: RenderOptions,
}

impl UnifiedRenderer {
    /// Create a new unified renderer
    pub fn new(format: OutputFormat) -> Self {
        Self {
            format,
            options: RenderOptions::default(),
        }
    }

    /// Create a renderer with custom options
    pub fn with_options(format: OutputFormat, options: RenderOptions) -> Self {
        Self { format, options }
    }

    /// Render data in the specified format
    pub fn render(&self, data: &RenderableData) -> RenderResult<String> {
        match self.format {
            OutputFormat::Table => TableFormatter::format(data, &self.options),
            OutputFormat::Json => JsonFormatter::format(data, &self.options),
            OutputFormat::Csv => CsvFormatter::format(data, &self.options),
            OutputFormat::Html => HtmlFormatter::format(data, &self.options),
        }
    }

    /// Get the current output format
    pub fn format(&self) -> &OutputFormat {
        &self.format
    }

    /// Get the current render options
    pub fn options(&self) -> &RenderOptions {
        &self.options
    }

    /// Update render options
    pub fn with_updated_options<F>(mut self, updater: F) -> Self
    where
        F: FnOnce(&mut RenderOptions),
    {
        updater(&mut self.options);
        self
    }
}

/// Convenience functions for common rendering scenarios
impl UnifiedRenderer {
    /// Render data as a table with default options
    pub fn render_table(data: &RenderableData) -> RenderResult<String> {
        Self::new(OutputFormat::Table).render(data)
    }

    /// Render data as JSON with default options
    pub fn render_json(data: &RenderableData) -> RenderResult<String> {
        Self::new(OutputFormat::Json).render(data)
    }

    /// Render data as CSV with default options
    pub fn render_csv(data: &RenderableData) -> RenderResult<String> {
        Self::new(OutputFormat::Csv).render(data)
    }

    /// Render data as HTML with default options
    pub fn render_html(data: &RenderableData) -> RenderResult<String> {
        Self::new(OutputFormat::Html).render(data)
    }

    /// Render data with custom format and options
    pub fn render_with_format(
        data: &RenderableData,
        format: OutputFormat,
        options: RenderOptions,
    ) -> RenderResult<String> {
        Self::with_options(format, options).render(data)
    }
}

/// Builder pattern for creating renderers with custom options
pub struct RendererBuilder {
    format: OutputFormat,
    options: RenderOptions,
}

impl RendererBuilder {
    /// Start building a renderer
    pub fn new(format: OutputFormat) -> Self {
        Self {
            format,
            options: RenderOptions::default(),
        }
    }

    /// Set CSV include headers option
    pub fn csv_include_headers(mut self, include: bool) -> Self {
        self.options.csv_include_headers = include;
        self
    }

    /// Set JSON pretty print option
    pub fn json_pretty(mut self, pretty: bool) -> Self {
        self.options.json_pretty = pretty;
        self
    }

    /// Set JSON include metadata option
    pub fn json_include_metadata(mut self, include: bool) -> Self {
        self.options.json_include_metadata = include;
        self
    }

    /// Set HTML table class
    pub fn html_table_class(mut self, class: Option<String>) -> Self {
        self.options.html_table_class = class;
        self
    }

    /// Set HTML inline styles option
    pub fn html_inline_styles(mut self, inline: bool) -> Self {
        self.options.html_inline_styles = inline;
        self
    }

    /// Set table max column width
    pub fn table_max_column_width(mut self, width: Option<usize>) -> Self {
        self.options.table_max_column_width = width;
        self
    }

    /// Set table truncate option
    pub fn table_truncate(mut self, truncate: bool) -> Self {
        self.options.table_truncate = truncate;
        self
    }

    /// Build the renderer
    pub fn build(self) -> UnifiedRenderer {
        UnifiedRenderer::with_options(self.format, self.options)
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
    fn test_render_table() {
        let data = create_test_data();
        let result = UnifiedRenderer::render_table(&data).unwrap();
        
        assert!(result.contains("Name"));
        assert!(result.contains("Age"));
        assert!(result.contains("John"));
        assert!(result.contains("Jane"));
    }

    #[test]
    fn test_render_json() {
        let data = create_test_data();
        let result = UnifiedRenderer::render_json(&data).unwrap();
        
        assert!(result.contains("\"Name\""));
        assert!(result.contains("\"Age\""));
        assert!(result.contains("\"John\""));
        assert!(result.contains("\"Jane\""));
    }

    #[test]
    fn test_render_csv() {
        let data = create_test_data();
        let result = UnifiedRenderer::render_csv(&data).unwrap();
        
        assert!(result.contains("Name,Age"));
        assert!(result.contains("John,25"));
        assert!(result.contains("Jane,30"));
    }

    #[test]
    fn test_render_html() {
        let data = create_test_data();
        let result = UnifiedRenderer::render_html(&data).unwrap();
        
        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("<table"));
        assert!(result.contains("<th>Name</th>"));
        assert!(result.contains("<td>John</td>"));
    }

    #[test]
    fn test_renderer_builder() {
        let data = create_test_data();
        let renderer = RendererBuilder::new(OutputFormat::Json)
            .json_pretty(false)
            .json_include_metadata(false)
            .build();
        
        let result = renderer.render(&data).unwrap();
        
        // Compact JSON should not contain newlines
        assert!(!result.contains('\n'));
    }

    #[test]
    fn test_render_with_custom_options() {
        let data = create_test_data();
        let mut options = RenderOptions::default();
        options.json_pretty = false;
        
        let result = UnifiedRenderer::render_with_format(&data, OutputFormat::Json, options).unwrap();
        
        assert!(!result.contains('\n'));
    }

    #[test]
    fn test_render_empty_data() {
        let data = RenderableData::new(vec![], vec![]);
        
        let table_result = UnifiedRenderer::render_table(&data).unwrap();
        assert_eq!(table_result, "No data available");
        
        let csv_result = UnifiedRenderer::render_csv(&data).unwrap();
        assert_eq!(csv_result, "No data available");
        
        let html_result = UnifiedRenderer::render_html(&data).unwrap();
        assert!(html_result.contains("No data available"));
    }

    #[test]
    fn test_render_with_title() {
        let data = create_test_data().with_title("Test Data".to_string());
        
        let table_result = UnifiedRenderer::render_table(&data).unwrap();
        assert!(table_result.contains("Test Data"));
        assert!(table_result.contains("========="));
        
        let json_result = UnifiedRenderer::render_json(&data).unwrap();
        assert!(json_result.contains("\"title\""));
        assert!(json_result.contains("\"Test Data\""));
        
        let html_result = UnifiedRenderer::render_html(&data).unwrap();
        assert!(html_result.contains("<h1>Test Data</h1>"));
    }
}
