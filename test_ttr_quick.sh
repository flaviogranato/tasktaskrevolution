#!/bin/bash

# =============================================================================
# TTR Quick Test Script
# =============================================================================
# Script rápido para testar funcionalidades básicas do TTR
# =============================================================================

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configuration
TEST_DIR="/tmp/teste"
TTR_BINARY="/home/flavio/projects/tasktaskrevolution/target/debug/ttr"

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_step() { echo -e "${PURPLE}[STEP]${NC} $1"; }
log_command() { echo -e "${CYAN}[CMD]${NC} $1"; }

run_ttr() {
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

main() {
    echo -e "${PURPLE}========================================${NC}"
    echo -e "${PURPLE}      TTR QUICK TEST SCRIPT            ${NC}"
    echo -e "${PURPLE}========================================${NC}"
    
    # Setup
    if [ -d "$TEST_DIR" ]; then
        log_info "Removing existing test directory: $TEST_DIR"
        rm -rf "$TEST_DIR"
    fi
    
    log_info "Creating test directory: $TEST_DIR"
    mkdir -p "$TEST_DIR"
    cd "$TEST_DIR"
    
    # Initialize
    log_step "Initializing TTR"
    run_ttr "$TTR_BINARY init --name \"Test Manager\" --email \"test@example.com\" --company-name \"Test Company\"" \
        "Initialize TTR repository"
    
    # Create company
    log_step "Creating company"
    run_ttr "$TTR_BINARY create company --name \"Tech Corp\" --code \"TECH-CORP\" --description \"Technology company\"" \
        "Create Tech Corp company"
    
    # Create resource
    log_step "Creating resource"
    run_ttr "$TTR_BINARY create resource \"John Doe\" \"Developer\" --company-code \"TECH-CORP\"" \
        "Create John Doe developer"
    
    # Create project
    log_step "Creating project"
    run_ttr "$TTR_BINARY create project \"Web App\" \"Web application project\" --company-code \"TECH-CORP\"" \
        "Create Web App project"
    
    # Create task
    log_step "Creating task"
    run_ttr "$TTR_BINARY create task --name \"Setup Environment\" --description \"Setup development environment\" --start-date \"2024-01-15\" --due-date \"2024-01-22\" --project-code \"proj-1\" --company-code \"TECH-CORP\"" \
        "Create Setup Environment task"
    
    # List entities
    log_step "Listing entities"
    run_ttr "$TTR_BINARY list projects" "List projects"
    run_ttr "$TTR_BINARY list resources" "List resources"
    run_ttr "$TTR_BINARY list tasks" "List tasks"
    
    # Build HTML
    log_step "Building HTML site"
    run_ttr "$TTR_BINARY build" "Build HTML site"
    
    # Summary
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}        TTR QUICK TEST SUMMARY          ${NC}"
    echo -e "${GREEN}========================================${NC}"
    log_success "TTR quick test completed!"
    log_info "Test directory: $TEST_DIR"
    log_info "Check the public/ directory for generated HTML files"
    echo -e "${GREEN}========================================${NC}"
}

main "$@"
