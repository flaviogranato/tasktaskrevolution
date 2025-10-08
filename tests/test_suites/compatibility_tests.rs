//! Compatibility Test Suite
//! 
//! This module contains compatibility tests for the TTR system.

use std::time::{Duration, Instant};
use tempfile::TempDir;
use assert_cmd::Command;
use assert_fs::prelude::*;

/// Compatibility test configuration
#[derive(Debug, Clone)]
pub struct CompatibilityTestConfig {
    pub test_categories: Vec<CompatibilityTestCategory>,
    pub target_versions: Vec<String>,
    pub data_formats: Vec<DataFormat>,
    pub operating_systems: Vec<OperatingSystem>,
    pub rust_versions: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum CompatibilityTestCategory {
    DataMigration,
    FileFormat,
    CLIInterface,
    APICompatibility,
    DatabaseSchema,
    Configuration,
    Dependencies,
    Performance,
}

#[derive(Debug, Clone)]
pub enum DataFormat {
    YAML,
    JSON,
    CSV,
    HTML,
    XML,
}

#[derive(Debug, Clone)]
pub enum OperatingSystem {
    Linux,
    Windows,
    macOS,
    FreeBSD,
    OpenBSD,
}

impl CompatibilityTestConfig {
    pub fn new() -> Self {
        Self {
            test_categories: vec![
                CompatibilityTestCategory::DataMigration,
                CompatibilityTestCategory::FileFormat,
                CompatibilityTestCategory::CLIInterface,
            ],
            target_versions: vec!["1.0.0".to_string(), "1.1.0".to_string()],
            data_formats: vec![DataFormat::YAML, DataFormat::JSON],
            operating_systems: vec![OperatingSystem::Linux, OperatingSystem::Windows, OperatingSystem::macOS],
            rust_versions: vec!["1.90".to_string(), "stable".to_string()],
        }
    }
    
    pub fn with_categories(mut self, categories: Vec<CompatibilityTestCategory>) -> Self {
        self.test_categories = categories;
        self
    }
    
    pub fn with_target_versions(mut self, versions: Vec<String>) -> Self {
        self.target_versions = versions;
        self
    }
    
    pub fn with_data_formats(mut self, formats: Vec<DataFormat>) -> Self {
        self.data_formats = formats;
        self
    }
    
    pub fn with_operating_systems(mut self, os: Vec<OperatingSystem>) -> Self {
        self.operating_systems = os;
        self
    }
}

/// Compatibility test results
#[derive(Debug, Clone)]
pub struct CompatibilityTestResult {
    pub test_name: String,
    pub category: CompatibilityTestCategory,
    pub target_version: String,
    pub compatibility_status: CompatibilityStatus,
    pub migration_required: bool,
    pub breaking_changes: Vec<String>,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
    pub status: TestStatus,
}

#[derive(Debug, Clone)]
pub enum CompatibilityStatus {
    FullyCompatible,
    PartiallyCompatible,
    Incompatible,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum TestStatus {
    Passed,
    Failed,
    Warning,
    Skipped,
}

/// Compatibility test suite
pub struct CompatibilityTestSuite {
    config: CompatibilityTestConfig,
    temp_dir: TempDir,
}

impl CompatibilityTestSuite {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = CompatibilityTestConfig::new();
        let temp_dir = TempDir::new()?;
        
        Ok(Self {
            config,
            temp_dir,
        })
    }
    
    pub fn with_config(mut self, config: CompatibilityTestConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Test data migration compatibility
    pub fn test_data_migration_compatibility(&self) -> CompatibilityTestResult {
        let test_name = "Data Migration Compatibility";
        let mut breaking_changes = Vec::new();
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();
        
        // Test migration from old format to new format
        let old_format_data = r#"
apiVersion: v1
kind: Company
metadata:
  name: "Old Company"
  code: "OLD-COMP"
spec:
  description: "Old company description"
"#;
        
        // Write old format data
        let old_file = self.temp_dir.path().join("old_company.yaml");
        std::fs::write(&old_file, old_format_data).unwrap();
        
        // Test if TTR can read old format
        let mut cmd = Command::cargo_bin("ttr").unwrap();
        cmd.current_dir(self.temp_dir.path());
        
        let result = cmd.args(&["list", "companies"]).assert();
        
        if result.is_ok() {
            // Check if data was migrated properly
            let new_file = self.temp_dir.path().join("companies").join("OLD-COMP").join("company.yaml");
            if new_file.exists() {
                if let Ok(content) = std::fs::read_to_string(&new_file) {
                    if !content.contains("apiVersion") {
                        breaking_changes.push("API version field missing in migrated data".to_string());
                    }
                    if !content.contains("metadata") {
                        breaking_changes.push("Metadata field missing in migrated data".to_string());
                    }
                }
            } else {
                warnings.push("Migrated file not found".to_string());
            }
        } else {
            breaking_changes.push("Failed to read old format data".to_string());
        }
        
        let compatibility_status = if breaking_changes.is_empty() {
            if warnings.is_empty() {
                CompatibilityStatus::FullyCompatible
            } else {
                CompatibilityStatus::PartiallyCompatible
            }
        } else {
            CompatibilityStatus::Incompatible
        };
        
        CompatibilityTestResult {
            test_name: test_name.to_string(),
            category: CompatibilityTestCategory::DataMigration,
            target_version: "current".to_string(),
            compatibility_status,
            migration_required: !breaking_changes.is_empty(),
            breaking_changes,
            warnings,
            recommendations: vec!["Ensure proper data migration scripts are in place".to_string()],
            status: if breaking_changes.is_empty() {
                TestStatus::Passed
            } else {
                TestStatus::Failed
            },
        }
    }
    
    /// Test file format compatibility
    pub fn test_file_format_compatibility(&self) -> CompatibilityTestResult {
        let test_name = "File Format Compatibility";
        let mut breaking_changes = Vec::new();
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();
        
        // Test different file formats
        for format in &self.config.data_formats {
            match format {
                DataFormat::YAML => {
                    // Test YAML format
                    let yaml_data = r#"
apiVersion: v1
kind: Company
metadata:
  name: "Test Company"
  code: "TEST"
spec:
  description: "Test company"
"#;
                    
                    let yaml_file = self.temp_dir.path().join("test_company.yaml");
                    std::fs::write(&yaml_file, yaml_data).unwrap();
                    
                    // Test if TTR can read YAML
                    let mut cmd = Command::cargo_bin("ttr").unwrap();
                    cmd.current_dir(self.temp_dir.path());
                    
                    let result = cmd.args(&["list", "companies"]).assert();
                    if result.is_err() {
                        breaking_changes.push("YAML format not supported".to_string());
                    }
                }
                DataFormat::JSON => {
                    // Test JSON format
                    let json_data = r#"{
  "apiVersion": "v1",
  "kind": "Company",
  "metadata": {
    "name": "Test Company",
    "code": "TEST"
  },
  "spec": {
    "description": "Test company"
  }
}"#;
                    
                    let json_file = self.temp_dir.path().join("test_company.json");
                    std::fs::write(&json_file, json_data).unwrap();
                    
                    // Test if TTR can read JSON
                    let mut cmd = Command::cargo_bin("ttr").unwrap();
                    cmd.current_dir(self.temp_dir.path());
                    
                    let result = cmd.args(&["list", "companies"]).assert();
                    if result.is_err() {
                        warnings.push("JSON format not supported".to_string());
                    }
                }
                _ => {
                    warnings.push(format!("Format {:?} not tested", format));
                }
            }
        }
        
        let compatibility_status = if breaking_changes.is_empty() {
            if warnings.is_empty() {
                CompatibilityStatus::FullyCompatible
            } else {
                CompatibilityStatus::PartiallyCompatible
            }
        } else {
            CompatibilityStatus::Incompatible
        };
        
        CompatibilityTestResult {
            test_name: test_name.to_string(),
            category: CompatibilityTestCategory::FileFormat,
            target_version: "current".to_string(),
            compatibility_status,
            migration_required: !breaking_changes.is_empty(),
            breaking_changes,
            warnings,
            recommendations: vec!["Ensure all required file formats are supported".to_string()],
            status: if breaking_changes.is_empty() {
                TestStatus::Passed
            } else {
                TestStatus::Failed
            },
        }
    }
    
    /// Test CLI interface compatibility
    pub fn test_cli_interface_compatibility(&self) -> CompatibilityTestResult {
        let test_name = "CLI Interface Compatibility";
        let mut breaking_changes = Vec::new();
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();
        
        // Test CLI commands
        let cli_commands = vec![
            ("--help", "Help command"),
            ("--version", "Version command"),
            ("init", "Init command"),
            ("create company", "Create company command"),
            ("list companies", "List companies command"),
            ("validate", "Validate command"),
            ("build", "Build command"),
        ];
        
        for (command, description) in cli_commands {
            let mut cmd = Command::cargo_bin("ttr").unwrap();
            cmd.current_dir(self.temp_dir.path());
            
            let result = cmd.args(command.split_whitespace()).assert();
            if result.is_err() {
                breaking_changes.push(format!("{} not working: {}", description, command));
            }
        }
        
        // Test CLI aliases
        let aliases = vec![
            ("c", "create"),
            ("l", "list"),
            ("v", "validate"),
            ("b", "build"),
        ];
        
        for (alias, full_command) in aliases {
            let mut cmd = Command::cargo_bin("ttr").unwrap();
            cmd.current_dir(self.temp_dir.path());
            
            let result = cmd.args(&[alias, "--help"]).assert();
            if result.is_err() {
                warnings.push(format!("Alias '{}' for '{}' not working", alias, full_command));
            }
        }
        
        let compatibility_status = if breaking_changes.is_empty() {
            if warnings.is_empty() {
                CompatibilityStatus::FullyCompatible
            } else {
                CompatibilityStatus::PartiallyCompatible
            }
        } else {
            CompatibilityStatus::Incompatible
        };
        
        CompatibilityTestResult {
            test_name: test_name.to_string(),
            category: CompatibilityTestCategory::CLIInterface,
            target_version: "current".to_string(),
            compatibility_status,
            migration_required: !breaking_changes.is_empty(),
            breaking_changes,
            warnings,
            recommendations: vec!["Ensure all CLI commands and aliases are working".to_string()],
            status: if breaking_changes.is_empty() {
                TestStatus::Passed
            } else {
                TestStatus::Failed
            },
        }
    }
    
    /// Test API compatibility
    pub fn test_api_compatibility(&self) -> CompatibilityTestResult {
        let test_name = "API Compatibility";
        let mut breaking_changes = Vec::new();
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();
        
        // Test API versioning
        let api_versions = vec!["v1", "v2", "v3"];
        
        for version in api_versions {
            let api_data = format!(r#"
apiVersion: {}
kind: Company
metadata:
  name: "Test Company"
  code: "TEST"
spec:
  description: "Test company"
"#, version);
            
            let api_file = self.temp_dir.path().join(format!("test_company_{}.yaml", version));
            std::fs::write(&api_file, api_data).unwrap();
            
            // Test if TTR can read the API version
            let mut cmd = Command::cargo_bin("ttr").unwrap();
            cmd.current_dir(self.temp_dir.path());
            
            let result = cmd.args(&["list", "companies"]).assert();
            if result.is_err() {
                breaking_changes.push(format!("API version {} not supported", version));
            }
        }
        
        let compatibility_status = if breaking_changes.is_empty() {
            if warnings.is_empty() {
                CompatibilityStatus::FullyCompatible
            } else {
                CompatibilityStatus::PartiallyCompatible
            }
        } else {
            CompatibilityStatus::Incompatible
        };
        
        CompatibilityTestResult {
            test_name: test_name.to_string(),
            category: CompatibilityTestCategory::APICompatibility,
            target_version: "current".to_string(),
            compatibility_status,
            migration_required: !breaking_changes.is_empty(),
            breaking_changes,
            warnings,
            recommendations: vec!["Ensure all API versions are supported".to_string()],
            status: if breaking_changes.is_empty() {
                TestStatus::Passed
            } else {
                TestStatus::Failed
            },
        }
    }
    
    /// Run all compatibility tests
    pub fn run_all_tests(&self) -> Vec<CompatibilityTestResult> {
        let mut results = Vec::new();
        
        for category in &self.config.test_categories {
            let result = match category {
                CompatibilityTestCategory::DataMigration => {
                    self.test_data_migration_compatibility()
                }
                CompatibilityTestCategory::FileFormat => {
                    self.test_file_format_compatibility()
                }
                CompatibilityTestCategory::CLIInterface => {
                    self.test_cli_interface_compatibility()
                }
                CompatibilityTestCategory::APICompatibility => {
                    self.test_api_compatibility()
                }
                _ => {
                    // Other categories would be implemented here
                    CompatibilityTestResult {
                        test_name: format!("{:?} Compatibility", category),
                        category: category.clone(),
                        target_version: "current".to_string(),
                        compatibility_status: CompatibilityStatus::Unknown,
                        migration_required: false,
                        breaking_changes: vec![],
                        warnings: vec![],
                        recommendations: vec![],
                        status: TestStatus::Skipped,
                    }
                }
            };
            
            results.push(result);
        }
        
        results
    }
}

/// Compatibility test runner
pub struct CompatibilityTestRunner {
    suite: CompatibilityTestSuite,
}

impl CompatibilityTestRunner {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let suite = CompatibilityTestSuite::new()?;
        Ok(Self { suite })
    }
    
    pub fn run_compatibility_tests(&self) -> Vec<CompatibilityTestResult> {
        self.suite.run_all_tests()
    }
    
    pub fn run_compatibility_tests_with_config(&mut self, config: CompatibilityTestConfig) -> Vec<CompatibilityTestResult> {
        self.suite = CompatibilityTestSuite::new().unwrap().with_config(config);
        self.suite.run_all_tests()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compatibility_config_creation() {
        let config = CompatibilityTestConfig::new();
        assert!(!config.target_versions.is_empty());
        assert!(!config.data_formats.is_empty());
    }
    
    #[test]
    fn test_compatibility_suite_creation() {
        let suite = CompatibilityTestSuite::new();
        assert!(suite.is_ok());
    }
    
    #[test]
    fn test_compatibility_runner_creation() {
        let runner = CompatibilityTestRunner::new();
        assert!(runner.is_ok());
    }
}
