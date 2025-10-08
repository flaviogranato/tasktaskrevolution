#!/bin/bash
# TTR Shell Completions Installation Script
# This script installs shell completions for TTR

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${BLUE}=== $1 ===${NC}"
}

# Function to detect shell
detect_shell() {
    if [ -n "$ZSH_VERSION" ]; then
        echo "zsh"
    elif [ -n "$BASH_VERSION" ]; then
        echo "bash"
    elif [ -n "$FISH_VERSION" ]; then
        echo "fish"
    else
        echo "unknown"
    fi
}

# Function to install bash completions
install_bash() {
    print_status "Installing bash completions..."
    
    # Check if bash-completion is installed
    if ! command -v bash-completion &> /dev/null; then
        print_warning "bash-completion not found. Installing..."
        
        if command -v apt-get &> /dev/null; then
            sudo apt-get update && sudo apt-get install -y bash-completion
        elif command -v yum &> /dev/null; then
            sudo yum install -y bash-completion
        elif command -v brew &> /dev/null; then
            brew install bash-completion
        else
            print_error "Cannot install bash-completion automatically. Please install it manually."
            return 1
        fi
    fi
    
    # Generate and install completions
    local completion_dir="/usr/share/bash-completion/completions"
    local user_completion_dir="$HOME/.local/share/bash-completion/completions"
    
    if [ -w "$completion_dir" ] || sudo -n true 2>/dev/null; then
        # System-wide installation
        ttr completions bash | sudo tee "$completion_dir/ttr" > /dev/null
        print_status "Bash completions installed system-wide"
    else
        # User installation
        mkdir -p "$user_completion_dir"
        ttr completions bash > "$user_completion_dir/ttr"
        print_status "Bash completions installed for user"
        print_warning "Add this to your ~/.bashrc:"
        echo "  export BASH_COMPLETION_USER_FILE=\"$user_completion_dir/ttr\""
    fi
}

# Function to install zsh completions
install_zsh() {
    print_status "Installing zsh completions..."
    
    local completion_dir="/usr/local/share/zsh/site-functions"
    local user_completion_dir="$HOME/.zsh/completions"
    
    if [ -w "$completion_dir" ] || sudo -n true 2>/dev/null; then
        # System-wide installation
        ttr completions zsh | sudo tee "$completion_dir/_ttr" > /dev/null
        print_status "Zsh completions installed system-wide"
    else
        # User installation
        mkdir -p "$user_completion_dir"
        ttr completions zsh > "$user_completion_dir/_ttr"
        print_status "Zsh completions installed for user"
        
        # Add to .zshrc if not already present
        if ! grep -q "fpath=(~/.zsh/completions" "$HOME/.zshrc" 2>/dev/null; then
            echo 'fpath=(~/.zsh/completions $fpath)' >> "$HOME/.zshrc"
            echo 'autoload -U compinit && compinit' >> "$HOME/.zshrc"
            print_status "Added completion paths to ~/.zshrc"
        fi
    fi
}

# Function to install fish completions
install_fish() {
    print_status "Installing fish completions..."
    
    local completion_dir="$HOME/.config/fish/completions"
    mkdir -p "$completion_dir"
    
    ttr completions fish > "$completion_dir/ttr.fish"
    print_status "Fish completions installed"
}

# Function to install PowerShell completions
install_powershell() {
    print_status "Installing PowerShell completions..."
    
    local completion_dir="$HOME/Documents/WindowsPowerShell/Modules/TTR/Completions"
    mkdir -p "$completion_dir"
    
    ttr completions powershell > "$completion_dir/ttr.ps1"
    print_status "PowerShell completions installed"
}

# Function to install Elvish completions
install_elvish() {
    print_status "Installing Elvish completions..."
    
    local completion_dir="$HOME/.elvish/lib"
    mkdir -p "$completion_dir"
    
    ttr completions elvish > "$completion_dir/ttr.elv"
    print_status "Elvish completions installed"
}

# Function to show help
show_help() {
    echo "TTR Shell Completions Installation Script"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -s, --shell SHELL    Install completions for specific shell (bash, zsh, fish, powershell, elvish)"
    echo "  -a, --all            Install completions for all supported shells"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 --shell bash      Install bash completions"
    echo "  $0 --all             Install all completions"
    echo "  $0                   Auto-detect shell and install completions"
}

# Main function
main() {
    print_header "TTR Shell Completions Installation"
    
    # Check if TTR is available
    if ! command -v ttr &> /dev/null; then
        print_error "TTR not found in PATH. Please install TTR first."
        exit 1
    fi
    
    # Parse arguments
    SHELL=""
    INSTALL_ALL=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            -s|--shell)
                SHELL="$2"
                shift 2
                ;;
            -a|--all)
                INSTALL_ALL=true
                shift
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # Auto-detect shell if not specified
    if [ -z "$SHELL" ] && [ "$INSTALL_ALL" = false ]; then
        SHELL=$(detect_shell)
        if [ "$SHELL" = "unknown" ]; then
            print_error "Could not detect shell. Please specify with --shell option."
            show_help
            exit 1
        fi
        print_status "Auto-detected shell: $SHELL"
    fi
    
    # Install completions
    if [ "$INSTALL_ALL" = true ]; then
        print_status "Installing completions for all supported shells..."
        install_bash
        install_zsh
        install_fish
        install_powershell
        install_elvish
    else
        case "$SHELL" in
            bash)
                install_bash
                ;;
            zsh)
                install_zsh
                ;;
            fish)
                install_fish
                ;;
            powershell)
                install_powershell
                ;;
            elvish)
                install_elvish
                ;;
            *)
                print_error "Unsupported shell: $SHELL"
                print_status "Supported shells: bash, zsh, fish, powershell, elvish"
                exit 1
                ;;
        esac
    fi
    
    print_header "Installation Complete"
    print_status "Completions have been installed successfully!"
    print_status "Restart your shell or run 'source ~/.bashrc' (or equivalent) to activate completions."
    print_status "Test completions by typing 'ttr <TAB>' in your shell."
}

# Run main function with all arguments
main "$@"
