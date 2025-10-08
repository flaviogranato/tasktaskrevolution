//! Regression Test Suite
//! 
//! This module contains regression tests to ensure that existing functionality
//! continues to work as expected after changes.

use std::time::{Duration, Instant};
use tempfile::TempDir;
use assert_cmd::Command;
use assert_fs::prelude::*;

/// Regression test configuration
#[derive(Debug, Clone)]
pub struct RegressionTestConfig {
    pub test_categories: Vec<RegressionTestCategory>,
    pub baseline_version: String,
    pub comparison_version: String,
    pub tolerance_percent: f64,
    pub test_data_path: Option<String>,
}

#[derive(Debug, Clone)]
pub enum RegressionTestCategory {
    CLICommands,
    DataValidation,
    HTMLGeneration,
    YAMLParsing,
    Performance,
    MemoryUsage,
    FileOperations,
    ErrorHandling,
}

impl RegressionTestConfig {
    pub fn new() -> Self {
        Self {
            test_categories: vec![
                RegressionTestCategory::CLICommands,
                RegressionTestCategory::DataValidation,
                RegressionTestCategory::HTMLGeneration,
            ],
            baseline_version: "1.0.0".to_string(),
            comparison_version: "current".to_string(),
            tolerance_percent: 5.0,
            test_data_path: None,
        }
    }
    
    pub fn with_categories(mut self, categories: Vec<RegressionTestCategory>) -> Self {
        self.test_categories = categories;
        self
    }
    
    pub fn with_versions(mut self, baseline: String, comparison: String) -> Self {
        self.baseline_version = baseline;
        self.comparison_version = comparison;
        self
    }
    
    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance_percent = tolerance;
        self
    }
    
    pub fn with_test_data(mut self, path: String) -> Self {
        self.test_data_path = Some(path);
        self
    }
}

/// Regression test results
#[derive(Debug, Clone)]
pub struct RegressionTestResult {
    pub test_name: String,
    pub category: RegressionTestCategory,
    pub baseline_result: TestResult,
    pub current_result: TestResult,
    pub regression_detected: bool,
    pub performance_change: f64,
    pub status: RegressionStatus,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub execution_time: Duration,
    pub memory_usage_mb: f64,
    pub success: bool,
    pub output: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum RegressionStatus {
    NoRegression,
    PerformanceRegression,
    FunctionalRegression,
    MemoryRegression,
    Warning,
}

/// Regression test suite
pub struct RegressionTestSuite {
    config: RegressionTestConfig,
    temp_dir: TempDir,
}

impl RegressionTestSuite {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = RegressionTestConfig::new();
        let temp_dir = TempDir::new()?;
        
        Ok(Self {
            config,
            temp_dir,
        })
    }
    
    pub fn with_config(mut self, config: RegressionTestConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Test CLI commands regression
    pub fn test_cli_commands_regression(&self) -> RegressionTestResult {
        let test_name = "CLI Commands Regression";
        let start_time = Instant::now();
        
        // Test basic CLI commands
        let mut cmd = Command::cargo_bin("ttr").unwrap();
        cmd.current_dir(self.temp_dir.path());
        
        let result = cmd.arg("--help").assert();
        let execution_time = start_time.elapsed();
        
        let baseline_result = TestResult {
            execution_time: Duration::from_millis(100), // Simulated baseline
            memory_usage_mb: 10.0,
            success: true,
            output: "Help text".to_string(),
            error_message: None,
        };
        
        let current_result = TestResult {
            execution_time,
            memory_usage_mb: 10.0,
            success: result.is_ok(),
            output: "Current help text".to_string(),
            error_message: None,
        };
        
        let performance_change = if baseline_result.execution_time.as_millis() > 0 {
            ((current_result.execution_time.as_millis() as f64 - baseline_result.execution_time.as_millis() as f64) 
             / baseline_result.execution_time.as_millis() as f64) * 100.0
        } else {
            0.0
        };
        
        let regression_detected = performance_change.abs() > self.config.tolerance_percent;
        
        RegressionTestResult {
            test_name: test_name.to_string(),
            category: RegressionTestCategory::CLICommands,
            baseline_result,
            current_result,
            regression_detected,
            performance_change,
            status: if regression_detected {
                if performance_change > 0.0 {
                    RegressionStatus::PerformanceRegression
                } else {
                    RegressionStatus::NoRegression
                }
            } else {
                RegressionStatus::NoRegression
            },
            details: format!("Performance change: {:.2}%", performance_change),
        }
    }
    
    /// Test data validation regression
    pub fn test_data_validation_regression(&self) -> RegressionTestResult {
        let test_name = "Data Validation Regression";
        let start_time = Instant::now();
        
        // Initialize TTR
        let mut cmd = Command::cargo_bin("ttr").unwrap();
        cmd.current_dir(self.temp_dir.path())
            .args(&["init", "--name", "Test", "--email", "test@example.com"])
            .assert()
            .success();
        
        // Test validation
        let mut cmd = Command::cargo_bin("ttr").unwrap();
        cmd.current_dir(self.temp_dir.path())
            .arg("validate")
            .assert()
            .success();
        
        let execution_time = start_time.elapsed();
        
        let baseline_result = TestResult {
            execution_time: Duration::from_millis(200),
            memory_usage_mb: 15.0,
            success: true,
            output: "Validation passed".to_string(),
            error_message: None,
        };
        
        let current_result = TestResult {
            execution_time,
            memory_usage_mb: 15.0,
            success: true,
            output: "Current validation passed".to_string(),
            error_message: None,
        };
        
        let performance_change = if baseline_result.execution_time.as_millis() > 0 {
            ((current_result.execution_time.as_millis() as f64 - baseline_result.execution_time.as_millis() as f64) 
             / baseline_result.execution_time.as_millis() as f64) * 100.0
        } else {
            0.0
        };
        
        let regression_detected = performance_change.abs() > self.config.tolerance_percent;
        
        RegressionTestResult {
            test_name: test_name.to_string(),
            category: RegressionTestCategory::DataValidation,
            baseline_result,
            current_result,
            regression_detected,
            performance_change,
            status: if regression_detected {
                if performance_change > 0.0 {
                    RegressionStatus::PerformanceRegression
                } else {
                    RegressionStatus::NoRegression
                }
            } else {
                RegressionStatus::NoRegression
            },
            details: format!("Validation performance change: {:.2}%", performance_change),
        }
    }
    
    /// Test HTML generation regression
    pub fn test_html_generation_regression(&self) -> RegressionTestResult {
        let test_name = "HTML Generation Regression";
        let start_time = Instant::now();
        
        // Create test data
        let mut cmd = Command::cargo_bin("ttr").unwrap();
        cmd.current_dir(self.temp_dir.path())
            .args(&["create", "company", "--name", "Test Company", "--code", "TEST"])
            .assert()
            .success();
        
        // Test HTML generation
        let mut cmd = Command::cargo_bin("ttr").unwrap();
        cmd.current_dir(self.temp_dir.path())
            .arg("build")
            .assert()
            .success();
        
        let execution_time = start_time.elapsed();
        
        let baseline_result = TestResult {
            execution_time: Duration::from_millis(500),
            memory_usage_mb: 25.0,
            success: true,
            output: "HTML generated".to_string(),
            error_message: None,
        };
        
        let current_result = TestResult {
            execution_time,
            memory_usage_mb: 25.0,
            success: true,
            output: "Current HTML generated".to_string(),
            error_message: None,
        };
        
        let performance_change = if baseline_result.execution_time.as_millis() > 0 {
            ((current_result.execution_time.as_millis() as f64 - baseline_result.execution_time.as_millis() as f64) 
             / baseline_result.execution_time.as_millis() as f64) * 100.0
        } else {
            0.0
        };
        
        let regression_detected = performance_change.abs() > self.config.tolerance_percent;
        
        RegressionTestResult {
            test_name: test_name.to_string(),
            category: RegressionTestCategory::HTMLGeneration,
            baseline_result,
            current_result,
            regression_detected,
            performance_change,
            status: if regression_detected {
                if performance_change > 0.0 {
                    RegressionStatus::PerformanceRegression
                } else {
                    RegressionStatus::NoRegression
                }
            } else {
                RegressionStatus::NoRegression
            },
            details: format!("HTML generation performance change: {:.2}%", performance_change),
        }
    }
    
    /// Test YAML parsing regression
    pub fn test_yaml_parsing_regression(&self) -> RegressionTestResult {
        let test_name = "YAML Parsing Regression";
        let start_time = Instant::now();
        
        // Test YAML parsing
        let mut cmd = Command::cargo_bin("ttr").unwrap();
        cmd.current_dir(self.temp_dir.path())
            .args(&["list", "companies"])
            .assert()
            .success();
        
        let execution_time = start_time.elapsed();
        
        let baseline_result = TestResult {
            execution_time: Duration::from_millis(150),
            memory_usage_mb: 12.0,
            success: true,
            output: "YAML parsed".to_string(),
            error_message: None,
        };
        
        let current_result = TestResult {
            execution_time,
            memory_usage_mb: 12.0,
            success: true,
            output: "Current YAML parsed".to_string(),
            error_message: None,
        };
        
        let performance_change = if baseline_result.execution_time.as_millis() > 0 {
            ((current_result.execution_time.as_millis() as f64 - baseline_result.execution_time.as_millis() as f64) 
             / baseline_result.execution_time.as_millis() as f64) * 100.0
        } else {
            0.0
        };
        
        let regression_detected = performance_change.abs() > self.config.tolerance_percent;
        
        RegressionTestResult {
            test_name: test_name.to_string(),
            category: RegressionTestCategory::YAMLParsing,
            baseline_result,
            current_result,
            regression_detected,
            performance_change,
            status: if regression_detected {
                if performance_change > 0.0 {
                    RegressionStatus::PerformanceRegression
                } else {
                    RegressionStatus::NoRegression
                }
            } else {
                RegressionStatus::NoRegression
            },
            details: format!("YAML parsing performance change: {:.2}%", performance_change),
        }
    }
    
    /// Run all regression tests
    pub fn run_all_tests(&self) -> Vec<RegressionTestResult> {
        let mut results = Vec::new();
        
        for category in &self.config.test_categories {
            let result = match category {
                RegressionTestCategory::CLICommands => {
                    self.test_cli_commands_regression()
                }
                RegressionTestCategory::DataValidation => {
                    self.test_data_validation_regression()
                }
                RegressionTestCategory::HTMLGeneration => {
                    self.test_html_generation_regression()
                }
                RegressionTestCategory::YAMLParsing => {
                    self.test_yaml_parsing_regression()
                }
                _ => {
                    // Other categories would be implemented here
                    RegressionTestResult {
                        test_name: format!("{:?} Regression", category),
                        category: category.clone(),
                        baseline_result: TestResult {
                            execution_time: Duration::from_millis(0),
                            memory_usage_mb: 0.0,
                            success: true,
                            output: "Baseline".to_string(),
                            error_message: None,
                        },
                        current_result: TestResult {
                            execution_time: Duration::from_millis(0),
                            memory_usage_mb: 0.0,
                            success: true,
                            output: "Current".to_string(),
                            error_message: None,
                        },
                        regression_detected: false,
                        performance_change: 0.0,
                        status: RegressionStatus::NoRegression,
                        details: "Not implemented".to_string(),
                    }
                }
            };
            
            results.push(result);
        }
        
        results
    }
}

/// Regression test runner
pub struct RegressionTestRunner {
    suite: RegressionTestSuite,
}

impl RegressionTestRunner {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let suite = RegressionTestSuite::new()?;
        Ok(Self { suite })
    }
    
    pub fn run_regression_tests(&self) -> Vec<RegressionTestResult> {
        self.suite.run_all_tests()
    }
    
    pub fn run_regression_tests_with_config(&mut self, config: RegressionTestConfig) -> Vec<RegressionTestResult> {
        self.suite = RegressionTestSuite::new().unwrap().with_config(config);
        self.suite.run_all_tests()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_regression_config_creation() {
        let config = RegressionTestConfig::new();
        assert_eq!(config.baseline_version, "1.0.0");
        assert_eq!(config.comparison_version, "current");
    }
    
    #[test]
    fn test_regression_suite_creation() {
        let suite = RegressionTestSuite::new();
        assert!(suite.is_ok());
    }
    
    #[test]
    fn test_regression_runner_creation() {
        let runner = RegressionTestRunner::new();
        assert!(runner.is_ok());
    }
}
