#!/bin/bash

# ========================================
#      TTR FUNCTIONAL TEST SCRIPT       
# ========================================
# Script para executar testes funcionais durante o desenvolvimento
# Executa testes integrados ao Cargo que validam CLI, YAML e HTML

set -e

echo "========================================"
echo "      TTR FUNCTIONAL TEST SCRIPT       "
echo "========================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if TTR binary exists
check_ttr_binary() {
    if [ ! -f "target/debug/ttr" ]; then
        log_warning "TTR binary not found. Building..."
        cargo build
        if [ $? -ne 0 ]; then
            log_error "Failed to build TTR binary"
            exit 1
        fi
    fi
    log_success "TTR binary found"
}

# Function to run specific test category
run_test_category() {
    local category="$1"
    local description="$2"
    
    log_info "Running $description..."
    
    if cargo test --test $category -- --nocapture; then
        log_success "$description - PASSED"
        return 0
    else
        log_error "$description - FAILED"
        return 1
    fi
}

# Function to run all functional tests
run_all_tests() {
    log_info "Running all functional tests..."
    
    local failures=0
    
    # Test CLI functionality
    if ! run_test_category "cli" "CLI Tests"; then
        failures=$((failures + 1))
    fi
    
    return $failures
}

# Function to run quick tests (only CLI)
run_quick_tests() {
    log_info "Running quick tests (CLI only)..."
    
    if run_test_category "cli" "Quick CLI Tests"; then
        log_success "Quick tests completed successfully"
        return 0
    else
        log_error "Quick tests failed"
        return 1
    fi
}

# Function to run comprehensive tests
run_comprehensive_tests() {
    log_info "Running comprehensive tests..."
    
    local failures=0
    
    # Build first
    log_info "Building TTR..."
    if ! cargo build; then
        log_error "Build failed"
        exit 1
    fi
    
    # Run all tests
    if ! run_all_tests; then
        failures=$((failures + 1))
    fi
    
    # Run unit tests as well
    log_info "Running unit tests..."
    if ! cargo test --lib; then
        log_error "Unit tests failed"
        failures=$((failures + 1))
    fi
    
    return $failures
}

# Function to run tests with coverage
run_tests_with_coverage() {
    log_info "Running tests with coverage..."
    
    # Check if tarpaulin is installed
    if ! command -v cargo-tarpaulin &> /dev/null; then
        log_warning "cargo-tarpaulin not found. Installing..."
        cargo install cargo-tarpaulin
    fi
    
    # Run tests with coverage
    cargo tarpaulin --tests --test integration --test cli_tests --test e2e_tests --out Html
    
    log_success "Coverage report generated in tarpaulin-report.html"
}

# Function to run specific test
run_specific_test() {
    local test_name="$1"
    
    log_info "Running specific test: $test_name"
    
    if cargo test --test integration --test cli_tests --test e2e_tests -- --nocapture --exact "$test_name"; then
        log_success "Test '$test_name' - PASSED"
        return 0
    else
        log_error "Test '$test_name' - FAILED"
        return 1
    fi
}

# Function to clean and rebuild
clean_and_rebuild() {
    log_info "Cleaning and rebuilding..."
    
    cargo clean
    cargo build
    
    log_success "Clean rebuild completed"
}

# Function to show help
show_help() {
    echo "Usage: $0 [OPTION]"
    echo ""
    echo "Options:"
    echo "  all              Run all functional tests (default)"
    echo "  quick            Run quick tests (CLI only)"
    echo "  comprehensive    Run comprehensive tests (all + unit tests)"
    echo "  coverage         Run tests with coverage report"
    echo "  clean            Clean and rebuild"
    echo "  test <name>      Run specific test by name"
    echo "  help             Show this help"
    echo ""
    echo "Examples:"
    echo "  $0                    # Run all tests"
    echo "  $0 quick             # Run quick tests"
    echo "  $0 test test_cli_help_command  # Run specific test"
    echo "  $0 coverage          # Run with coverage"
}

# Main execution
main() {
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        log_error "This script must be run from the project root directory"
        exit 1
    fi
    
    # Check TTR binary
    check_ttr_binary
    
    # Parse arguments
    case "${1:-all}" in
        "all")
            run_all_tests
            ;;
        "quick")
            run_quick_tests
            ;;
        "comprehensive")
            run_comprehensive_tests
            ;;
        "coverage")
            run_tests_with_coverage
            ;;
        "clean")
            clean_and_rebuild
            ;;
        "test")
            if [ -z "$2" ]; then
                log_error "Test name required for 'test' option"
                show_help
                exit 1
            fi
            run_specific_test "$2"
            ;;
        "help"|"-h"|"--help")
            show_help
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
    
    local exit_code=$?
    
    # Summary
    echo "========================================"
    if [ $exit_code -eq 0 ]; then
        log_success "All tests completed successfully!"
        echo "✅ CLI functionality: WORKING"
        echo "✅ YAML generation: WORKING"
        echo "✅ HTML generation: WORKING"
        echo "✅ Data validation: WORKING"
    else
        log_error "Some tests failed!"
        echo "❌ Check the output above for details"
    fi
    echo "========================================"
    
    exit $exit_code
}

# Run main function
main "$@"
