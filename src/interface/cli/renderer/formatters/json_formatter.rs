use super::super::types::{RenderableData, RenderOptions, RenderError, RenderResult};
use serde_json::{json, Value};

/// JSON formatter for structured data
pub struct JsonFormatter;

impl JsonFormatter {
    /// Format data as JSON
    pub fn format(data: &RenderableData, options: &RenderOptions) -> RenderResult<String> {
        let json_data = Self::build_json_structure(data, options)?;
        
        if options.json_pretty {
            serde_json::to_string_pretty(&json_data).map_err(|e| RenderError::Serialization(e))
        } else {
            serde_json::to_string(&json_data).map_err(|e| RenderError::Serialization(e))
        }
    }

    /// Build JSON structure from renderable data
    fn build_json_structure(data: &RenderableData, options: &RenderOptions) -> RenderResult<Value> {
        let mut json_data = json!({
            "data": Self::build_data_array(data),
            "metadata": Self::build_metadata(data, options)
        });

        // Add title if present
        if let Some(title) = &data.title {
            json_data["title"] = json!(title);
        }

        // Add projection information if present
        if let Some(projection) = &data.projection {
            let projection_info = json!({
                "include_all": projection.include_all,
                "fields": projection.fields.iter().map(|field| {
                    json!({
                        "field": field.field,
                        "alias": field.alias
                    })
                }).collect::<Vec<_>>()
            });
            json_data["projection"] = projection_info;
        }

        Ok(json_data)
    }

    /// Build data array from rows
    fn build_data_array(data: &RenderableData) -> Value {
        if data.is_empty() {
            return json!([]);
        }

        let mut array = Vec::new();
        
        for row in &data.rows {
            let mut obj = serde_json::Map::new();
            for (i, cell) in row.iter().enumerate() {
                if i < data.headers.len() {
                    let header = &data.headers[i];
                    obj.insert(header.clone(), json!(cell));
                }
            }
            array.push(Value::Object(obj));
        }

        json!(array)
    }

    /// Build metadata object
    fn build_metadata(data: &RenderableData, options: &RenderOptions) -> Value {
        let mut metadata = json!({
            "row_count": data.row_count(),
            "column_count": data.column_count(),
            "headers": data.headers,
            "format": "json"
        });

        // Add custom metadata if present
        if let Some(custom_metadata) = &data.metadata {
            for (key, value) in custom_metadata {
                metadata[key] = value.clone();
            }
        }

        metadata
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
    fn test_format_basic_json() {
        let data = create_test_data();
        let options = RenderOptions::default();
        let result = JsonFormatter::format(&data, &options).unwrap();

        assert!(result.contains("\"Name\""));
        assert!(result.contains("\"Age\""));
        assert!(result.contains("\"John\""));
        assert!(result.contains("\"Jane\""));
        assert!(result.contains("\"row_count\""));
        assert!(result.contains("\"column_count\""));
    }

    #[test]
    fn test_format_json_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), json!("test"));
        metadata.insert("version".to_string(), json!("1.0"));

        let data = create_test_data().with_metadata_map(metadata);
        let options = RenderOptions::default();
        let result = JsonFormatter::format(&data, &options).unwrap();

        assert!(result.contains("\"source\""));
        assert!(result.contains("\"test\""));
        assert!(result.contains("\"version\""));
        assert!(result.contains("\"1.0\""));
    }

    #[test]
    fn test_format_json_with_title() {
        let data = create_test_data().with_title("Test Data".to_string());
        let options = RenderOptions::default();
        let result = JsonFormatter::format(&data, &options).unwrap();

        assert!(result.contains("\"title\""));
        assert!(result.contains("\"Test Data\""));
    }

    #[test]
    fn test_format_empty_data() {
        let data = RenderableData::new(vec![], vec![]);
        let options = RenderOptions::default();
        let result = JsonFormatter::format(&data, &options).unwrap();

        assert!(result.contains("\"data\": []"));
        assert!(result.contains("\"row_count\": 0"));
    }

    #[test]
    fn test_format_compact_json() {
        let data = create_test_data();
        let mut options = RenderOptions::default();
        options.json_pretty = false;
        
        let result = JsonFormatter::format(&data, &options).unwrap();
        
        // Compact JSON should not contain newlines
        assert!(!result.contains('\n'));
    }
}
