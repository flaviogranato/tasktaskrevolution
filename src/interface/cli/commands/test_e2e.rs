use crate::application::errors::AppError;
use crate::application::test::e2e_testing::*;
use clap::{Args, Parser};
use std::path::PathBuf;
use std::time::Duration;

/// E2E testing command
#[derive(Parser, Debug)]
pub enum TestE2ECommand {
    /// Run all E2E tests
    Run(RunE2ETestsArgs),
    /// Generate E2E test report
    Report(ReportE2EArgs),
    /// Validate HTML files
    Validate(ValidateHTMLArgs),
    /// Create E2E test template
    Create(CreateE2ETestArgs),
}

/// Arguments for running E2E tests
#[derive(Args, Debug)]
pub struct RunE2ETestsArgs {
    /// Test suite to run
    #[clap(short, long, default_value = "all")]
    pub suite: String,

    /// Browser to use for testing
    #[clap(short, long, default_value = "chrome")]
    pub browser: String,

    /// Run tests in headless mode
    #[clap(long)]
    pub headless: bool,

    /// Run tests in parallel
    #[clap(long)]
    pub parallel: bool,

    /// Base URL for testing
    #[clap(long, default_value = "http://localhost:3000")]
    pub base_url: String,

    /// Timeout for tests in seconds
    #[clap(long, default_value = "30")]
    pub timeout: u64,

    /// Generate screenshots on failure
    #[clap(long)]
    pub screenshot_on_failure: bool,

    /// Record video of test execution
    #[clap(long)]
    pub video_recording: bool,

    /// Output directory for test results
    #[clap(short, long, default_value = "./test-results")]
    pub output: PathBuf,

    /// Test type filter
    #[clap(long)]
    pub test_type: Option<String>,

    /// Viewport width
    #[clap(long, default_value = "1920")]
    pub viewport_width: u32,

    /// Viewport height
    #[clap(long, default_value = "1080")]
    pub viewport_height: u32,
}

/// Arguments for generating E2E test reports
#[derive(Args, Debug)]
pub struct ReportE2EArgs {
    /// Input directory containing test results
    #[clap(short, long, default_value = "./test-results")]
    pub input: PathBuf,

    /// Output file for the report
    #[clap(short, long, default_value = "./e2e-report.html")]
    pub output: PathBuf,

    /// Report format
    #[clap(long, default_value = "html")]
    pub format: String,
}

/// Arguments for HTML validation
#[derive(Args, Debug)]
pub struct ValidateHTMLArgs {
    /// HTML file or directory to validate
    #[clap(short, long)]
    pub input: PathBuf,

    /// Output file for validation results
    #[clap(short, long, default_value = "./validation-results.json")]
    pub output: PathBuf,

    /// Validation rules to apply
    #[clap(long)]
    pub rules: Option<Vec<String>>,

    /// Fail on warnings
    #[clap(long)]
    pub strict: bool,
}

/// Arguments for creating E2E test templates
#[derive(Args, Debug)]
pub struct CreateE2ETestArgs {
    /// Test name
    #[clap(short, long)]
    pub name: String,

    /// Test description
    #[clap(short, long)]
    pub description: String,

    /// Test type
    #[clap(long, default_value = "smoke")]
    pub test_type: String,

    /// Output file for the test
    #[clap(short, long, default_value = "./test.yaml")]
    pub output: PathBuf,
}

impl TestE2ECommand {
    /// Execute the E2E test command
    pub async fn execute(&self, base_path: &str) -> Result<(), AppError> {
        match self {
            TestE2ECommand::Run(args) => Self::run_tests(args, base_path).await,
            TestE2ECommand::Report(args) => Self::generate_report(args, base_path).await,
            TestE2ECommand::Validate(args) => Self::validate_html(args, base_path).await,
            TestE2ECommand::Create(args) => Self::create_test_template(args, base_path).await,
        }
    }

    /// Run E2E tests
    async fn run_tests(args: &RunE2ETestsArgs, _base_path: &str) -> Result<(), AppError> {
        println!("🚀 Starting E2E tests...");
        println!("📊 Configuration:");
        println!("  Suite: {}", args.suite);
        println!("  Browser: {}", args.browser);
        println!("  Headless: {}", args.headless);
        println!("  Parallel: {}", args.parallel);
        println!("  Base URL: {}", args.base_url);
        println!("  Timeout: {}s", args.timeout);
        println!("  Output: {:?}", args.output);

        // Create test configuration
        let config = E2ETestConfig {
            base_url: args.base_url.clone(),
            browsers: vec![Self::parse_browser(&args.browser)],
            viewport: Viewport {
                width: args.viewport_width,
                height: args.viewport_height,
                device_scale_factor: 1.0,
            },
            timeout: Duration::from_secs(args.timeout),
            headless: args.headless,
            parallel: args.parallel,
            screenshot_on_failure: args.screenshot_on_failure,
            video_recording: args.video_recording,
        };

        // Create test runner
        let mut runner = E2ETestRunner::new(config);

        // Create test suite
        let suite = Self::create_test_suite(&args.suite, &args.test_type)?;

        // Run tests
        let _results = runner.run_suite(suite).await?;

        // Generate report
        let report = runner.generate_report();
        Self::save_report(&report, &args.output).await?;

        // Print summary
        Self::print_test_summary(&report);

        Ok(())
    }

    /// Generate E2E test report
    async fn generate_report(args: &ReportE2EArgs, _base_path: &str) -> Result<(), AppError> {
        println!("📊 Generating E2E test report...");
        println!("  Input: {:?}", args.input);
        println!("  Output: {:?}", args.output);
        println!("  Format: {}", args.format);

        // Load test results
        let results = Self::load_test_results(&args.input).await?;

        // Generate report based on format
        match args.format.as_str() {
            "html" => Self::generate_html_report(&results, &args.output).await?,
            "json" => Self::generate_json_report(&results, &args.output).await?,
            "csv" => Self::generate_csv_report(&results, &args.output).await?,
            _ => {
                return Err(AppError::ValidationError {
                    field: "format".to_string(),
                    message: "Unsupported report format".to_string(),
                });
            }
        }

        println!("✅ Report generated successfully!");
        Ok(())
    }

    /// Validate HTML files
    async fn validate_html(args: &ValidateHTMLArgs, _base_path: &str) -> Result<(), AppError> {
        println!("🔍 Validating HTML files...");
        println!("  Input: {:?}", args.input);
        println!("  Output: {:?}", args.output);
        println!("  Strict: {}", args.strict);

        let validator = HTMLValidator::new();
        let mut all_results = Vec::new();

        if args.input.is_file() {
            let html = std::fs::read_to_string(&args.input)?;
            let results = validator.validate(&html);
            all_results.extend(results);
        } else if args.input.is_dir() {
            for entry in std::fs::read_dir(&args.input)? {
                let entry = entry?;
                if entry.path().extension().map_or(false, |ext| ext == "html") {
                    let html = std::fs::read_to_string(entry.path())?;
                    let results = validator.validate(&html);
                    all_results.extend(results);
                }
            }
        }

        // Save validation results
        let json_results = serde_json::to_string_pretty(&all_results)?;
        std::fs::write(&args.output, json_results)?;

        // Print summary
        let passed = all_results.iter().filter(|r| r.passed).count();
        let failed = all_results.len() - passed;
        println!("✅ Validation complete!");
        println!("  Passed: {}", passed);
        println!("  Failed: {}", failed);

        if args.strict && failed > 0 {
            return Err(AppError::ValidationError {
                field: "validation".to_string(),
                message: format!("{} validation errors found", failed),
            });
        }

        Ok(())
    }

    /// Create E2E test template
    async fn create_test_template(args: &CreateE2ETestArgs, _base_path: &str) -> Result<(), AppError> {
        println!("📝 Creating E2E test template...");
        println!("  Name: {}", args.name);
        println!("  Description: {}", args.description);
        println!("  Type: {}", args.test_type);
        println!("  Output: {:?}", args.output);

        let test_type = Self::parse_test_type(&args.test_type)?;

        let test = E2ETest {
            name: args.name.clone(),
            description: args.description.clone(),
            test_type,
            steps: vec![
                TestStep {
                    action: TestAction::Navigate,
                    selector: None,
                    value: Some("/".to_string()),
                    wait_time: None,
                },
                TestStep {
                    action: TestAction::Assert,
                    selector: Some("h1".to_string()),
                    value: None,
                    wait_time: None,
                },
            ],
            expected_results: vec![ExpectedResult {
                assertion_type: AssertionType::ElementExists,
                selector: Some("h1".to_string()),
                expected_value: None,
                message: "Page should have h1 element".to_string(),
            }],
        };

        let yaml_content = serde_yaml::to_string(&test)?;
        std::fs::write(&args.output, yaml_content)?;

        println!("✅ Test template created successfully!");
        Ok(())
    }

    /// Parse browser string to Browser enum
    fn parse_browser(browser: &str) -> Browser {
        match browser.to_lowercase().as_str() {
            "firefox" => Browser::Firefox,
            "safari" => Browser::Safari,
            "edge" => Browser::Edge,
            _ => Browser::Chrome,
        }
    }

    /// Parse test type string to TestType enum
    fn parse_test_type(test_type: &str) -> Result<TestType, AppError> {
        match test_type.to_lowercase().as_str() {
            "smoke" => Ok(TestType::Smoke),
            "regression" => Ok(TestType::Regression),
            "integration" => Ok(TestType::Integration),
            "performance" => Ok(TestType::Performance),
            "accessibility" => Ok(TestType::Accessibility),
            "responsive" => Ok(TestType::Responsive),
            _ => Err(AppError::ValidationError {
                field: "test_type".to_string(),
                message: format!("Unknown test type: {}", test_type),
            }),
        }
    }

    /// Create test suite based on suite name and test type filter
    fn create_test_suite(suite_name: &str, test_type_filter: &Option<String>) -> Result<E2ETestSuite, AppError> {
        let mut tests = Vec::new();

        // Add smoke tests
        if suite_name == "all" || suite_name == "smoke" {
            tests.push(E2ETest {
                name: "smoke_navigation".to_string(),
                description: "Test basic navigation functionality".to_string(),
                test_type: TestType::Smoke,
                steps: vec![
                    TestStep {
                        action: TestAction::Navigate,
                        selector: None,
                        value: Some("/".to_string()),
                        wait_time: None,
                    },
                    TestStep {
                        action: TestAction::Assert,
                        selector: Some("h1".to_string()),
                        value: None,
                        wait_time: None,
                    },
                ],
                expected_results: vec![ExpectedResult {
                    assertion_type: AssertionType::ElementExists,
                    selector: Some("h1".to_string()),
                    expected_value: None,
                    message: "Page should have h1 element".to_string(),
                }],
            });
        }

        // Add regression tests
        if suite_name == "all" || suite_name == "regression" {
            tests.push(E2ETest {
                name: "regression_data_integrity".to_string(),
                description: "Test data integrity across pages".to_string(),
                test_type: TestType::Regression,
                steps: vec![
                    TestStep {
                        action: TestAction::Navigate,
                        selector: None,
                        value: Some("/companies".to_string()),
                        wait_time: None,
                    },
                    TestStep {
                        action: TestAction::Assert,
                        selector: Some(".company-list".to_string()),
                        value: None,
                        wait_time: None,
                    },
                ],
                expected_results: vec![ExpectedResult {
                    assertion_type: AssertionType::ElementExists,
                    selector: Some(".company-list".to_string()),
                    expected_value: None,
                    message: "Company list should be present".to_string(),
                }],
            });
        }

        // Filter by test type if specified
        if let Some(filter) = test_type_filter {
            let filter_type = Self::parse_test_type(filter)?;
            tests.retain(|test| std::mem::discriminant(&test.test_type) == std::mem::discriminant(&filter_type));
        }

        Ok(E2ETestSuite {
            name: suite_name.to_string(),
            tests,
            config: E2ETestConfig {
                base_url: "http://localhost:3000".to_string(),
                browsers: vec![Browser::Chrome],
                viewport: Viewport {
                    width: 1920,
                    height: 1080,
                    device_scale_factor: 1.0,
                },
                timeout: Duration::from_secs(30),
                headless: true,
                parallel: false,
                screenshot_on_failure: true,
                video_recording: false,
            },
        })
    }

    /// Save test report to file
    async fn save_report(report: &E2ETestReport, output_dir: &PathBuf) -> Result<(), AppError> {
        std::fs::create_dir_all(output_dir)?;

        let report_file = output_dir.join("e2e-report.json");
        let json_content = serde_json::to_string_pretty(report)?;
        std::fs::write(report_file, json_content)?;

        Ok(())
    }

    /// Load test results from directory
    async fn load_test_results(input_dir: &PathBuf) -> Result<E2ETestReport, AppError> {
        let report_file = input_dir.join("e2e-report.json");
        let json_content = std::fs::read_to_string(report_file)?;
        let report: E2ETestReport = serde_json::from_str(&json_content)?;
        Ok(report)
    }

    /// Generate HTML report
    async fn generate_html_report(report: &E2ETestReport, output_file: &PathBuf) -> Result<(), AppError> {
        let html_content = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>E2E Test Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .summary {{ background: #f5f5f5; padding: 20px; border-radius: 5px; }}
        .test-result {{ margin: 10px 0; padding: 10px; border-radius: 3px; }}
        .passed {{ background: #d4edda; border: 1px solid #c3e6cb; }}
        .failed {{ background: #f8d7da; border: 1px solid #f5c6cb; }}
        .skipped {{ background: #fff3cd; border: 1px solid #ffeaa7; }}
    </style>
</head>
<body>
    <h1>E2E Test Report</h1>
    <div class="summary">
        <h2>Summary</h2>
        <p>Total Tests: {}</p>
        <p>Passed: {}</p>
        <p>Failed: {}</p>
        <p>Skipped: {}</p>
        <p>Success Rate: {:.1}%</p>
        <p>Total Duration: {:.2}s</p>
    </div>
    <h2>Test Results</h2>
    {}
</body>
</html>"#,
            report.summary.total_tests,
            report.summary.passed_tests,
            report.summary.failed_tests,
            report.summary.skipped_tests,
            report.summary.success_rate,
            report.summary.total_duration.as_secs_f64(),
            report
                .results
                .iter()
                .map(|result| {
                    let class = match result.status {
                        TestStatus::Passed => "passed",
                        TestStatus::Failed => "failed",
                        TestStatus::Skipped => "skipped",
                        TestStatus::Error => "failed",
                    };
                    format!(
                        r#"<div class="test-result {}">
                        <h3>{}</h3>
                        <p>Status: {:?}</p>
                        <p>Duration: {:.2}s</p>
                        {}
                    </div>"#,
                        class,
                        result.test_name,
                        result.status,
                        result.duration.as_secs_f64(),
                        if let Some(error) = &result.error_message {
                            format!("<p>Error: {}</p>", error)
                        } else {
                            String::new()
                        }
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        );

        std::fs::write(output_file, html_content)?;
        Ok(())
    }

    /// Generate JSON report
    async fn generate_json_report(report: &E2ETestReport, output_file: &PathBuf) -> Result<(), AppError> {
        let json_content = serde_json::to_string_pretty(report)?;
        std::fs::write(output_file, json_content)?;
        Ok(())
    }

    /// Generate CSV report
    async fn generate_csv_report(report: &E2ETestReport, output_file: &PathBuf) -> Result<(), AppError> {
        let mut csv_content = String::from("Test Name,Status,Duration,Error Message\n");

        for result in &report.results {
            csv_content.push_str(&format!(
                "{},{:?},{:.2},{}\n",
                result.test_name,
                result.status,
                result.duration.as_secs_f64(),
                result.error_message.as_deref().unwrap_or("")
            ));
        }

        std::fs::write(output_file, csv_content)?;
        Ok(())
    }

    /// Print test summary to console
    fn print_test_summary(report: &E2ETestReport) {
        println!("\n📊 Test Summary:");
        println!("  Total Tests: {}", report.summary.total_tests);
        println!("  ✅ Passed: {}", report.summary.passed_tests);
        println!("  ❌ Failed: {}", report.summary.failed_tests);
        println!("  ⏭️  Skipped: {}", report.summary.skipped_tests);
        println!("  📈 Success Rate: {:.1}%", report.summary.success_rate);
        println!(
            "  ⏱️  Total Duration: {:.2}s",
            report.summary.total_duration.as_secs_f64()
        );

        if report.summary.failed_tests > 0 {
            println!("\n❌ Failed Tests:");
            for result in &report.results {
                if matches!(result.status, TestStatus::Failed) {
                    println!(
                        "  - {}: {}",
                        result.test_name,
                        result.error_message.as_deref().unwrap_or("Unknown error")
                    );
                }
            }
        }
    }
}
