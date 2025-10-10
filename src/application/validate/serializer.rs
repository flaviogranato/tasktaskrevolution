use super::types::{Finding, OutputFormat, ValidationResult};
use serde_json;
use std::fmt::Write;

/// Serializes validation results to different output formats
pub struct ValidationSerializer;

impl ValidationSerializer {
    /// Convert ValidationResults to Findings
    pub fn to_findings(results: &[ValidationResult]) -> Vec<Finding> {
        results.iter().map(Finding::from).collect()
    }

    /// Serialize findings to JSON format
    pub fn to_json(findings: &[Finding]) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string_pretty(findings)?)
    }

    /// Serialize findings to table format
    pub fn to_table(findings: &[Finding]) -> String {
        if findings.is_empty() {
            return "No validation issues found".to_string();
        }

        let mut output = String::new();

        // Table header
        writeln!(
            output,
            "{:<8} {:<12} {:<50} {:<20} {:<20}",
            "LEVEL", "CODE", "MESSAGE", "PATH", "ENTITY_REF"
        )
        .unwrap();
        writeln!(output, "{:-<8} {:-<12} {:-<50} {:-<20} {:-<20}", "", "", "", "", "").unwrap();

        // Table rows
        for finding in findings {
            let message = if finding.message.len() > 47 {
                format!("{}...", &finding.message[..47])
            } else {
                finding.message.clone()
            };

            let path = if finding.path.len() > 17 {
                format!("{}...", &finding.path[..17])
            } else {
                finding.path.clone()
            };

            let entity_ref = if finding.entity_ref.len() > 17 {
                format!("{}...", &finding.entity_ref[..17])
            } else {
                finding.entity_ref.clone()
            };

            writeln!(
                output,
                "{:<8} {:<12} {:<50} {:<20} {:<20}",
                finding.level.to_uppercase(),
                finding.code,
                message,
                path,
                entity_ref
            )
            .unwrap();
        }

        output
    }

    /// Serialize findings to CSV format
    pub fn to_csv(findings: &[Finding]) -> String {
        if findings.is_empty() {
            return "level,code,message,path,entity_ref\nNo validation issues found".to_string();
        }

        let mut output = String::new();

        // CSV header
        output.push_str("level,code,message,path,entity_ref\n");

        // CSV rows
        for finding in findings {
            let message = finding.message.replace('"', "\"\"");
            let path = finding.path.replace('"', "\"\"");
            let entity_ref = finding.entity_ref.replace('"', "\"\"");

            writeln!(
                output,
                "{},{},\"{}\",\"{}\",\"{}\"",
                finding.level, finding.code, message, path, entity_ref
            )
            .unwrap();
        }

        output
    }

    /// Serialize validation results to the specified format
    pub fn serialize(results: &[ValidationResult], format: OutputFormat) -> Result<String, Box<dyn std::error::Error>> {
        let findings = Self::to_findings(results);

        match format {
            OutputFormat::Json => Self::to_json(&findings),
            OutputFormat::Table => Ok(Self::to_table(&findings)),
            OutputFormat::Csv => Ok(Self::to_csv(&findings)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::validate::types::ValidationResult;

    fn create_test_finding() -> Finding {
        Finding {
            level: "error".to_string(),
            code: "TEST_ERROR".to_string(),
            message: "Test error message".to_string(),
            path: "test/path.yaml".to_string(),
            entity_ref: "Project:PROJ-001".to_string(),
        }
    }

    fn create_test_validation_result() -> ValidationResult {
        ValidationResult::error("TEST_ERROR".to_string(), "Test error message".to_string())
            .with_entity("Project".to_string(), "PROJ-001".to_string())
            .with_path("test/path.yaml".to_string())
    }

    #[test]
    fn test_validation_result_to_finding_conversion() {
        let result = create_test_validation_result();
        let finding = Finding::from(&result);

        assert_eq!(finding.level, "error");
        assert_eq!(finding.code, "TEST_ERROR");
        assert_eq!(finding.message, "Test error message");
        assert_eq!(finding.path, "test/path.yaml");
        assert_eq!(finding.entity_ref, "Project:PROJ-001");
    }

    #[test]
    fn test_validation_result_to_finding_with_missing_fields() {
        let result = ValidationResult::error("TEST_ERROR".to_string(), "Test error message".to_string());
        let finding = Finding::from(&result);

        assert_eq!(finding.level, "error");
        assert_eq!(finding.code, "TEST_ERROR");
        assert_eq!(finding.message, "Test error message");
        assert_eq!(finding.path, "N/A");
        assert_eq!(finding.entity_ref, "N/A");
    }

    #[test]
    fn test_to_findings() {
        let results = vec![create_test_validation_result()];
        let findings = ValidationSerializer::to_findings(&results);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].level, "error");
        assert_eq!(findings[0].code, "TEST_ERROR");
    }

    #[test]
    fn test_to_json() {
        let findings = vec![create_test_finding()];
        let json = ValidationSerializer::to_json(&findings).unwrap();

        assert!(json.contains("\"level\": \"error\""));
        assert!(json.contains("\"code\": \"TEST_ERROR\""));
        assert!(json.contains("\"message\": \"Test error message\""));
    }

    #[test]
    fn test_to_table() {
        let findings = vec![create_test_finding()];
        let table = ValidationSerializer::to_table(&findings);

        assert!(table.contains("LEVEL"));
        assert!(table.contains("CODE"));
        assert!(table.contains("MESSAGE"));
        assert!(table.contains("PATH"));
        assert!(table.contains("ENTITY_REF"));
        assert!(table.contains("ERROR"));
        assert!(table.contains("TEST_ERROR"));
    }

    #[test]
    fn test_to_table_empty() {
        let findings = vec![];
        let table = ValidationSerializer::to_table(&findings);

        assert_eq!(table, "No validation issues found");
    }

    #[test]
    fn test_to_csv() {
        let findings = vec![create_test_finding()];
        let csv = ValidationSerializer::to_csv(&findings);

        assert!(csv.contains("level,code,message,path,entity_ref"));
        assert!(csv.contains("error,TEST_ERROR,\"Test error message\",\"test/path.yaml\",\"Project:PROJ-001\""));
    }

    #[test]
    fn test_to_csv_empty() {
        let findings = vec![];
        let csv = ValidationSerializer::to_csv(&findings);

        assert!(csv.contains("level,code,message,path,entity_ref"));
        assert!(csv.contains("No validation issues found"));
    }

    #[test]
    fn test_serialize_json() {
        let results = vec![create_test_validation_result()];
        let output = ValidationSerializer::serialize(&results, OutputFormat::Json).unwrap();

        assert!(output.contains("\"level\": \"error\""));
        assert!(output.contains("\"code\": \"TEST_ERROR\""));
    }

    #[test]
    fn test_serialize_table() {
        let results = vec![create_test_validation_result()];
        let output = ValidationSerializer::serialize(&results, OutputFormat::Table).unwrap();

        assert!(output.contains("LEVEL"));
        assert!(output.contains("ERROR"));
        assert!(output.contains("TEST_ERROR"));
    }

    #[test]
    fn test_serialize_csv() {
        let results = vec![create_test_validation_result()];
        let output = ValidationSerializer::serialize(&results, OutputFormat::Csv).unwrap();

        assert!(output.contains("level,code,message,path,entity_ref"));
        assert!(output.contains("error,TEST_ERROR"));
    }

    #[test]
    fn test_output_format_from_str() {
        assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert_eq!("table".parse::<OutputFormat>().unwrap(), OutputFormat::Table);
        assert_eq!("csv".parse::<OutputFormat>().unwrap(), OutputFormat::Csv);
        assert_eq!("JSON".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert_eq!("TABLE".parse::<OutputFormat>().unwrap(), OutputFormat::Table);
        assert_eq!("CSV".parse::<OutputFormat>().unwrap(), OutputFormat::Csv);
    }

    #[test]
    fn test_output_format_from_str_invalid() {
        assert!("invalid".parse::<OutputFormat>().is_err());
        assert!("xml".parse::<OutputFormat>().is_err());
    }

    #[test]
    fn test_output_format_display() {
        assert_eq!(format!("{}", OutputFormat::Json), "json");
        assert_eq!(format!("{}", OutputFormat::Table), "table");
        assert_eq!(format!("{}", OutputFormat::Csv), "csv");
    }
}
