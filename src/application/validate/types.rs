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
    pub severity: ValidationSeverity,
    pub message: String,
    pub entity_type: Option<String>,
    pub entity_code: Option<String>,
    pub field: Option<String>,
    pub details: Option<String>,
}

impl ValidationResult {
    pub fn info(message: String) -> Self {
        Self {
            severity: ValidationSeverity::Info,
            message,
            entity_type: None,
            entity_code: None,
            field: None,
            details: None,
        }
    }

    pub fn warning(message: String) -> Self {
        Self {
            severity: ValidationSeverity::Warning,
            message,
            entity_type: None,
            entity_code: None,
            field: None,
            details: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            severity: ValidationSeverity::Error,
            message,
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
}

impl std::fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let severity_str = match self.severity {
            ValidationSeverity::Info => "[INFO]",
            ValidationSeverity::Warning => "[WARNING]",
            ValidationSeverity::Error => "[ERROR]",
        };

        let mut output = format!("{} {}", severity_str, self.message);

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
