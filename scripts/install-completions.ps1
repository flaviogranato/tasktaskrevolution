# TTR Shell Completions Installation Script for PowerShell
# This script installs shell completions for TTR on Windows

param(
    [string]$Shell = "",
    [switch]$All,
    [switch]$Help
)

# Function to print colored output
function Write-Status {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

function Write-Header {
    param([string]$Message)
    Write-Host "=== $Message ===" -ForegroundColor Blue
}

# Function to show help
function Show-Help {
    Write-Host "TTR Shell Completions Installation Script for PowerShell"
    Write-Host ""
    Write-Host "Usage: .\install-completions.ps1 [OPTIONS]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -Shell SHELL    Install completions for specific shell (powershell, bash, zsh, fish, elvish)"
    Write-Host "  -All            Install completions for all supported shells"
    Write-Host "  -Help           Show this help message"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\install-completions.ps1 -Shell powershell    Install PowerShell completions"
    Write-Host "  .\install-completions.ps1 -All                  Install all completions"
    Write-Host "  .\install-completions.ps1                       Auto-detect and install completions"
}

# Function to install PowerShell completions
function Install-PowerShellCompletions {
    Write-Status "Installing PowerShell completions..."
    
    $completionDir = "$env:USERPROFILE\Documents\WindowsPowerShell\Modules\TTR\Completions"
    New-Item -ItemType Directory -Force -Path $completionDir | Out-Null
    
    ttr completions powershell | Out-File -FilePath "$completionDir\ttr.ps1" -Encoding UTF8
    Write-Status "PowerShell completions installed at: $completionDir"
    
    # Add to PowerShell profile if not already present
    $profilePath = $PROFILE
    if (-not (Test-Path $profilePath)) {
        New-Item -ItemType File -Force -Path $profilePath | Out-Null
    }
    
    $profileContent = Get-Content $profilePath -Raw
    if ($profileContent -notmatch "TTR Completions") {
        Add-Content -Path $profilePath -Value "`n# TTR Completions`nImport-Module '$completionDir\ttr.ps1' -Force"
        Write-Status "Added TTR completions to PowerShell profile"
    }
}

# Function to install Bash completions (for WSL or Git Bash)
function Install-BashCompletions {
    Write-Status "Installing Bash completions..."
    
    $completionDir = "$env:USERPROFILE\.local\share\bash-completion\completions"
    New-Item -ItemType Directory -Force -Path $completionDir | Out-Null
    
    ttr completions bash | Out-File -FilePath "$completionDir\ttr" -Encoding UTF8
    Write-Status "Bash completions installed at: $completionDir"
    
    Write-Warning "Add this to your ~/.bashrc:"
    Write-Host "  export BASH_COMPLETION_USER_FILE=`"$completionDir\ttr`""
}

# Function to install Zsh completions (for WSL or Git Bash)
function Install-ZshCompletions {
    Write-Status "Installing Zsh completions..."
    
    $completionDir = "$env:USERPROFILE\.zsh\completions"
    New-Item -ItemType Directory -Force -Path $completionDir | Out-Null
    
    ttr completions zsh | Out-File -FilePath "$completionDir\_ttr" -Encoding UTF8
    Write-Status "Zsh completions installed at: $completionDir"
    
    # Add to .zshrc if it exists
    $zshrcPath = "$env:USERPROFILE\.zshrc"
    if (Test-Path $zshrcPath) {
        $zshrcContent = Get-Content $zshrcPath -Raw
        if ($zshrcContent -notmatch "fpath=\(~/.zsh/completions") {
            Add-Content -Path $zshrcPath -Value "`nfpath=(~/.zsh/completions `$fpath)`nautoload -U compinit && compinit"
            Write-Status "Added completion paths to ~/.zshrc"
        }
    }
}

# Function to install Fish completions
function Install-FishCompletions {
    Write-Status "Installing Fish completions..."
    
    $completionDir = "$env:USERPROFILE\.config\fish\completions"
    New-Item -ItemType Directory -Force -Path $completionDir | Out-Null
    
    ttr completions fish | Out-File -FilePath "$completionDir\ttr.fish" -Encoding UTF8
    Write-Status "Fish completions installed at: $completionDir"
}

# Function to install Elvish completions
function Install-ElvishCompletions {
    Write-Status "Installing Elvish completions..."
    
    $completionDir = "$env:USERPROFILE\.elvish\lib"
    New-Item -ItemType Directory -Force -Path $completionDir | Out-Null
    
    ttr completions elvish | Out-File -FilePath "$completionDir\ttr.elv" -Encoding UTF8
    Write-Status "Elvish completions installed at: $completionDir"
}

# Main function
function Main {
    Write-Header "TTR Shell Completions Installation"
    
    # Check if TTR is available
    if (-not (Get-Command ttr -ErrorAction SilentlyContinue)) {
        Write-Error "TTR not found in PATH. Please install TTR first."
        exit 1
    }
    
    # Show help if requested
    if ($Help) {
        Show-Help
        return
    }
    
    # Install completions
    if ($All) {
        Write-Status "Installing completions for all supported shells..."
        Install-PowerShellCompletions
        Install-BashCompletions
        Install-ZshCompletions
        Install-FishCompletions
        Install-ElvishCompletions
    } else {
        if ([string]::IsNullOrEmpty($Shell)) {
            # Auto-detect shell (default to PowerShell on Windows)
            $Shell = "powershell"
            Write-Status "Auto-detected shell: $Shell"
        }
        
        switch ($Shell.ToLower()) {
            "powershell" { Install-PowerShellCompletions }
            "bash" { Install-BashCompletions }
            "zsh" { Install-ZshCompletions }
            "fish" { Install-FishCompletions }
            "elvish" { Install-ElvishCompletions }
            default {
                Write-Error "Unsupported shell: $Shell"
                Write-Status "Supported shells: powershell, bash, zsh, fish, elvish"
                exit 1
            }
        }
    }
    
    Write-Header "Installation Complete"
    Write-Status "Completions have been installed successfully!"
    Write-Status "Restart your shell or run 'Import-Module' to activate completions."
    Write-Status "Test completions by typing 'ttr <TAB>' in your shell."
}

# Run main function
Main
