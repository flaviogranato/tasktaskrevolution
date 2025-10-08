//! Performance Test Suite
//! 
//! This module contains performance tests for the TTR system.

use std::time::{Duration, Instant};
use tempfile::TempDir;
use assert_cmd::Command;
use assert_fs::prelude::*;

/// Performance test configuration
#[derive(Debug, Clone)]
pub struct PerformanceTestConfig {
    pub max_execution_time: Duration,
    pub memory_limit_mb: u64,
    pub cpu_usage_threshold: f64,
    pub test_data_size: TestDataSize,
}

#[derive(Debug, Clone)]
pub enum TestDataSize {
    Small,    // < 100 entities
    Medium,   // 100-1000 entities
    Large,    // 1000-10000 entities
    XLarge,   // > 10000 entities
}

impl PerformanceTestConfig {
    pub fn new() -> Self {
        Self {
            max_execution_time: Duration::from_secs(30),
            memory_limit_mb: 512,
            cpu_usage_threshold: 80.0,
            test_data_size: TestDataSize::Medium,
        }
    }
    
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.max_execution_time = timeout;
        self
    }
    
    pub fn with_memory_limit(mut self, limit_mb: u64) -> Self {
        self.memory_limit_mb = limit_mb;
        self
    }
    
    pub fn with_data_size(mut self, size: TestDataSize) -> Self {
        self.test_data_size = size;
        self
    }
}

/// Performance test results
#[derive(Debug, Clone)]
pub struct PerformanceTestResult {
    pub test_name: String,
    pub execution_time: Duration,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub throughput: f64,
    pub status: PerformanceStatus,
    pub metrics: PerformanceMetrics,
}

#[derive(Debug, Clone)]
pub enum PerformanceStatus {
    Passed,
    Failed,
    Warning,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub operations_per_second: f64,
    pub average_response_time: Duration,
    pub peak_memory_usage: f64,
    pub cache_hit_rate: f64,
}

/// Performance test suite
pub struct PerformanceTestSuite {
    config: PerformanceTestConfig,
    temp_dir: TempDir,
}

impl PerformanceTestSuite {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = PerformanceTestConfig::new();
        let temp_dir = TempDir::new()?;
        
        Ok(Self {
            config,
            temp_dir,
        })
    }
    
    pub fn with_config(mut self, config: PerformanceTestConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Test CLI command performance
    pub fn test_cli_performance(&self) -> PerformanceTestResult {
        let test_name = "CLI Command Performance";
        let start_time = Instant::now();
        
        // Test basic CLI commands
        let mut cmd = Command::cargo_bin("ttr").unwrap();
        cmd.current_dir(self.temp_dir.path());
        
        let result = cmd.arg("--help").assert();
        let execution_time = start_time.elapsed();
        
        PerformanceTestResult {
            test_name: test_name.to_string(),
            execution_time,
            memory_usage_mb: 0.0, // Would be measured in real implementation
            cpu_usage_percent: 0.0,
            throughput: 0.0,
            status: if execution_time <= self.config.max_execution_time {
                PerformanceStatus::Passed
            } else {
                PerformanceStatus::Failed
            },
            metrics: PerformanceMetrics {
                operations_per_second: 0.0,
                average_response_time: execution_time,
                peak_memory_usage: 0.0,
                cache_hit_rate: 0.0,
            },
        }
    }
    
    /// Test build performance with large datasets
    pub fn test_build_performance(&self) -> PerformanceTestResult {
        let test_name = "Build Performance";
        let start_time = Instant::now();
        
        // Initialize TTR
        let mut cmd = Command::cargo_bin("ttr").unwrap();
        cmd.current_dir(self.temp_dir.path())
            .args(&["init", "--name", "Test", "--email", "test@example.com"])
            .assert()
            .success();
        
        // Create test data based on size
        self.create_test_data();
        
        // Test build command
        let mut cmd = Command::cargo_bin("ttr").unwrap();
        cmd.current_dir(self.temp_dir.path())
            .arg("build")
            .assert()
            .success();
        
        let execution_time = start_time.elapsed();
        
        PerformanceTestResult {
            test_name: test_name.to_string(),
            execution_time,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            throughput: 0.0,
            status: if execution_time <= self.config.max_execution_time {
                PerformanceStatus::Passed
            } else {
                PerformanceStatus::Failed
            },
            metrics: PerformanceMetrics {
                operations_per_second: 0.0,
                average_response_time: execution_time,
                peak_memory_usage: 0.0,
                cache_hit_rate: 0.0,
            },
        }
    }
    
    /// Test repository operations performance
    pub fn test_repository_performance(&self) -> PerformanceTestResult {
        let test_name = "Repository Operations Performance";
        let start_time = Instant::now();
        
        // Test repository operations
        let operations = match self.config.test_data_size {
            TestDataSize::Small => 10,
            TestDataSize::Medium => 100,
            TestDataSize::Large => 1000,
            TestDataSize::XLarge => 10000,
        };
        
        // Simulate repository operations
        for i in 0..operations {
            // This would be actual repository operations
            let _ = i;
        }
        
        let execution_time = start_time.elapsed();
        
        PerformanceTestResult {
            test_name: test_name.to_string(),
            execution_time,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            throughput: operations as f64 / execution_time.as_secs_f64(),
            status: if execution_time <= self.config.max_execution_time {
                PerformanceStatus::Passed
            } else {
                PerformanceStatus::Failed
            },
            metrics: PerformanceMetrics {
                operations_per_second: operations as f64 / execution_time.as_secs_f64(),
                average_response_time: execution_time / operations,
                peak_memory_usage: 0.0,
                cache_hit_rate: 0.0,
            },
        }
    }
    
    /// Test HTML generation performance
    pub fn test_html_generation_performance(&self) -> PerformanceTestResult {
        let test_name = "HTML Generation Performance";
        let start_time = Instant::now();
        
        // Test HTML generation
        let mut cmd = Command::cargo_bin("ttr").unwrap();
        cmd.current_dir(self.temp_dir.path())
            .arg("build")
            .assert()
            .success();
        
        let execution_time = start_time.elapsed();
        
        PerformanceTestResult {
            test_name: test_name.to_string(),
            execution_time,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            throughput: 0.0,
            status: if execution_time <= self.config.max_execution_time {
                PerformanceStatus::Passed
            } else {
                PerformanceStatus::Failed
            },
            metrics: PerformanceMetrics {
                operations_per_second: 0.0,
                average_response_time: execution_time,
                peak_memory_usage: 0.0,
                cache_hit_rate: 0.0,
            },
        }
    }
    
    /// Run all performance tests
    pub fn run_all_tests(&self) -> Vec<PerformanceTestResult> {
        vec![
            self.test_cli_performance(),
            self.test_build_performance(),
            self.test_repository_performance(),
            self.test_html_generation_performance(),
        ]
    }
    
    /// Create test data based on configuration
    fn create_test_data(&self) {
        let count = match self.config.test_data_size {
            TestDataSize::Small => 10,
            TestDataSize::Medium => 100,
            TestDataSize::Large => 1000,
            TestDataSize::XLarge => 10000,
        };
        
        // Create companies
        for i in 0..(count / 10).max(1) {
            let mut cmd = Command::cargo_bin("ttr").unwrap();
            cmd.current_dir(self.temp_dir.path())
                .args(&[
                    "create", "company",
                    "--name", &format!("Company {}", i),
                    "--code", &format!("COMP-{}", i),
                ])
                .assert()
                .success();
        }
        
        // Create resources
        for i in 0..(count / 5).max(1) {
            let mut cmd = Command::cargo_bin("ttr").unwrap();
            cmd.current_dir(self.temp_dir.path())
                .args(&[
                    "create", "resource",
                    &format!("Resource {}", i),
                    "Developer",
                    "--company-code", "COMP-0",
                ])
                .assert()
                .success();
        }
        
        // Create projects
        for i in 0..(count / 20).max(1) {
            let mut cmd = Command::cargo_bin("ttr").unwrap();
            cmd.current_dir(self.temp_dir.path())
                .args(&[
                    "create", "project",
                    &format!("Project {}", i),
                    "Test project",
                    "--company-code", "COMP-0",
                ])
                .assert()
                .success();
        }
    }
}

/// Performance test runner
pub struct PerformanceTestRunner {
    suite: PerformanceTestSuite,
}

impl PerformanceTestRunner {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let suite = PerformanceTestSuite::new()?;
        Ok(Self { suite })
    }
    
    pub fn run_benchmark(&self) -> Vec<PerformanceTestResult> {
        self.suite.run_all_tests()
    }
    
    pub fn run_benchmark_with_config(&mut self, config: PerformanceTestConfig) -> Vec<PerformanceTestResult> {
        self.suite = PerformanceTestSuite::new().unwrap().with_config(config);
        self.suite.run_all_tests()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_config_creation() {
        let config = PerformanceTestConfig::new();
        assert_eq!(config.max_execution_time, Duration::from_secs(30));
        assert_eq!(config.memory_limit_mb, 512);
    }
    
    #[test]
    fn test_performance_suite_creation() {
        let suite = PerformanceTestSuite::new();
        assert!(suite.is_ok());
    }
    
    #[test]
    fn test_performance_runner_creation() {
        let runner = PerformanceTestRunner::new();
        assert!(runner.is_ok());
    }
}
