use std::collections::HashMap;
use std::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Service for managing code-to-ID mappings for all entity types
#[derive(Debug)]
pub struct CodeMappingService {
    mappings: RwLock<HashMap<String, HashMap<String, String>>>, // entity_type -> code -> id
    file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MappingData {
    entity_type: String,
    code: String,
    id: String,
}

impl CodeMappingService {
    /// Create a new CodeMappingService
    pub fn new(file_path: &str) -> Self {
        let service = Self {
            mappings: RwLock::new(HashMap::new()),
            file_path: file_path.to_string(),
        };
        service.load_mappings();
        service
    }

    /// Load mappings from disk
    fn load_mappings(&self) {
        if !Path::new(&self.file_path).exists() {
            return;
        }

        match fs::read_to_string(&self.file_path) {
            Ok(content) => {
                let mappings: Vec<MappingData> = match serde_json::from_str(&content) {
                    Ok(data) => data,
                    Err(_) => return,
                };

                let mut mapping_map = self.mappings.write().unwrap();
                for mapping in mappings {
                    mapping_map
                        .entry(mapping.entity_type)
                        .or_insert_with(HashMap::new)
                        .insert(mapping.code, mapping.id);
                }
            }
            Err(_) => {}
        }
    }

    /// Save mappings to disk
    fn save_mappings(&self) -> Result<(), String> {
        let mappings = self.mappings.read().unwrap();
        let mut mapping_data = Vec::new();

        for (entity_type, code_map) in mappings.iter() {
            for (code, id) in code_map.iter() {
                mapping_data.push(MappingData {
                    entity_type: entity_type.clone(),
                    code: code.clone(),
                    id: id.clone(),
                });
            }
        }

        let content = serde_json::to_string_pretty(&mapping_data)
            .map_err(|e| format!("Failed to serialize mappings: {}", e))?;

        fs::write(&self.file_path, content)
            .map_err(|e| format!("Failed to write mappings file: {}", e))?;

        Ok(())
    }

    /// Add a code-to-ID mapping
    pub fn add_mapping(&self, entity_type: &str, code: &str, id: &str) -> Result<(), String> {
        let mut mappings = self.mappings.write().unwrap();
        mappings
            .entry(entity_type.to_string())
            .or_insert_with(HashMap::new)
            .insert(code.to_string(), id.to_string());
        
        drop(mappings);
        self.save_mappings()
    }

    /// Get ID for a given code
    pub fn get_id(&self, entity_type: &str, code: &str) -> Option<String> {
        let mappings = self.mappings.read().unwrap();
        mappings
            .get(entity_type)
            .and_then(|code_map| code_map.get(code))
            .cloned()
    }

    /// Get code for a given ID
    pub fn get_code(&self, entity_type: &str, id: &str) -> Option<String> {
        let mappings = self.mappings.read().unwrap();
        mappings
            .get(entity_type)
            .and_then(|code_map| {
                code_map.iter()
                    .find(|(_, mapping_id)| **mapping_id == id)
                    .map(|(code, _)| code.clone())
            })
    }

    /// Update a code mapping (when code changes)
    pub fn update_code(&self, entity_type: &str, old_code: &str, new_code: &str) -> Result<(), String> {
        let mut mappings = self.mappings.write().unwrap();
        
        if let Some(code_map) = mappings.get_mut(entity_type) {
            if let Some(id) = code_map.remove(old_code) {
                code_map.insert(new_code.to_string(), id);
            }
        }
        
        drop(mappings);
        self.save_mappings()
    }

    /// Remove a mapping
    pub fn remove_mapping(&self, entity_type: &str, code: &str) -> Result<(), String> {
        let mut mappings = self.mappings.write().unwrap();
        
        if let Some(code_map) = mappings.get_mut(entity_type) {
            code_map.remove(code);
        }
        
        drop(mappings);
        self.save_mappings()
    }

    /// Get all codes for an entity type
    pub fn get_all_codes(&self, entity_type: &str) -> Vec<String> {
        let mappings = self.mappings.read().unwrap();
        mappings
            .get(entity_type)
            .map(|code_map| code_map.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all IDs for an entity type
    pub fn get_all_ids(&self, entity_type: &str) -> Vec<String> {
        let mappings = self.mappings.read().unwrap();
        mappings
            .get(entity_type)
            .map(|code_map| code_map.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Check if a code exists
    pub fn code_exists(&self, entity_type: &str, code: &str) -> bool {
        let mappings = self.mappings.read().unwrap();
        mappings
            .get(entity_type)
            .map(|code_map| code_map.contains_key(code))
            .unwrap_or(false)
    }

    /// Check if an ID exists
    pub fn id_exists(&self, entity_type: &str, id: &str) -> bool {
        let mappings = self.mappings.read().unwrap();
        mappings
            .get(entity_type)
            .map(|code_map| code_map.values().any(|mapping_id| mapping_id == id))
            .unwrap_or(false)
    }

    /// Get mapping count for an entity type
    pub fn get_mapping_count(&self, entity_type: &str) -> usize {
        let mappings = self.mappings.read().unwrap();
        mappings
            .get(entity_type)
            .map(|code_map| code_map.len())
            .unwrap_or(0)
    }

    /// Clear all mappings for an entity type
    pub fn clear_entity_mappings(&self, entity_type: &str) -> Result<(), String> {
        let mut mappings = self.mappings.write().unwrap();
        mappings.remove(entity_type);
        drop(mappings);
        self.save_mappings()
    }

    /// Clear all mappings
    pub fn clear_all_mappings(&self) -> Result<(), String> {
        let mut mappings = self.mappings.write().unwrap();
        mappings.clear();
        drop(mappings);
        self.save_mappings()
    }
}

impl Default for CodeMappingService {
    fn default() -> Self {
        Self::new(".ttr/mappings.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_add_and_get_mapping() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("mappings.json");
        let service = CodeMappingService::new(file_path.to_str().unwrap());

        service.add_mapping("company", "TECH-001", "id-123").unwrap();
        
        assert_eq!(service.get_id("company", "TECH-001"), Some("id-123".to_string()));
        assert_eq!(service.get_code("company", "id-123"), Some("TECH-001".to_string()));
    }

    #[test]
    fn test_update_code() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("mappings.json");
        let service = CodeMappingService::new(file_path.to_str().unwrap());

        service.add_mapping("company", "TECH-001", "id-123").unwrap();
        service.update_code("company", "TECH-001", "TECH-002").unwrap();
        
        assert_eq!(service.get_id("company", "TECH-002"), Some("id-123".to_string()));
        assert_eq!(service.get_id("company", "TECH-001"), None);
    }

    #[test]
    fn test_remove_mapping() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("mappings.json");
        let service = CodeMappingService::new(file_path.to_str().unwrap());

        service.add_mapping("company", "TECH-001", "id-123").unwrap();
        service.remove_mapping("company", "TECH-001").unwrap();
        
        assert_eq!(service.get_id("company", "TECH-001"), None);
    }

    #[test]
    fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("mappings.json");
        
        // Create service and add mapping
        let service1 = CodeMappingService::new(file_path.to_str().unwrap());
        service1.add_mapping("company", "TECH-001", "id-123").unwrap();
        
        // Create new service instance and verify mapping is loaded
        let service2 = CodeMappingService::new(file_path.to_str().unwrap());
        assert_eq!(service2.get_id("company", "TECH-001"), Some("id-123".to_string()));
    }

    #[test]
    fn test_multiple_entity_types() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("mappings.json");
        let service = CodeMappingService::new(file_path.to_str().unwrap());

        service.add_mapping("company", "TECH-001", "company-id-123").unwrap();
        service.add_mapping("project", "PROJ-001", "project-id-456").unwrap();
        
        assert_eq!(service.get_id("company", "TECH-001"), Some("company-id-123".to_string()));
        assert_eq!(service.get_id("project", "PROJ-001"), Some("project-id-456".to_string()));
    }
}
