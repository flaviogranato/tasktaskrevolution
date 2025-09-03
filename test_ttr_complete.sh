#!/bin/bash

# =============================================================================
# TTR Complete Test Script
# =============================================================================
# Este script testa todas as funcionalidades do TaskTaskRevolution
# Cria um ambiente completo de teste com empresas, projetos, recursos e tarefas
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

# Function to run TTR command with logging
run_ttr() {
    local cmd="$1"
    local description="$2"
    
    log_command "Executing: $cmd"
    log_info "$description"
    
    if eval "$cmd"; then
        log_success "$description - Completed successfully"
        return 0
    else
        log_error "$description - Failed"
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
    
    local cmd="$TTR_BINARY init --name \"Test Manager\" --email \"test@example.com\" --company-name \"Test Company\""
    
    run_ttr "$cmd" "Initialize TTR repository"
    
    log_success "TTR initialization complete"
}

# Function to create companies
create_companies() {
    log_step "Creating companies"
    
    # Tech Corp
    run_ttr "$TTR_BINARY create company --name \"Tech Corp\" --code \"TECH-CORP\" --description \"Leading technology company specializing in web applications\" --tax-id \"12.345.678/0001-90\" --address \"123 Tech Street, Silicon Valley, CA 94000\" --email \"contact@techcorp.com\" --phone \"+1-555-0123\" --website \"https://techcorp.com\" --industry \"Technology\"" \
        "Create Tech Corp company"
    
    # Finance Corp
    run_ttr "$TTR_BINARY create company --name \"Finance Corp\" --code \"FINANCE-CORP\" --description \"Financial services company focused on banking solutions\" --tax-id \"98.765.432/0001-10\" --address \"456 Finance Avenue, New York, NY 10001\" --email \"info@financecorp.com\" --phone \"+1-555-0456\" --website \"https://financecorp.com\" --industry \"Financial Services\"" \
        "Create Finance Corp company"
    
    # Startup XYZ
    run_ttr "$TTR_BINARY create company --name \"Startup XYZ\" --code \"STARTUP-XYZ\" --description \"Innovative startup developing mobile applications\" --tax-id \"11.222.333/0001-44\" --address \"789 Innovation Blvd, Austin, TX 78701\" --email \"hello@startupxyz.com\" --phone \"+1-555-0789\" --website \"https://startupxyz.com\" --industry \"Technology\"" \
        "Create Startup XYZ company"
    
    log_success "All companies created successfully"
}

# Function to create resources
create_resources() {
    log_step "Creating resources"
    
    # Tech Corp Resources
    run_ttr "$TTR_BINARY create resource \
        'John Doe' 'Developer' \
        --company-code 'TECH-CORP'" \
        "Create John Doe developer for Tech Corp"
    
    run_ttr "$TTR_BINARY create resource \
        'Jane Smith' 'Developer' \
        --company-code 'TECH-CORP'" \
        "Create Jane Smith developer for Tech Corp"
    
    run_ttr "$TTR_BINARY create resource \
        'Mike Wilson' 'ProjectManager' \
        --company-code 'TECH-CORP'" \
        "Create Mike Wilson project manager for Tech Corp"
    
    # Finance Corp Resources
    run_ttr "$TTR_BINARY create resource \
        'Alice Brown' 'Developer' \
        --company-code 'FINANCE-CORP'" \
        "Create Alice Brown developer for Finance Corp"
    
    run_ttr "$TTR_BINARY create resource \
        'Bob Johnson' 'Analyst' \
        --company-code 'FINANCE-CORP'" \
        "Create Bob Johnson analyst for Finance Corp"
    
    # Startup XYZ Resources
    run_ttr "$TTR_BINARY create resource \
        'Charlie Davis' 'Developer' \
        --company-code 'STARTUP-XYZ'" \
        "Create Charlie Davis developer for Startup XYZ"
    
    run_ttr "$TTR_BINARY create resource \
        'Diana Prince' 'Designer' \
        --company-code 'STARTUP-XYZ'" \
        "Create Diana Prince designer for Startup XYZ"
    
    log_success "All resources created successfully"
}

# Function to create projects
create_projects() {
    log_step "Creating projects"
    
    # Tech Corp Projects
    run_ttr "$TTR_BINARY create project \
        'Web Application Platform' \
        'Modern web application with React frontend and Node.js backend' \
        --company-code 'TECH-CORP'" \
        "Create Web Application Platform project for Tech Corp"
    
    run_ttr "$TTR_BINARY create project \
        'Mobile App Development' \
        'Cross-platform mobile application using React Native' \
        --company-code 'TECH-CORP'" \
        "Create Mobile App Development project for Tech Corp"
    
    # Finance Corp Projects
    run_ttr "$TTR_BINARY create project \
        'Banking System Modernization' \
        'Legacy banking system modernization with microservices architecture' \
        --company-code 'FINANCE-CORP'" \
        "Create Banking System Modernization project for Finance Corp"
    
    # Startup XYZ Projects
    run_ttr "$TTR_BINARY create project \
        'E-commerce Platform' \
        'Full-stack e-commerce platform with payment integration' \
        --company-code 'STARTUP-XYZ'" \
        "Create E-commerce Platform project for Startup XYZ"
    
    log_success "All projects created successfully"
}

# Function to create tasks
create_tasks() {
    log_step "Creating tasks"
    
    # Tech Corp - Web Application Platform tasks
    run_ttr "$TTR_BINARY create task \
        --name 'Setup Development Environment' \
        --description 'Configure development environment with React, Node.js, and database' \
        --start-date '2024-01-15' \
        --due-date '2024-01-22' \
        --project-code 'web-application-platform' \
        --company-code 'TECH-CORP' \
        --assignees 'john-doe,jane-smith'" \
        "Create Setup Development Environment task"
    
    run_ttr "$TTR_BINARY create task \
        --name 'Implement User Authentication' \
        --description 'Create user authentication system with JWT tokens' \
        --start-date '2024-01-23' \
        --due-date '2024-02-05' \
        --project-code 'web-application-platform' \
        --company-code 'TECH-CORP' \
        --assignees 'john-doe'" \
        "Create Implement User Authentication task"
    
    run_ttr "$TTR_BINARY create task \
        --name 'Design Database Schema' \
        --description 'Design and implement database schema for the application' \
        --start-date '2024-01-20' \
        --due-date '2024-01-30' \
        --project-code 'web-application-platform' \
        --company-code 'TECH-CORP' \
        --assignees 'jane-smith'" \
        "Create Design Database Schema task"
    
    # Tech Corp - Mobile App Development tasks
    run_ttr "$TTR_BINARY create task \
        --name 'Mobile App UI Design' \
        --description 'Design user interface for the mobile application' \
        --start-date '2024-02-01' \
        --due-date '2024-02-15' \
        --project-code 'mobile-app-development' \
        --company-code 'TECH-CORP' \
        --assignees 'jane-smith'" \
        "Create Mobile App UI Design task"
    
    # Finance Corp - Banking System Modernization tasks
    run_ttr "$TTR_BINARY create task \
        --name 'Legacy System Analysis' \
        --description 'Analyze existing legacy banking system for migration planning' \
        --start-date '2024-02-01' \
        --due-date '2024-02-15' \
        --project-code 'banking-system-modernization' \
        --company-code 'FINANCE-CORP' \
        --assignees 'alice-brown'" \
        "Create Legacy System Analysis task"
    
    run_ttr "$TTR_BINARY create task \
        --name 'Security Audit' \
        --description 'Conduct comprehensive security audit of banking system' \
        --start-date '2024-02-16' \
        --due-date '2024-03-01' \
        --project-code 'banking-system-modernization' \
        --company-code 'FINANCE-CORP' \
        --assignees 'bob-johnson'" \
        "Create Security Audit task"
    
    # Startup XYZ - E-commerce Platform tasks
    run_ttr "$TTR_BINARY create task \
        --name 'Payment Integration' \
        --description 'Integrate payment gateway with Stripe and PayPal' \
        --start-date '2024-03-01' \
        --due-date '2024-03-15' \
        --project-code 'e-commerce-platform' \
        --company-code 'STARTUP-XYZ' \
        --assignees 'charlie-davis'" \
        "Create Payment Integration task"
    
    run_ttr "$TTR_BINARY create task \
        --name 'Product Catalog Design' \
        --description 'Design and implement product catalog with search functionality' \
        --start-date '2024-03-01' \
        --due-date '2024-03-20' \
        --project-code 'e-commerce-platform' \
        --company-code 'STARTUP-XYZ' \
        --assignees 'diana-prince'" \
        "Create Product Catalog Design task"
    
    log_success "All tasks created successfully"
}

# Function to assign additional resources to tasks
assign_resources_to_tasks() {
    log_step "Assigning additional resources to tasks"
    
    # Assign Mike Wilson (PM) to Tech Corp tasks
    run_ttr "$TTR_BINARY task assign \
        --task 'setup-development-environment' \
        --resources 'mike-wilson'" \
        "Assign Mike Wilson to Setup Development Environment task"
    
    run_ttr "$TTR_BINARY task assign \
        --task 'implement-user-authentication' \
        --resources 'mike-wilson'" \
        "Assign Mike Wilson to Implement User Authentication task"
    
    # Assign Alice Brown to Finance Corp tasks
    run_ttr "$TTR_BINARY task assign \
        --task 'security-audit' \
        --resources 'alice-brown'" \
        "Assign Alice Brown to Security Audit task"
    
    # Assign Charlie Davis to Startup XYZ tasks
    run_ttr "$TTR_BINARY task assign \
        --task 'product-catalog-design' \
        --resources 'charlie-davis'" \
        "Assign Charlie Davis to Product Catalog Design task"
    
    log_success "Additional resource assignments completed"
}

# Function to create vacation periods
create_vacations() {
    log_step "Creating vacation periods"
    
    # John Doe vacation
    run_ttr "$TTR_BINARY create vacation \
        'john-doe' \
        --start-date '2024-07-01' \
        --end-date '2024-07-15' \
        --description 'Summer vacation'" \
        "Create vacation for John Doe"
    
    # Jane Smith vacation
    run_ttr "$TTR_BINARY create vacation \
        'jane-smith' \
        --start-date '2024-12-20' \
        --end-date '2024-12-31' \
        --description 'Holiday vacation'" \
        "Create vacation for Jane Smith"
    
    log_success "Vacation periods created successfully"
}

# Function to create time-off entries
create_time_off() {
    log_step "Creating time-off entries"
    
    # Mike Wilson time-off
    run_ttr "$TTR_BINARY create time-off \
        'mike-wilson' \
        --date '2024-06-15' \
        --hours 8 \
        --description 'Personal day'" \
        "Create time-off for Mike Wilson"
    
    # Alice Brown time-off
    run_ttr "$TTR_BINARY create time-off \
        'alice-brown' \
        --date '2024-08-10' \
        --hours 4 \
        --description 'Doctor appointment'" \
        "Create time-off for Alice Brown"
    
    log_success "Time-off entries created successfully"
}

# Function to list all entities
list_all_entities() {
    log_step "Listing all entities"
    
    log_info "Listing companies:"
    run_ttr "$TTR_BINARY list companies" "List all companies"
    
    log_info "Listing projects:"
    run_ttr "$TTR_BINARY list projects" "List all projects"
    
    log_info "Listing resources:"
    run_ttr "$TTR_BINARY list resources" "List all resources"
    
    log_info "Listing tasks:"
    run_ttr "$TTR_BINARY list tasks" "List all tasks"
    
    log_success "All entities listed successfully"
}

# Function to validate system
validate_system() {
    log_step "Validating system"
    
    run_ttr "$TTR_BINARY validate" "Validate system integrity"
    
    log_success "System validation completed"
}

# Function to generate reports
generate_reports() {
    log_step "Generating reports"
    
    run_ttr "$TTR_BINARY report vacation" "Generate vacation report"
    run_ttr "$TTR_BINARY report task" "Generate task report"
    
    log_success "Reports generated successfully"
}

# Function to build HTML site
build_html_site() {
    log_step "Building HTML site"
    
    run_ttr "$TTR_BINARY build" "Build HTML site"
    
    log_success "HTML site built successfully"
    
    # List generated files
    log_info "Generated HTML files:"
    find public -name "*.html" | sort | while read file; do
        log_info "  - $file"
    done
}

# Function to show final summary
show_summary() {
    log_step "Test Summary"
    
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}        TTR COMPLETE TEST SUMMARY        ${NC}"
    echo -e "${GREEN}========================================${NC}"
    
    log_info "Test directory: $TEST_DIR"
    log_info "Companies created: 3"
    log_info "Resources created: 8"
    log_info "Projects created: 4"
    log_info "Tasks created: 8"
    log_info "Vacations created: 2"
    log_info "Time-off entries created: 2"
    
    echo -e "${GREEN}========================================${NC}"
    log_success "All TTR functionality tested successfully!"
    echo -e "${GREEN}========================================${NC}"
    
    log_info "You can now:"
    log_info "1. Navigate to $TEST_DIR/public to view the generated HTML"
    log_info "2. Run individual TTR commands to test specific functionality"
    log_info "3. Modify the test data and re-run the script"
}

# Main execution function
main() {
    echo -e "${PURPLE}========================================${NC}"
    echo -e "${PURPLE}     TTR COMPLETE TEST SCRIPT          ${NC}"
    echo -e "${PURPLE}========================================${NC}"
    
    # Check prerequisites
    check_ttr_binary
    
    # Setup environment
    setup_test_environment
    
    # Initialize TTR
    initialize_ttr
    
    # Create entities
    create_companies
    create_resources
    create_projects
    create_tasks
    
    # Assign resources
    assign_resources_to_tasks
    
    # Create time-off entries
    create_vacations
    create_time_off
    
    # List and validate
    list_all_entities
    validate_system
    
    # Generate reports and build site
    generate_reports
    build_html_site
    
    # Show summary
    show_summary
}

# Run main function
main "$@"

