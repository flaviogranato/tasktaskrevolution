#!/bin/bash

# =============================================================================
# TTR Simple Test Script
# =============================================================================
# Este script testa as funcionalidades básicas do TaskTaskRevolution
# Versão mais robusta que continua mesmo se alguns comandos falharem
# =============================================================================

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
TEST_DIR="/tmp/teste"
TTR_BINARY="/home/flavio/projects/tasktaskrevolution/target/debug/ttr"

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

log_step() {
    echo -e "${PURPLE}[STEP]${NC} $1"
}

log_command() {
    echo -e "${CYAN}[CMD]${NC} $1"
}

# Function to run TTR command with error handling
run_ttr_safe() {
    local cmd="$1"
    local description="$2"
    
    log_command "Executing: $cmd"
    log_info "$description"
    
    if eval "$cmd"; then
        log_success "$description - Completed successfully"
        return 0
    else
        log_warning "$description - Failed (continuing anyway)"
        return 1
    fi
}

# Function to check if TTR binary exists
check_ttr_binary() {
    if [ ! -f "$TTR_BINARY" ]; then
        log_error "TTR binary not found at $TTR_BINARY"
        log_info "Please build the project first with: cargo build"
        exit 1
    fi
    log_success "TTR binary found at $TTR_BINARY"
}

# Function to clean and setup test environment
setup_test_environment() {
    log_step "Setting up test environment"
    
    # Remove existing test directory
    if [ -d "$TEST_DIR" ]; then
        log_info "Removing existing test directory: $TEST_DIR"
        rm -rf "$TEST_DIR"
    fi
    
    # Create new test directory
    log_info "Creating test directory: $TEST_DIR"
    mkdir -p "$TEST_DIR"
    cd "$TEST_DIR"
    
    log_success "Test environment setup complete"
}

# Function to initialize TTR
initialize_ttr() {
    log_step "Initializing TTR"
    
    log_command "Executing: $TTR_BINARY init --name \"Test Manager\" --email \"test@example.com\" --company-name \"Test Company\""
    log_info "Initialize TTR repository"
    
    if eval "$TTR_BINARY init --name \"Test Manager\" --email \"test@example.com\" --company-name \"Test Company\""; then
        log_success "Initialize TTR repository - Completed successfully"
    else
        log_warning "Initialize TTR repository - Failed (continuing anyway)"
    fi
    
    log_success "TTR initialization complete"
}

# Function to create basic companies
create_basic_companies() {
    log_step "Creating basic companies"
    
    # Tech Corp
    run_ttr_safe "$TTR_BINARY create company --name \"Tech Corp\" --code \"TECH-CORP\" --description \"Technology company\"" \
        "Create Tech Corp company"
    
    # Finance Corp
    run_ttr_safe "$TTR_BINARY create company --name \"Finance Corp\" --code \"FINANCE-CORP\" --description \"Financial services company\"" \
        "Create Finance Corp company"
    
    log_success "Basic companies created"
}

# Function to create basic resources
create_basic_resources() {
    log_step "Creating basic resources"
    
    # Tech Corp Resources
    run_ttr_safe "$TTR_BINARY create resource \"John Doe\" \"Developer\" --company-code \"TECH-CORP\"" \
        "Create John Doe developer"
    
    run_ttr_safe "$TTR_BINARY create resource \"Jane Smith\" \"Developer\" --company-code \"TECH-CORP\"" \
        "Create Jane Smith developer"
    
    # Finance Corp Resources
    run_ttr_safe "$TTR_BINARY create resource \"Alice Brown\" \"Developer\" --company-code \"FINANCE-CORP\"" \
        "Create Alice Brown developer"
    
    log_success "Basic resources created"
}

# Function to create basic projects
create_basic_projects() {
    log_step "Creating basic projects"
    
    # Tech Corp Project
    run_ttr_safe "$TTR_BINARY create project \"Web App\" \"Web application project\" --company-code \"TECH-CORP\"" \
        "Create Web App project"
    
    # Finance Corp Project
    run_ttr_safe "$TTR_BINARY create project \"Banking System\" \"Banking system project\" --company-code \"FINANCE-CORP\"" \
        "Create Banking System project"
    
    log_success "Basic projects created"
}

# Function to create basic tasks
create_basic_tasks() {
    log_step "Creating basic tasks"
    
    # Tech Corp tasks
    run_ttr_safe "$TTR_BINARY create task --name \"Setup Environment\" --description \"Setup development environment\" --start-date \"2024-01-15\" --due-date \"2024-01-22\" --project-code \"proj-1\" --company-code \"TECH-CORP\"" \
        "Create Setup Environment task"
    
    run_ttr_safe "$TTR_BINARY create task --name \"Implement Auth\" --description \"Implement authentication\" --start-date \"2024-01-23\" --due-date \"2024-02-05\" --project-code \"proj-1\" --company-code \"TECH-CORP\"" \
        "Create Implement Auth task"
    
    # Finance Corp tasks
    run_ttr_safe "$TTR_BINARY create task --name \"System Analysis\" --description \"Analyze banking system\" --start-date \"2024-02-01\" --due-date \"2024-02-15\" --project-code \"proj-1\" --company-code \"FINANCE-CORP\"" \
        "Create System Analysis task"
    
    log_success "Basic tasks created"
}

# Function to list entities
list_entities() {
    log_step "Listing entities"
    
    log_info "Listing projects:"
    run_ttr_safe "$TTR_BINARY list projects" "List projects"
    
    log_info "Listing resources:"
    run_ttr_safe "$TTR_BINARY list resources" "List resources"
    
    log_info "Listing tasks:"
    run_ttr_safe "$TTR_BINARY list tasks" "List tasks"
    
    log_success "Entity listing completed"
}

# Function to build HTML site
build_html_site() {
    log_step "Building HTML site"
    
    run_ttr_safe "$TTR_BINARY build" "Build HTML site"
    
    if [ -d "public" ]; then
        log_success "HTML site built successfully"
        log_info "Generated HTML files:"
        find public -name "*.html" 2>/dev/null | sort | while read file; do
            log_info "  - $file"
        done
    else
        log_warning "HTML site build may have failed - public directory not found"
    fi
}

# Function to show final summary
show_summary() {
    log_step "Test Summary"
    
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}        TTR SIMPLE TEST SUMMARY         ${NC}"
    echo -e "${GREEN}========================================${NC}"
    
    log_info "Test directory: $TEST_DIR"
    
    # Count created entities
    local companies=0
    local resources=0
    local projects=0
    local tasks=0
    
    if [ -d "companies" ]; then
        companies=$(find companies -name "company.yaml" 2>/dev/null | wc -l)
        resources=$(find companies -name "*.yaml" -path "*/resources/*" 2>/dev/null | wc -l)
        projects=$(find companies -name "project.yaml" 2>/dev/null | wc -l)
        tasks=$(find companies -name "task-*.yaml" 2>/dev/null | wc -l)
    fi
    
    log_info "Companies created: $companies"
    log_info "Resources created: $resources"
    log_info "Projects created: $projects"
    log_info "Tasks created: $tasks"
    
    echo -e "${GREEN}========================================${NC}"
    log_success "TTR basic functionality test completed!"
    echo -e "${GREEN}========================================${NC}"
    
    log_info "You can now:"
    log_info "1. Navigate to $TEST_DIR to explore the created structure"
    log_info "2. Check the public/ directory for generated HTML files"
    log_info "3. Run individual TTR commands to test specific functionality"
}

# Main execution function
main() {
    echo -e "${PURPLE}========================================${NC}"
    echo -e "${PURPLE}      TTR SIMPLE TEST SCRIPT           ${NC}"
    echo -e "${PURPLE}========================================${NC}"
    
    # Check prerequisites
    check_ttr_binary
    
    # Setup environment
    setup_test_environment
    
    # Initialize TTR
    initialize_ttr
    
    # Create basic entities
    create_basic_companies
    create_basic_resources
    create_basic_projects
    create_basic_tasks
    
    # List entities
    list_entities
    
    # Build HTML site
    build_html_site
    
    # Show summary
    show_summary
}

# Run main function
main "$@"

