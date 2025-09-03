#!/bin/bash

# ========================================
#      TTR NAVIGATION TEST SCRIPT       
# ========================================

set -e

echo "========================================"
echo "      TTR NAVIGATION TEST SCRIPT       "
echo "========================================"

# Configuration
TEST_DIR="/tmp/teste_nav"
TTR_BINARY="./target/debug/ttr"

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

# Test function
test_navigation() {
    local test_name="$1"
    local expected_path="$2"
    local actual_path="$3"
    
    if [ "$expected_path" = "$actual_path" ]; then
        log_success "$test_name - PASSED"
        return 0
    else
        log_error "$test_name - FAILED"
        log_error "Expected: $expected_path"
        log_error "Actual: $actual_path"
        return 1
    fi
}

# HTML link validation function
validate_html_links() {
    local html_file="$1"
    local base_dir="$2"
    
    log_info "Validating links in: $html_file"
    
    # Extract all href attributes
    local links=$(grep -o 'href="[^"]*"' "$html_file" | sed 's/href="//g' | sed 's/"//g')
    
    local failed_links=0
    
    for link in $links; do
        # Skip external links
        if [[ $link == http* ]]; then
            continue
        fi
        
        # Convert relative paths to absolute
        local full_path="$base_dir/$link"
        
        # Check if file exists
        if [ ! -f "$full_path" ]; then
            log_error "Broken link: $link (file not found: $full_path)"
            failed_links=$((failed_links + 1))
        else
            log_success "Valid link: $link"
        fi
    done
    
    return $failed_links
}

# Main test execution
main() {
    log_info "Removing existing test directory: $TEST_DIR"
    rm -rf "$TEST_DIR"
    
    log_info "Creating test directory: $TEST_DIR"
    mkdir -p "$TEST_DIR"
    cd "$TEST_DIR"
    
    # Initialize TTR
    log_info "Initializing TTR"
    eval "$TTR_BINARY init --name \"Test Manager\" --email \"test@example.com\" --company-name \"Test Company\""
    
    # Create test data
    log_info "Creating test company"
    eval "$TTR_BINARY create company --name \"Tech Corp\" --code \"TECH-CORP\" --description \"Technology company\""
    
    log_info "Creating test resource"
    eval "$TTR_BINARY create resource \"John Doe\" \"Developer\" --company-code \"TECH-CORP\""
    
    log_info "Creating test project"
    eval "$TTR_BINARY create project \"Web App\" \"Web application project\" --company-code \"TECH-CORP\""
    
    log_info "Creating test task"
    eval "$TTR_BINARY create task --name \"Setup Environment\" --description \"Setup development environment\" --start-date \"2024-01-15\" --due-date \"2024-01-22\" --project-code \"proj-1\" --company-code \"TECH-CORP\""
    
    # Build HTML
    log_info "Building HTML site for local file system"
    TTR_LOCAL_BUILD=true eval "$TTR_BINARY build"
    
    # Navigation tests
    log_info "Running navigation tests..."
    
    local test_failures=0
    
    # Test 1: Logo navigation from company page
    local company_logo_link=$(grep -o 'href="[^"]*"' "public/companies/TECH-CORP/index.html" | head -1 | sed 's/href="//g' | sed 's/"//g')
    if ! test_navigation "Company page logo link" "/index.html" "$company_logo_link"; then
        test_failures=$((test_failures + 1))
    fi
    
    # Test 2: Logo navigation from project page
    local project_logo_link=$(grep -o 'href="[^"]*"' "public/companies/TECH-CORP/projects/proj-1/index.html" | head -1 | sed 's/href="//g' | sed 's/"//g')
    if ! test_navigation "Project page logo link" "/index.html" "$project_logo_link"; then
        test_failures=$((test_failures + 1))
    fi
    
    # Test 3: Dashboard navigation from company page
    local company_dashboard_link=$(grep -A2 -B2 "Dashboard" "public/companies/TECH-CORP/index.html" | grep 'href=' | sed 's/.*href="//g' | sed 's/".*//g' | head -1)
    if ! test_navigation "Company page dashboard link" "/index.html" "$company_dashboard_link"; then
        test_failures=$((test_failures + 1))
    fi
    
    # Test 4: HTML link validation
    log_info "Validating HTML links..."
    
    local link_failures=0
    
    # Validate main index
    if ! validate_html_links "public/index.html" "$TEST_DIR/public"; then
        link_failures=$((link_failures + 1))
    fi
    
    # Validate company page
    if ! validate_html_links "public/companies/TECH-CORP/index.html" "$TEST_DIR/public/companies/TECH-CORP"; then
        link_failures=$((link_failures + 1))
    fi
    
    # Validate project page
    if ! validate_html_links "public/companies/TECH-CORP/projects/proj-1/index.html" "$TEST_DIR/public/companies/TECH-CORP/projects/proj-1"; then
        link_failures=$((link_failures + 1))
    fi
    
    # Test 5: Breadcrumb navigation
    log_info "Testing breadcrumb navigation..."
    
    # Check if breadcrumb links exist and are valid
    local breadcrumb_links=$(grep -o 'href="[^"]*"' "public/companies/TECH-CORP/projects/proj-1/index.html" | grep -v "/index.html" | head -5)
    if [ -z "$breadcrumb_links" ]; then
        log_warning "No breadcrumb links found in project page"
    else
        log_success "Breadcrumb links found: $breadcrumb_links"
    fi
    
    # Summary
    echo "========================================"
    echo "        NAVIGATION TEST SUMMARY         "
    echo "========================================"
    
    if [ $test_failures -eq 0 ] && [ $link_failures -eq 0 ]; then
        log_success "All navigation tests PASSED!"
        echo "✅ Logo navigation: WORKING"
        echo "✅ Dashboard navigation: WORKING"
        echo "✅ HTML links: VALID"
        echo "✅ Breadcrumb navigation: CHECKED"
        return 0
    else
        log_error "Navigation tests FAILED!"
        echo "❌ Test failures: $test_failures"
        echo "❌ Link failures: $link_failures"
        return 1
    fi
}

# Run main function
main "$@"
