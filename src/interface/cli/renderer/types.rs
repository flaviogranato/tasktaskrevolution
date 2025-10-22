use crate::domain::shared::query_parser::ProjectionOptions;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported output formats for the unified renderer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
    Html,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "table" => Ok(OutputFormat::Table),
            "json" => Ok(OutputFormat::Json),
            "csv" => Ok(OutputFormat::Csv),
            "html" => Ok(OutputFormat::Html),
            _ => Err(format!(
                "Unsupported format: {}. Supported formats: table, json, csv, html",
                s
            )),
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Csv => write!(f, "csv"),
            OutputFormat::Html => write!(f, "html"),
        }
    }
}

/// Structured data that can be rendered in any format
#[derive(Debug, Clone)]
pub struct RenderableData {
    /// Column headers
    pub headers: Vec<String>,
    /// Data rows
    pub rows: Vec<Vec<String>>,
    /// Optional metadata for JSON/HTML output
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Projection options for field selection
    pub projection: Option<ProjectionOptions>,
    /// Optional title for the data
    pub title: Option<String>,
}

impl RenderableData {
    /// Create new renderable data with headers and rows
    pub fn new(headers: Vec<String>, rows: Vec<Vec<String>>) -> Self {
        Self {
            headers,
            rows,
            metadata: None,
            projection: None,
            title: None,
        }
    }

    /// Create renderable data with metadata
    pub fn with_metadata(
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            headers,
            rows,
            metadata: Some(metadata),
            projection: None,
            title: None,
        }
    }

    /// Add a title to the data
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    /// Add metadata to the data
    pub fn with_metadata_map(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Add projection options to the data
    pub fn with_projection(mut self, projection: ProjectionOptions) -> Self {
        self.projection = Some(projection);
        self
    }

    /// Apply projection to filter headers and rows based on projection options
    pub fn apply_projection(mut self) -> Self {
        if let Some(projection) = &self.projection {
            if !projection.include_all && !projection.fields.is_empty() {
                // Get the field names from projection
                let projected_fields: Vec<String> = projection.fields.iter()
                    .map(|field| field.field.clone())
                    .collect();

                // Filter headers to only include projected fields
                let mut new_headers = Vec::new();
                let mut header_indices = Vec::new();
                
                for (i, header) in self.headers.iter().enumerate() {
                    if projected_fields.contains(header) {
                        new_headers.push(header.clone());
                        header_indices.push(i);
                    }
                }

                // Filter rows to only include projected columns
                let mut new_rows = Vec::new();
                for row in &self.rows {
                    let mut new_row = Vec::new();
                    for &index in &header_indices {
                        if index < row.len() {
                            new_row.push(row[index].clone());
                        }
                    }
                    new_rows.push(new_row);
                }

                self.headers = new_headers;
                self.rows = new_rows;
            }
        }
        self
    }

    /// Check if data is empty
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Get the number of rows
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Get the number of columns
    pub fn column_count(&self) -> usize {
        self.headers.len()
    }
}

/// Rendering options for different formats
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Whether to include headers in CSV output
    pub csv_include_headers: bool,
    /// Whether to pretty-print JSON
    pub json_pretty: bool,
    /// Whether to include metadata in JSON output
    pub json_include_metadata: bool,
    /// CSS classes for HTML table
    pub html_table_class: Option<String>,
    /// Whether to include inline styles in HTML
    pub html_inline_styles: bool,
    /// Maximum column width for table format
    pub table_max_column_width: Option<usize>,
    /// Whether to truncate long content
    pub table_truncate: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            csv_include_headers: true,
            json_pretty: true,
            json_include_metadata: true,
            html_table_class: Some("ttr-table".to_string()),
            html_inline_styles: true,
            table_max_column_width: Some(50),
            table_truncate: true,
        }
    }
}

/// Errors that can occur during rendering
#[derive(Debug)]
pub enum RenderError {
    Serialization(serde_json::Error),
    Io(std::io::Error),
    InvalidData(String),
    UnsupportedFormat(String),
    Rendering(String),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::Serialization(e) => write!(f, "Serialization error: {}", e),
            RenderError::Io(e) => write!(f, "IO error: {}", e),
            RenderError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            RenderError::UnsupportedFormat(format) => write!(f, "Unsupported format: {}", format),
            RenderError::Rendering(msg) => write!(f, "Rendering error: {}", msg),
        }
    }
}

impl std::error::Error for RenderError {}

impl From<serde_json::Error> for RenderError {
    fn from(err: serde_json::Error) -> Self {
        RenderError::Serialization(err)
    }
}

impl From<std::io::Error> for RenderError {
    fn from(err: std::io::Error) -> Self {
        RenderError::Io(err)
    }
}

impl From<std::fmt::Error> for RenderError {
    fn from(err: std::fmt::Error) -> Self {
        RenderError::Rendering(format!("Format error: {}", err))
    }
}

/// Result type for rendering operations
pub type RenderResult<T> = Result<T, RenderError>;
