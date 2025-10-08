//! Shell completions for TTR CLI
//! 
//! This module provides shell completion generation for various shells.

use clap::{Command, CommandFactory};
use std::io;

/// Supported shell types for completions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

impl ShellType {
    /// Parse shell type from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bash" => Some(Self::Bash),
            "zsh" => Some(Self::Zsh),
            "fish" => Some(Self::Fish),
            "powershell" | "pwsh" => Some(Self::PowerShell),
            "elvish" => Some(Self::Elvish),
            _ => None,
        }
    }
    
    /// Get all supported shell types
    pub fn all() -> Vec<Self> {
        vec![
            Self::Bash,
            Self::Zsh,
            Self::Fish,
            Self::PowerShell,
            Self::Elvish,
        ]
    }
}

/// Shell completion generator
pub struct ShellCompletionGenerator {
    command: Command,
}

impl ShellCompletionGenerator {
    /// Create a new completion generator
    pub fn new() -> Self {
        Self {
            command: crate::interface::cli::Cli::command(),
        }
    }
    
    /// Generate completions for a specific shell
    pub fn generate_completions(&self, shell: ShellType) -> Result<String, Box<dyn std::error::Error>> {
        match shell {
            ShellType::Bash => self.generate_bash_completions(),
            ShellType::Zsh => self.generate_zsh_completions(),
            ShellType::Fish => self.generate_fish_completions(),
            ShellType::PowerShell => self.generate_powershell_completions(),
            ShellType::Elvish => self.generate_elvish_completions(),
        }
    }
    
    /// Generate bash completions
    fn generate_bash_completions(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut buf = Vec::new();
        clap_complete::generate(clap_complete::shells::Bash, &mut self.command.clone(), "ttr", &mut buf);
        Ok(String::from_utf8(buf)?)
    }
    
    /// Generate zsh completions
    fn generate_zsh_completions(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut buf = Vec::new();
        clap_complete::generate(clap_complete::shells::Zsh, &mut self.command.clone(), "ttr", &mut buf);
        Ok(String::from_utf8(buf)?)
    }
    
    /// Generate fish completions
    fn generate_fish_completions(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut buf = Vec::new();
        clap_complete::generate(clap_complete::shells::Fish, &mut self.command.clone(), "ttr", &mut buf);
        Ok(String::from_utf8(buf)?)
    }
    
    /// Generate PowerShell completions
    fn generate_powershell_completions(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut buf = Vec::new();
        clap_complete::generate(clap_complete::shells::PowerShell, &mut self.command.clone(), "ttr", &mut buf);
        Ok(String::from_utf8(buf)?)
    }
    
    /// Generate Elvish completions
    fn generate_elvish_completions(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut buf = Vec::new();
        clap_complete::generate(clap_complete::shells::Elvish, &mut self.command.clone(), "ttr", &mut buf);
        Ok(String::from_utf8(buf)?)
    }
    
    /// Generate completions for all supported shells
    pub fn generate_all_completions(&self) -> Result<std::collections::HashMap<ShellType, String>, Box<dyn std::error::Error>> {
        let mut completions = std::collections::HashMap::new();
        
        for shell in ShellType::all() {
            match self.generate_completions(shell.clone()) {
                Ok(completion) => {
                    completions.insert(shell, completion);
                }
                Err(e) => {
                    eprintln!("Failed to generate completions for {:?}: {}", shell, e);
                }
            }
        }
        
        Ok(completions)
    }
    
    /// Save completions to files
    pub fn save_completions_to_files(&self, output_dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(output_dir)?;
        
        let completions = self.generate_all_completions()?;
        
        for (shell, content) in completions {
            let filename = match shell {
                ShellType::Bash => "ttr.bash",
                ShellType::Zsh => "_ttr",
                ShellType::Fish => "ttr.fish",
                ShellType::PowerShell => "ttr.ps1",
                ShellType::Elvish => "ttr.elv",
            };
            
            let file_path = output_dir.join(filename);
            std::fs::write(&file_path, content)?;
            println!("Generated completions for {:?} at {:?}", shell, file_path);
        }
        
        Ok(())
    }
    
    /// Generate installation instructions
    pub fn generate_installation_instructions(&self) -> String {
        format!(r#"# TTR Shell Completions Installation

## Bash
To install bash completions, add this to your ~/.bashrc or ~/.bash_profile:

```bash
# TTR completions
source <(ttr completions bash)
```

Or copy the generated file to your completions directory:
```bash
# Copy to system directory
sudo cp completions/ttr.bash /usr/share/bash-completion/completions/ttr

# Or copy to user directory
mkdir -p ~/.local/share/bash-completion/completions
cp completions/ttr.bash ~/.local/share/bash-completion/completions/ttr
```

## Zsh
To install zsh completions, add this to your ~/.zshrc:

```zsh
# TTR completions
source <(ttr completions zsh)
```

Or copy the generated file to your completions directory:
```zsh
# Copy to system directory
sudo cp completions/_ttr /usr/local/share/zsh/site-functions/_ttr

# Or copy to user directory
mkdir -p ~/.zsh/completions
cp completions/_ttr ~/.zsh/completions/_ttr
echo 'fpath=(~/.zsh/completions $fpath)' >> ~/.zshrc
echo 'autoload -U compinit && compinit' >> ~/.zshrc
```

## Fish
To install fish completions, copy the generated file to your completions directory:

```fish
# Copy to user directory
mkdir -p ~/.config/fish/completions
cp completions/ttr.fish ~/.config/fish/completions/ttr.fish
```

## PowerShell
To install PowerShell completions, copy the generated file to your completions directory:

```powershell
# Copy to user directory
$completionsDir = "$env:USERPROFILE\Documents\WindowsPowerShell\Modules\TTR\Completions"
New-Item -ItemType Directory -Force -Path $completionsDir
Copy-Item completions/ttr.ps1 $completionsDir\ttr.ps1
```

## Elvish
To install Elvish completions, copy the generated file to your completions directory:

```elvish
# Copy to user directory
mkdir -p ~/.elvish/lib
cp completions/ttr.elv ~/.elvish/lib/ttr.elv
```

## Verification
After installation, restart your shell and test the completions:

```bash
# Test completions
ttr <TAB>
ttr create <TAB>
ttr list <TAB>
```

## Troubleshooting
If completions are not working:

1. Make sure the completion files are in the correct location
2. Restart your shell after installation
3. Check that your shell is configured to load completions
4. Verify the file permissions are correct

For more help, see the TTR documentation or run:
```bash
ttr completions --help
```
"#)
    }
}

/// Completion command handler
pub struct CompletionCommandHandler;

impl CompletionCommandHandler {
    /// Handle completion command
    pub fn handle_completion_command(shell: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let generator = ShellCompletionGenerator::new();
        
        if let Some(shell_str) = shell {
            let shell_type = ShellType::from_str(&shell_str)
                .ok_or_else(|| format!("Unsupported shell: {}", shell_str))?;
            
            let completions = generator.generate_completions(shell_type)?;
            print!("{}", completions);
        } else {
            // Show available shells
            println!("Available shells:");
            for shell in ShellType::all() {
                println!("  - {:?}", shell);
            }
            println!("\nUsage: ttr completions <shell>");
        }
        
        Ok(())
    }
    
    /// Handle completion installation command
    pub fn handle_install_command(output_dir: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let generator = ShellCompletionGenerator::new();
        let output_path = output_dir
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("completions"));
        
        generator.save_completions_to_files(&output_path)?;
        
        println!("Completions generated successfully!");
        println!("Output directory: {:?}", output_path);
        println!("\n{}", generator.generate_installation_instructions());
        
        Ok(())
    }
    
    /// Handle completion help command
    pub fn handle_help_command() -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", ShellCompletionGenerator::new().generate_installation_instructions());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shell_type_parsing() {
        assert_eq!(ShellType::from_str("bash"), Some(ShellType::Bash));
        assert_eq!(ShellType::from_str("zsh"), Some(ShellType::Zsh));
        assert_eq!(ShellType::from_str("fish"), Some(ShellType::Fish));
        assert_eq!(ShellType::from_str("powershell"), Some(ShellType::PowerShell));
        assert_eq!(ShellType::from_str("elvish"), Some(ShellType::Elvish));
        assert_eq!(ShellType::from_str("unknown"), None);
    }
    
    #[test]
    fn test_completion_generator_creation() {
        let generator = ShellCompletionGenerator::new();
        assert!(!generator.command.get_name().is_empty());
    }
    
    #[test]
    fn test_all_shell_types() {
        let shells = ShellType::all();
        assert_eq!(shells.len(), 5);
        assert!(shells.contains(&ShellType::Bash));
        assert!(shells.contains(&ShellType::Zsh));
        assert!(shells.contains(&ShellType::Fish));
        assert!(shells.contains(&ShellType::PowerShell));
        assert!(shells.contains(&ShellType::Elvish));
    }
}
