//! End-to-End Test Suite
//! 
//! This module contains end-to-end tests for the TTR system.

use std::time::{Duration, Instant};
use tempfile::TempDir;
use assert_cmd::Command;
use assert_fs::prelude::*;

/// E2E test configuration
#[derive(Debug, Clone)]
pub struct E2ETestConfig {
    pub test_scenarios: Vec<E2ETestScenario>,
    pub timeout_seconds: u64,
    pub retry_count: u32,
    pub parallel_execution: bool,
    pub data_cleanup: bool,
}

#[derive(Debug, Clone)]
pub enum E2ETestScenario {
    CompleteWorkflow,
    CompanyManagement,
    ProjectManagement,
    ResourceManagement,
    TaskManagement,
    HTMLGeneration,
    DataValidation,
    ErrorHandling,
    Performance,
    Security,
}

impl E2ETestConfig {
    pub fn new() -> Self {
        Self {
            test_scenarios: vec![
                E2ETestScenario::CompleteWorkflow,
                E2ETestScenario::CompanyManagement,
                E2ETestScenario::ProjectManagement,
            ],
            timeout_seconds: 300, // 5 minutes
            retry_count: 3,
            parallel_execution: false,
            data_cleanup: true,
        }
    }
    
    pub fn with_scenarios(mut self, scenarios: Vec<E2ETestScenario>) -> Self {
        self.test_scenarios = scenarios;
        self
    }
    
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout_seconds = timeout;
        self
    }
    
    pub fn with_retry(mut self, retry_count: u32) -> Self {
        self.retry_count = retry_count;
        self
    }
    
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel_execution = parallel;
        self
    }
}

/// E2E test results
#[derive(Debug, Clone)]
pub struct E2ETestResult {
    pub test_name: String,
    pub scenario: E2ETestScenario,
    pub execution_time: Duration,
    pub steps_completed: u32,
    pub steps_failed: u32,
    pub status: E2ETestStatus,
    pub error_message: Option<String>,
    pub metrics: E2ETestMetrics,
}

#[derive(Debug, Clone)]
pub enum E2ETestStatus {
    Passed,
    Failed,
    Timeout,
    Error,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct E2ETestMetrics {
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub disk_usage_mb: f64,
    pub network_requests: u32,
    pub response_time_ms: u64,
}

/// E2E test suite
pub struct E2ETestSuite {
    config: E2ETestConfig,
    temp_dir: TempDir,
}

impl E2ETestSuite {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = E2ETestConfig::new();
        let temp_dir = TempDir::new()?;
        
        Ok(Self {
            config,
            temp_dir,
        })
    }
    
    pub fn with_config(mut self, config: E2ETestConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Test complete workflow
    pub fn test_complete_workflow(&self) -> E2ETestResult {
        let test_name = "Complete Workflow";
        let start_time = Instant::now();
        let mut steps_completed = 0;
        let mut steps_failed = 0;
        let mut error_message = None;
        
        // Step 1: Initialize TTR
        let mut cmd = Command::cargo_bin("ttr").unwrap();
        cmd.current_dir(self.temp_dir.path());
        
        let result = cmd.args(&[
            "init",
            "--name", "Test Manager",
            "--email", "test@example.com",
            "--company-name", "Test Company"
        ]).assert();
        
        if result.is_ok() {
            steps_completed += 1;
        } else {
            steps_failed += 1;
            error_message = Some("Failed to initialize TTR".to_string());
        }
        
        // Step 2: Create company
        if steps_failed == 0 {
            let mut cmd = Command::cargo_bin("ttr").unwrap();
            cmd.current_dir(self.temp_dir.path());
            
            let result = cmd.args(&[
                "create", "company",
                "--name", "Tech Corp",
                "--code", "TECH-CORP",
                "--description", "Technology company"
            ]).assert();
            
            if result.is_ok() {
                steps_completed += 1;
            } else {
                steps_failed += 1;
                error_message = Some("Failed to create company".to_string());
            }
        }
        
        // Step 3: Create resource
        if steps_failed == 0 {
            let mut cmd = Command::cargo_bin("ttr").unwrap();
            cmd.current_dir(self.temp_dir.path());
            
            let result = cmd.args(&[
                "create", "resource",
                "John Doe", "Developer",
                "--company-code", "TECH-CORP"
            ]).assert();
            
            if result.is_ok() {
                steps_completed += 1;
            } else {
                steps_failed += 1;
                error_message = Some("Failed to create resource".to_string());
            }
        }
        
        // Step 4: Create project
        if steps_failed == 0 {
            let mut cmd = Command::cargo_bin("ttr").unwrap();
            cmd.current_dir(self.temp_dir.path());
            
            let result = cmd.args(&[
                "create", "project",
                "Web App", "Web application project",
                "--company-code", "TECH-CORP"
            ]).assert();
            
            if result.is_ok() {
                steps_completed += 1;
            } else {
                steps_failed += 1;
                error_message = Some("Failed to create project".to_string());
            }
        }
        
        // Step 5: Create task
        if steps_failed == 0 {
            let mut cmd = Command::cargo_bin("ttr").unwrap();
            cmd.current_dir(self.temp_dir.path());
            
            let result = cmd.args(&[
                "create", "task",
                "--name", "Implement feature",
                "--description", "Implement new feature",
                "--start-date", "2024-01-01",
                "--due-date", "2024-01-31",
                "--project-code", "web-app",
                "--company-code", "TECH-CORP"
            ]).assert();
            
            if result.is_ok() {
                steps_completed += 1;
            } else {
                steps_failed += 1;
                error_message = Some("Failed to create task".to_string());
            }
        }
        
        // Step 6: Generate HTML
        if steps_failed == 0 {
            let mut cmd = Command::cargo_bin("ttr").unwrap();
            cmd.current_dir(self.temp_dir.path());
            
            let result = cmd.arg("build").assert();
            
            if result.is_ok() {
                steps_completed += 1;
            } else {
                steps_failed += 1;
                error_message = Some("Failed to generate HTML".to_string());
            }
        }
        
        let execution_time = start_time.elapsed();
        
        E2ETestResult {
            test_name: test_name.to_string(),
            scenario: E2ETestScenario::CompleteWorkflow,
            execution_time,
            steps_completed,
            steps_failed,
            status: if steps_failed == 0 {
                E2ETestStatus::Passed
            } else {
                E2ETestStatus::Failed
            },
            error_message,
            metrics: E2ETestMetrics {
                memory_usage_mb: 0.0,
                cpu_usage_percent: 0.0,
                disk_usage_mb: 0.0,
                network_requests: 0,
                response_time_ms: execution_time.as_millis() as u64,
            },
        }
    }
    
    /// Test company management workflow
    pub fn test_company_management(&self) -> E2ETestResult {
        let test_name = "Company Management";
        let start_time = Instant::now();
        let mut steps_completed = 0;
        let mut steps_failed = 0;
        let mut error_message = None;
        
        // Initialize TTR
        let mut cmd = Command::cargo_bin("ttr").unwrap();
        cmd.current_dir(self.temp_dir.path());
        
        let result = cmd.args(&[
            "init",
            "--name", "Test Manager",
            "--email", "test@example.com"
        ]).assert();
        
        if result.is_ok() {
            steps_completed += 1;
        } else {
            steps_failed += 1;
            error_message = Some("Failed to initialize TTR".to_string());
        }
        
        // Create company
        if steps_failed == 0 {
            let mut cmd = Command::cargo_bin("ttr").unwrap();
            cmd.current_dir(self.temp_dir.path());
            
            let result = cmd.args(&[
                "create", "company",
                "--name", "Test Company",
                "--code", "TEST-COMP",
                "--description", "Test company description"
            ]).assert();
            
            if result.is_ok() {
                steps_completed += 1;
            } else {
                steps_failed += 1;
                error_message = Some("Failed to create company".to_string());
            }
        }
        
        // List companies
        if steps_failed == 0 {
            let mut cmd = Command::cargo_bin("ttr").unwrap();
            cmd.current_dir(self.temp_dir.path());
            
            let result = cmd.args(&["list", "companies"]).assert();
            
            if result.is_ok() {
                steps_completed += 1;
            } else {
                steps_failed += 1;
                error_message = Some("Failed to list companies".to_string());
            }
        }
        
        // Update company
        if steps_failed == 0 {
            let mut cmd = Command::cargo_bin("ttr").unwrap();
            cmd.current_dir(self.temp_dir.path());
            
            let result = cmd.args(&[
                "update", "company",
                "--code", "TEST-COMP",
                "--name", "Updated Test Company"
            ]).assert();
            
            if result.is_ok() {
                steps_completed += 1;
            } else {
                steps_failed += 1;
                error_message = Some("Failed to update company".to_string());
            }
        }
        
        let execution_time = start_time.elapsed();
        
        E2ETestResult {
            test_name: test_name.to_string(),
            scenario: E2ETestScenario::CompanyManagement,
            execution_time,
            steps_completed,
            steps_failed,
            status: if steps_failed == 0 {
                E2ETestStatus::Passed
            } else {
                E2ETestStatus::Failed
            },
            error_message,
            metrics: E2ETestMetrics {
                memory_usage_mb: 0.0,
                cpu_usage_percent: 0.0,
                disk_usage_mb: 0.0,
                network_requests: 0,
                response_time_ms: execution_time.as_millis() as u64,
            },
        }
    }
    
    /// Test project management workflow
    pub fn test_project_management(&self) -> E2ETestResult {
        let test_name = "Project Management";
        let start_time = Instant::now();
        let mut steps_completed = 0;
        let mut steps_failed = 0;
        let mut error_message = None;
        
        // Initialize TTR and create company
        let mut cmd = Command::cargo_bin("ttr").unwrap();
        cmd.current_dir(self.temp_dir.path());
        
        let result = cmd.args(&[
            "init",
            "--name", "Test Manager",
            "--email", "test@example.com"
        ]).assert();
        
        if result.is_ok() {
            steps_completed += 1;
        } else {
            steps_failed += 1;
            error_message = Some("Failed to initialize TTR".to_string());
        }
        
        // Create company
        if steps_failed == 0 {
            let mut cmd = Command::cargo_bin("ttr").unwrap();
            cmd.current_dir(self.temp_dir.path());
            
            let result = cmd.args(&[
                "create", "company",
                "--name", "Test Company",
                "--code", "TEST-COMP"
            ]).assert();
            
            if result.is_ok() {
                steps_completed += 1;
            } else {
                steps_failed += 1;
                error_message = Some("Failed to create company".to_string());
            }
        }
        
        // Create project
        if steps_failed == 0 {
            let mut cmd = Command::cargo_bin("ttr").unwrap();
            cmd.current_dir(self.temp_dir.path());
            
            let result = cmd.args(&[
                "create", "project",
                "Test Project", "Test project description",
                "--company-code", "TEST-COMP"
            ]).assert();
            
            if result.is_ok() {
                steps_completed += 1;
            } else {
                steps_failed += 1;
                error_message = Some("Failed to create project".to_string());
            }
        }
        
        // List projects
        if steps_failed == 0 {
            let mut cmd = Command::cargo_bin("ttr").unwrap();
            cmd.current_dir(self.temp_dir.path());
            
            let result = cmd.args(&["list", "projects"]).assert();
            
            if result.is_ok() {
                steps_completed += 1;
            } else {
                steps_failed += 1;
                error_message = Some("Failed to list projects".to_string());
            }
        }
        
        let execution_time = start_time.elapsed();
        
        E2ETestResult {
            test_name: test_name.to_string(),
            scenario: E2ETestScenario::ProjectManagement,
            execution_time,
            steps_completed,
            steps_failed,
            status: if steps_failed == 0 {
                E2ETestStatus::Passed
            } else {
                E2ETestStatus::Failed
            },
            error_message,
            metrics: E2ETestMetrics {
                memory_usage_mb: 0.0,
                cpu_usage_percent: 0.0,
                disk_usage_mb: 0.0,
                network_requests: 0,
                response_time_ms: execution_time.as_millis() as u64,
            },
        }
    }
    
    /// Test HTML generation workflow
    pub fn test_html_generation(&self) -> E2ETestResult {
        let test_name = "HTML Generation";
        let start_time = Instant::now();
        let mut steps_completed = 0;
        let mut steps_failed = 0;
        let mut error_message = None;
        
        // Initialize TTR
        let mut cmd = Command::cargo_bin("ttr").unwrap();
        cmd.current_dir(self.temp_dir.path());
        
        let result = cmd.args(&[
            "init",
            "--name", "Test Manager",
            "--email", "test@example.com"
        ]).assert();
        
        if result.is_ok() {
            steps_completed += 1;
        } else {
            steps_failed += 1;
            error_message = Some("Failed to initialize TTR".to_string());
        }
        
        // Create test data
        if steps_failed == 0 {
            let mut cmd = Command::cargo_bin("ttr").unwrap();
            cmd.current_dir(self.temp_dir.path());
            
            let result = cmd.args(&[
                "create", "company",
                "--name", "Test Company",
                "--code", "TEST-COMP"
            ]).assert();
            
            if result.is_ok() {
                steps_completed += 1;
            } else {
                steps_failed += 1;
                error_message = Some("Failed to create company".to_string());
            }
        }
        
        // Generate HTML
        if steps_failed == 0 {
            let mut cmd = Command::cargo_bin("ttr").unwrap();
            cmd.current_dir(self.temp_dir.path());
            
            let result = cmd.arg("build").assert();
            
            if result.is_ok() {
                steps_completed += 1;
            } else {
                steps_failed += 1;
                error_message = Some("Failed to generate HTML".to_string());
            }
        }
        
        // Verify HTML files exist
        if steps_failed == 0 {
            let html_dir = self.temp_dir.path().join("output");
            if html_dir.exists() {
                steps_completed += 1;
            } else {
                steps_failed += 1;
                error_message = Some("HTML output directory not found".to_string());
            }
        }
        
        let execution_time = start_time.elapsed();
        
        E2ETestResult {
            test_name: test_name.to_string(),
            scenario: E2ETestScenario::HTMLGeneration,
            execution_time,
            steps_completed,
            steps_failed,
            status: if steps_failed == 0 {
                E2ETestStatus::Passed
            } else {
                E2ETestStatus::Failed
            },
            error_message,
            metrics: E2ETestMetrics {
                memory_usage_mb: 0.0,
                cpu_usage_percent: 0.0,
                disk_usage_mb: 0.0,
                network_requests: 0,
                response_time_ms: execution_time.as_millis() as u64,
            },
        }
    }
    
    /// Run all E2E tests
    pub fn run_all_tests(&self) -> Vec<E2ETestResult> {
        let mut results = Vec::new();
        
        for scenario in &self.config.test_scenarios {
            let result = match scenario {
                E2ETestScenario::CompleteWorkflow => {
                    self.test_complete_workflow()
                }
                E2ETestScenario::CompanyManagement => {
                    self.test_company_management()
                }
                E2ETestScenario::ProjectManagement => {
                    self.test_project_management()
                }
                E2ETestScenario::HTMLGeneration => {
                    self.test_html_generation()
                }
                _ => {
                    // Other scenarios would be implemented here
                    E2ETestResult {
                        test_name: format!("{:?} E2E Test", scenario),
                        scenario: scenario.clone(),
                        execution_time: Duration::from_secs(0),
                        steps_completed: 0,
                        steps_failed: 0,
                        status: E2ETestStatus::Skipped,
                        error_message: Some("Test not implemented".to_string()),
                        metrics: E2ETestMetrics {
                            memory_usage_mb: 0.0,
                            cpu_usage_percent: 0.0,
                            disk_usage_mb: 0.0,
                            network_requests: 0,
                            response_time_ms: 0,
                        },
                    }
                }
            };
            
            results.push(result);
        }
        
        results
    }
}

/// E2E test runner
pub struct E2ETestRunner {
    suite: E2ETestSuite,
}

impl E2ETestRunner {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let suite = E2ETestSuite::new()?;
        Ok(Self { suite })
    }
    
    pub fn run_e2e_tests(&self) -> Vec<E2ETestResult> {
        self.suite.run_all_tests()
    }
    
    pub fn run_e2e_tests_with_config(&mut self, config: E2ETestConfig) -> Vec<E2ETestResult> {
        self.suite = E2ETestSuite::new().unwrap().with_config(config);
        self.suite.run_all_tests()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_e2e_config_creation() {
        let config = E2ETestConfig::new();
        assert!(!config.test_scenarios.is_empty());
        assert_eq!(config.timeout_seconds, 300);
    }
    
    #[test]
    fn test_e2e_suite_creation() {
        let suite = E2ETestSuite::new();
        assert!(suite.is_ok());
    }
    
    #[test]
    fn test_e2e_runner_creation() {
        let runner = E2ETestRunner::new();
        assert!(runner.is_ok());
    }
}
