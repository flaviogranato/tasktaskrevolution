#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
}

impl std::fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationSeverity::Info => write!(f, "INFO"),
            ValidationSeverity::Warning => write!(f, "WARNING"),
            ValidationSeverity::Error => write!(f, "ERROR"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub code: String,
    pub level: ValidationSeverity,
    pub message: String,
    pub path: Option<String>,
    pub entity_type: Option<String>,
    pub entity_code: Option<String>,
    pub field: Option<String>,
    pub details: Option<String>,
}

impl ValidationResult {
    pub fn info(code: String, message: String) -> Self {
        Self {
            code,
            level: ValidationSeverity::Info,
            message,
            path: None,
            entity_type: None,
            entity_code: None,
            field: None,
            details: None,
        }
    }

    pub fn warning(code: String, message: String) -> Self {
        Self {
            code,
            level: ValidationSeverity::Warning,
            message,
            path: None,
            entity_type: None,
            entity_code: None,
            field: None,
            details: None,
        }
    }

    pub fn error(code: String, message: String) -> Self {
        Self {
            code,
            level: ValidationSeverity::Error,
            message,
            path: None,
            entity_type: None,
            entity_code: None,
            field: None,
            details: None,
        }
    }

    pub fn with_entity(mut self, entity_type: String, entity_code: String) -> Self {
        self.entity_type = Some(entity_type);
        self.entity_code = Some(entity_code);
        self
    }

    pub fn with_field(mut self, field: String) -> Self {
        self.field = Some(field);
        self
    }

    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }
}

impl std::fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level_str = match self.level {
            ValidationSeverity::Info => "[INFO]",
            ValidationSeverity::Warning => "[WARNING]",
            ValidationSeverity::Error => "[ERROR]",
        };

        let mut output = format!("{} [{}] {}", level_str, self.code, self.message);

        if let Some(path) = &self.path {
            output.push_str(&format!(" (Path: {})", path));
        }

        if let (Some(entity_type), Some(entity_code)) = (&self.entity_type, &self.entity_code) {
            output.push_str(&format!(" ({}: {})", entity_type, entity_code));
        }

        if let Some(field) = &self.field {
            output.push_str(&format!(" [Field: {}]", field));
        }

        if let Some(details) = &self.details {
            output.push_str(&format!(" - {}", details));
        }

        write!(f, "{}", output)
    }
}

/// Structured finding for validation results with standardized format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub level: String,
    pub code: String,
    pub message: String,
    pub path: String,
    pub entity_ref: String,
}

impl From<&ValidationResult> for Finding {
    fn from(result: &ValidationResult) -> Self {
        let level = match result.level {
            ValidationSeverity::Info => "info",
            ValidationSeverity::Warning => "warning",
            ValidationSeverity::Error => "error",
        }
        .to_string();

        let path = result.path.clone().unwrap_or_else(|| "N/A".to_string());

        let entity_ref = if let (Some(entity_type), Some(entity_code)) = (&result.entity_type, &result.entity_code) {
            format!("{}:{}", entity_type, entity_code)
        } else {
            "N/A".to_string()
        };

        Self {
            level,
            code: result.code.clone(),
            message: result.message.clone(),
            path,
            entity_ref,
        }
    }
}

/// Supported output formats for validation results
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Json,
    Table,
    Csv,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "table" => Ok(OutputFormat::Table),
            "csv" => Ok(OutputFormat::Csv),
            _ => Err(format!(
                "Unsupported format: {}. Supported formats: json, table, csv",
                s
            )),
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Csv => write!(f, "csv"),
        }
    }
}
