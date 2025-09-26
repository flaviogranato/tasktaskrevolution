use crate::application::errors::AppError;
use crate::application::report::types::{ExportFormat, ReportData};
use serde_json;
use serde_yaml;

/// Trait para formatadores de exportação
pub trait ReportFormatter {
    fn format(&self, data: &ReportData) -> Result<String, AppError>;
    fn format_to_file(&self, data: &ReportData, file_path: &str) -> Result<(), AppError>;
}

/// Formatador JSON
pub struct JsonFormatter;

impl ReportFormatter for JsonFormatter {
    fn format(&self, data: &ReportData) -> Result<String, AppError> {
        serde_json::to_string_pretty(data)
            .map_err(|e| AppError::validation_error("json", format!("Failed to serialize JSON: {}", e)))
    }

    fn format_to_file(&self, data: &ReportData, file_path: &str) -> Result<(), AppError> {
        let json_string = self.format(data)?;
        std::fs::write(file_path, json_string)
            .map_err(|e| AppError::validation_error("file", format!("Failed to write JSON file: {}", e)))?;
        Ok(())
    }
}

/// Formatador CSV
pub struct CsvFormatter;

impl ReportFormatter for CsvFormatter {
    fn format(&self, data: &ReportData) -> Result<String, AppError> {
        if data.data.is_empty() {
            return Ok(String::new());
        }

        // Obter todos os campos únicos dos dados
        let mut all_fields = std::collections::HashSet::new();
        for record in &data.data {
            for field in record.keys() {
                all_fields.insert(field.clone());
            }
        }

        let mut fields: Vec<String> = all_fields.into_iter().collect();
        fields.sort();

        let mut csv = String::new();

        // Cabeçalho
        csv.push_str(&fields.join(","));
        csv.push('\n');

        // Dados
        for record in &data.data {
            let mut row = Vec::new();
            for field in &fields {
                let value = record.get(field).map(format_value_for_csv).unwrap_or_else(String::new);
                row.push(value);
            }
            csv.push_str(&row.join(","));
            csv.push('\n');
        }

        Ok(csv)
    }

    fn format_to_file(&self, data: &ReportData, file_path: &str) -> Result<(), AppError> {
        let csv_string = self.format(data)?;
        std::fs::write(file_path, csv_string)
            .map_err(|e| AppError::validation_error("file", format!("Failed to write CSV file: {}", e)))?;
        Ok(())
    }
}

/// Formatador YAML
pub struct YamlFormatter;

impl ReportFormatter for YamlFormatter {
    fn format(&self, data: &ReportData) -> Result<String, AppError> {
        serde_yaml::to_string(data)
            .map_err(|e| AppError::validation_error("yaml", format!("Failed to serialize YAML: {}", e)))
    }

    fn format_to_file(&self, data: &ReportData, file_path: &str) -> Result<(), AppError> {
        let yaml_string = self.format(data)?;
        std::fs::write(file_path, yaml_string)
            .map_err(|e| AppError::validation_error("file", format!("Failed to write YAML file: {}", e)))?;
        Ok(())
    }
}

/// Formatador PDF (placeholder - implementação básica)
pub struct PdfFormatter;

impl ReportFormatter for PdfFormatter {
    fn format(&self, data: &ReportData) -> Result<String, AppError> {
        // Implementação básica - retorna texto formatado
        let mut pdf_content = String::new();
        pdf_content.push_str(&format!("# {}\n\n", data.title));
        pdf_content.push_str(&format!(
            "Generated at: {}\n",
            data.generated_at.format("%Y-%m-%d %H:%M:%S")
        ));
        pdf_content.push_str(&format!("Total records: {}\n\n", data.total_records));

        if let Some(summary) = &data.summary {
            pdf_content.push_str("## Summary\n\n");
            pdf_content.push_str(&format!("Total count: {}\n", summary.total_count));
            for (field, stats) in &summary.field_stats {
                pdf_content.push_str(&format!(
                    "- {}: {} records, {} unique\n",
                    field, stats.count, stats.unique_count
                ));
            }
            pdf_content.push('\n');
        }

        pdf_content.push_str("## Data\n\n");
        for (i, record) in data.data.iter().enumerate() {
            pdf_content.push_str(&format!("### Record {}\n", i + 1));
            for (field, value) in record {
                pdf_content.push_str(&format!("- {}: {}\n", field, format_value_for_display(value)));
            }
            pdf_content.push('\n');
        }

        Ok(pdf_content)
    }

    fn format_to_file(&self, data: &ReportData, file_path: &str) -> Result<(), AppError> {
        let pdf_content = self.format(data)?;
        std::fs::write(file_path, pdf_content)
            .map_err(|e| AppError::validation_error("file", format!("Failed to write PDF file: {}", e)))?;
        Ok(())
    }
}

/// Factory para criar formatadores
pub struct FormatterFactory;

impl FormatterFactory {
    pub fn create_formatter(format: &ExportFormat) -> Box<dyn ReportFormatter> {
        match format {
            ExportFormat::Json => Box::new(JsonFormatter),
            ExportFormat::Csv => Box::new(CsvFormatter),
            ExportFormat::Yaml => Box::new(YamlFormatter),
            ExportFormat::Pdf => Box::new(PdfFormatter),
        }
    }
}

/// Função auxiliar para formatar valores para CSV
fn format_value_for_csv(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => {
            // Escapar aspas duplas e quebras de linha
            let escaped = s.replace("\"", "\"\"");
            if escaped.contains(',') || escaped.contains('\n') || escaped.contains('"') {
                format!("\"{}\"", escaped)
            } else {
                escaped
            }
        }
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => String::new(),
        _ => value.to_string(),
    }
}

/// Função auxiliar para formatar valores para exibição
fn format_value_for_display(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_value_for_display).collect();
            format!("[{}]", items.join(", "))
        }
        serde_json::Value::Object(obj) => {
            let pairs: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_value_for_display(v)))
                .collect();
            format!("{{{}}}", pairs.join(", "))
        }
    }
}
