//! Test Suites Configuration
//! 
//! This module organizes test suites for comprehensive testing coverage.

pub mod unit_tests;
pub mod integration_tests;
pub mod performance_tests;
pub mod stress_tests;
pub mod regression_tests;
pub mod security_tests;
pub mod compatibility_tests;
pub mod e2e_tests;

/// Test suite configuration
#[derive(Debug, Clone)]
pub struct TestSuiteConfig {
    pub name: String,
    pub description: String,
    pub timeout_seconds: u64,
    pub parallel: bool,
    pub retry_count: u32,
    pub required: bool,
}

impl TestSuiteConfig {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            timeout_seconds: 300, // 5 minutes default
            parallel: true,
            retry_count: 0,
            required: true,
        }
    }
    
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout_seconds = timeout;
        self
    }
    
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }
    
    pub fn with_retry(mut self, retry_count: u32) -> Self {
        self.retry_count = retry_count;
        self
    }
    
    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}

/// Test suite registry
pub struct TestSuiteRegistry {
    pub suites: Vec<TestSuiteConfig>,
}

impl TestSuiteRegistry {
    pub fn new() -> Self {
        Self {
            suites: Vec::new(),
        }
    }
    
    pub fn add_suite(&mut self, config: TestSuiteConfig) {
        self.suites.push(config);
    }
    
    pub fn get_required_suites(&self) -> Vec<&TestSuiteConfig> {
        self.suites.iter()
            .filter(|suite| suite.required)
            .collect()
    }
    
    pub fn get_optional_suites(&self) -> Vec<&TestSuiteConfig> {
        self.suites.iter()
            .filter(|suite| !suite.required)
            .collect()
    }
}

/// Test execution result
#[derive(Debug, Clone)]
pub struct TestExecutionResult {
    pub suite_name: String,
    pub status: TestStatus,
    pub duration: std::time::Duration,
    pub test_count: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Timeout,
    Error,
}

impl TestExecutionResult {
    pub fn success_rate(&self) -> f64 {
        if self.test_count == 0 {
            0.0
        } else {
            (self.passed as f64 / self.test_count as f64) * 100.0
        }
    }
    
    pub fn is_successful(&self) -> bool {
        matches!(self.status, TestStatus::Passed)
    }
}

/// Test suite executor
pub struct TestSuiteExecutor {
    registry: TestSuiteRegistry,
}

impl TestSuiteExecutor {
    pub fn new() -> Self {
        Self {
            registry: TestSuiteRegistry::new(),
        }
    }
    
    pub fn register_suite(&mut self, config: TestSuiteConfig) {
        self.registry.add_suite(config);
    }
    
    pub fn execute_all(&self) -> Vec<TestExecutionResult> {
        let mut results = Vec::new();
        
        for suite in &self.registry.suites {
            let result = self.execute_suite(suite);
            results.push(result);
        }
        
        results
    }
    
    pub fn execute_required(&self) -> Vec<TestExecutionResult> {
        let mut results = Vec::new();
        
        for suite in self.registry.get_required_suites() {
            let result = self.execute_suite(suite);
            results.push(result);
        }
        
        results
    }
    
    fn execute_suite(&self, suite: &TestSuiteConfig) -> TestExecutionResult {
        let start_time = std::time::Instant::now();
        
        // This would be implemented with actual test execution logic
        // For now, we'll simulate the execution
        
        TestExecutionResult {
            suite_name: suite.name.clone(),
            status: TestStatus::Passed,
            duration: start_time.elapsed(),
            test_count: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            error_message: None,
        }
    }
}
