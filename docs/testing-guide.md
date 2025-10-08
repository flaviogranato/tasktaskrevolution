# Testing Guide for TTR

This document provides a comprehensive guide to the testing infrastructure and test suites available in the TTR project.

## Overview

The TTR project includes a comprehensive testing infrastructure with multiple test suites designed to ensure code quality, performance, security, and compatibility.

## Test Suites

### 1. Unit Tests
- **Purpose**: Test individual components and functions in isolation
- **Location**: `src/` directory with `#[cfg(test)]` modules
- **Coverage**: Domain, Application, Infrastructure, and Interface layers
- **Execution**: `cargo test --lib`

### 2. Integration Tests
- **Purpose**: Test the interaction between different components
- **Location**: `tests/` directory
- **Categories**:
  - CLI integration tests
  - Adapter tests
  - Performance tests
  - Compatibility tests
  - E2E tests
  - Security tests
  - Regression tests

### 3. Performance Tests
- **Purpose**: Measure and validate system performance
- **Configuration**: Configurable timeouts, memory limits, and data sizes
- **Test Types**:
  - CLI command performance
  - Build performance
  - Repository operations performance
  - HTML generation performance

### 4. Stress Tests
- **Purpose**: Test system behavior under extreme conditions
- **Scenarios**:
  - Concurrent CLI commands
  - Large dataset operations
  - Memory intensive operations
  - CPU intensive operations
  - File system stress

### 5. Regression Tests
- **Purpose**: Ensure existing functionality continues to work after changes
- **Categories**:
  - CLI commands regression
  - Data validation regression
  - HTML generation regression
  - YAML parsing regression

### 6. Security Tests
- **Purpose**: Identify security vulnerabilities and issues
- **Categories**:
  - Input validation security
  - File system security
  - Data protection security
  - Command injection security
  - XSS security

### 7. Compatibility Tests
- **Purpose**: Ensure compatibility across different versions and platforms
- **Categories**:
  - Data migration compatibility
  - File format compatibility
  - CLI interface compatibility
  - API compatibility

### 8. E2E Tests
- **Purpose**: Test complete user workflows from start to finish
- **Scenarios**:
  - Complete workflow
  - Company management
  - Project management
  - Resource management
  - Task management
  - HTML generation

## Test Configuration

The test configuration is managed through `tests/test_config.yaml` and includes:

- Global test settings
- Suite-specific configurations
- CI/CD integration settings
- Test data management
- Reporting configuration

## Running Tests

### Local Development

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test cli
cargo test --test performance
cargo test --test security

# Run with verbose output
cargo test --verbose

# Run with specific features
cargo test --features test-suite
```

### CI/CD Pipeline

The project includes multiple CI/CD workflows:

1. **Basic CI** (`.github/workflows/ci.yml`)
   - Basic build and test
   - Format checking
   - Clippy linting

2. **Comprehensive CI** (`.github/workflows/comprehensive-ci.yml`)
   - Extended test suite
   - Performance testing
   - Security scanning
   - Coverage analysis

3. **Advanced CI** (`.github/workflows/advanced-ci.yml`)
   - Full test suite
   - Stress testing
   - Fuzz testing
   - Benchmark testing

4. **QA Pipeline** (`.github/workflows/qa.yml`)
   - Quality assurance
   - Navigation tests
   - Performance tests
   - Security scans

## Test Data Management

### Test Data Generation
- Automatic test data generation based on configuration
- Configurable data sizes (small, medium, large, xlarge)
- Cleanup after test execution

### Test Data Categories
- **Small**: < 100 entities
- **Medium**: 100-1000 entities
- **Large**: 1000-10000 entities
- **XLarge**: > 10000 entities

## Performance Testing

### Metrics Tracked
- Execution time
- Memory usage
- CPU usage
- Throughput
- Response time

### Performance Thresholds
- Maximum execution time: 30 seconds
- Memory limit: 512 MB
- CPU usage threshold: 80%

## Security Testing

### Security Categories
- Input validation
- File system security
- Data protection
- Authentication
- Authorization
- Injection attacks
- Path traversal
- Command injection
- XSS
- CSRF

### Security Tools
- Cargo audit for vulnerability scanning
- Semgrep for static analysis
- Custom security test suites

## Compatibility Testing

### Tested Platforms
- Linux (Ubuntu, CentOS, Debian)
- Windows (10, 11, Server)
- macOS (10.15+, 11+, 12+)

### Tested Rust Versions
- Rust 1.90
- Stable
- Beta
- Nightly

### Data Format Compatibility
- YAML
- JSON
- CSV
- HTML
- XML

## Test Reporting

### Report Formats
- HTML reports
- JSON reports
- XML reports
- JUnit reports

### Coverage Reports
- Line coverage
- Branch coverage
- Function coverage
- Region coverage

### Performance Reports
- Execution time analysis
- Memory usage analysis
- CPU usage analysis
- Throughput analysis

## Best Practices

### Writing Tests
1. **Test Naming**: Use descriptive names that explain what is being tested
2. **Test Structure**: Follow Arrange-Act-Assert pattern
3. **Test Isolation**: Each test should be independent
4. **Test Data**: Use realistic test data
5. **Error Testing**: Test both success and failure scenarios

### Test Maintenance
1. **Regular Updates**: Keep tests up to date with code changes
2. **Test Review**: Review tests during code review
3. **Test Documentation**: Document complex test scenarios
4. **Test Performance**: Monitor test execution time

### CI/CD Integration
1. **Fast Feedback**: Keep test execution time reasonable
2. **Parallel Execution**: Use parallel execution where possible
3. **Test Categorization**: Categorize tests by importance
4. **Failure Handling**: Implement proper failure handling and reporting

## Troubleshooting

### Common Issues

#### Test Failures
- Check test data and setup
- Verify test environment
- Review test logs and output
- Check for race conditions

#### Performance Issues
- Monitor resource usage
- Check for memory leaks
- Optimize test data size
- Use appropriate timeouts

#### Security Issues
- Review security test results
- Check for false positives
- Update security test patterns
- Monitor for new vulnerabilities

### Debugging Tests

```bash
# Run tests with debug output
RUST_LOG=debug cargo test

# Run specific test with output
cargo test test_name -- --nocapture

# Run tests with backtrace
RUST_BACKTRACE=1 cargo test
```

## Contributing

### Adding New Tests
1. Follow existing test patterns
2. Add appropriate test configuration
3. Update test documentation
4. Ensure tests are maintainable

### Test Review Process
1. Review test coverage
2. Check test quality
3. Verify test performance
4. Ensure test documentation

## Resources

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Cargo Test Documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Cargo Tarpaulin Documentation](https://github.com/xd009642/tarpaulin)
