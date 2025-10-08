# TTR Shell Completions

This document describes the shell completion system for TTR, which provides intelligent command and argument completion for various shells.

## Overview

TTR includes a comprehensive shell completion system that supports multiple shells and provides intelligent completion for:

- Commands and subcommands
- Command arguments and options
- File paths and entity codes
- Dynamic values (company codes, project codes, etc.)

## Supported Shells

- **Bash** - Linux, macOS, WSL
- **Zsh** - Linux, macOS, WSL
- **Fish** - Linux, macOS, WSL
- **PowerShell** - Windows, Linux, macOS
- **Elvish** - Linux, macOS, WSL

## Installation

### Automatic Installation

Use the provided installation scripts:

**Linux/macOS:**
```bash
# Install for current shell
./scripts/install-completions.sh

# Install for specific shell
./scripts/install-completions.sh --shell bash

# Install for all shells
./scripts/install-completions.sh --all
```

**Windows:**
```powershell
# Install for PowerShell
.\scripts\install-completions.ps1 -Shell powershell

# Install for all shells
.\scripts\install-completions.ps1 -All
```

### Manual Installation

#### Bash

1. Generate completions:
```bash
ttr completions bash > ttr.bash
```

2. Install system-wide:
```bash
sudo cp ttr.bash /usr/share/bash-completion/completions/ttr
```

3. Or install for user:
```bash
mkdir -p ~/.local/share/bash-completion/completions
cp ttr.bash ~/.local/share/bash-completion/completions/ttr
```

4. Add to your `~/.bashrc`:
```bash
export BASH_COMPLETION_USER_FILE="$HOME/.local/share/bash-completion/completions/ttr"
```

#### Zsh

1. Generate completions:
```bash
ttr completions zsh > _ttr
```

2. Install system-wide:
```bash
sudo cp _ttr /usr/local/share/zsh/site-functions/_ttr
```

3. Or install for user:
```bash
mkdir -p ~/.zsh/completions
cp _ttr ~/.zsh/completions/_ttr
```

4. Add to your `~/.zshrc`:
```zsh
fpath=(~/.zsh/completions $fpath)
autoload -U compinit && compinit
```

#### Fish

1. Generate completions:
```bash
ttr completions fish > ttr.fish
```

2. Install:
```bash
mkdir -p ~/.config/fish/completions
cp ttr.fish ~/.config/fish/completions/ttr.fish
```

#### PowerShell

1. Generate completions:
```powershell
ttr completions powershell > ttr.ps1
```

2. Install:
```powershell
$completionDir = "$env:USERPROFILE\Documents\WindowsPowerShell\Modules\TTR\Completions"
New-Item -ItemType Directory -Force -Path $completionDir
Copy-Item ttr.ps1 $completionDir\ttr.ps1
```

3. Add to your PowerShell profile:
```powershell
Import-Module "$env:USERPROFILE\Documents\WindowsPowerShell\Modules\TTR\Completions\ttr.ps1" -Force
```

#### Elvish

1. Generate completions:
```bash
ttr completions elvish > ttr.elv
```

2. Install:
```bash
mkdir -p ~/.elvish/lib
cp ttr.elv ~/.elvish/lib/ttr.elv
```

## Usage

After installation, restart your shell and use tab completion:

```bash
# Complete commands
ttr <TAB>
ttr create <TAB>
ttr list <TAB>

# Complete arguments
ttr create company --name <TAB>
ttr create project --company <TAB>

# Complete file paths
ttr build --output <TAB>
```

## Features

### Command Completion

- All TTR commands and subcommands
- Command aliases (e.g., `c` for `create`, `l` for `list`)
- Global options (`--verbose`, `--debug`, `--quiet`)

### Argument Completion

- Required and optional arguments
- Argument types (strings, numbers, booleans)
- File paths and directories
- Entity codes (company codes, project codes, resource codes)

### Dynamic Completion

- Company codes from existing companies
- Project codes from existing projects
- Resource codes from existing resources
- Task codes from existing tasks

### Context-Aware Completion

- Different completions based on current command
- Subcommand-specific completions
- Option-specific completions

## Configuration

### Customization

You can customize completion behavior by modifying the completion scripts or using shell-specific configuration.

### Troubleshooting

#### Completions Not Working

1. **Check Installation**: Verify that completion files are in the correct location
2. **Restart Shell**: Restart your shell after installation
3. **Check Permissions**: Ensure completion files are readable
4. **Check Shell Configuration**: Verify that your shell is configured to load completions

#### Common Issues

**Bash:**
- Ensure `bash-completion` is installed
- Check that `BASH_COMPLETION_USER_FILE` is set correctly

**Zsh:**
- Ensure `compinit` is called
- Check that completion paths are in `fpath`

**Fish:**
- Ensure completions are in `~/.config/fish/completions/`
- Check that Fish is configured to load completions

**PowerShell:**
- Ensure the module is imported
- Check that the profile is loaded

**Elvish:**
- Ensure completions are in `~/.elvish/lib/`
- Check that Elvish is configured to load completions

## Development

### Adding New Completions

To add new completions for TTR commands:

1. **Update Command Structure**: Add new commands to the CLI structure
2. **Regenerate Completions**: Run `ttr completions <shell>` to generate new completions
3. **Test Completions**: Verify that new completions work correctly
4. **Update Documentation**: Update this documentation with new features

### Testing Completions

Test completions by:

1. **Manual Testing**: Use tab completion in your shell
2. **Automated Testing**: Use shell-specific testing tools
3. **Cross-Platform Testing**: Test on different operating systems

### Contributing

When contributing to TTR completions:

1. **Follow Shell Standards**: Use standard completion patterns
2. **Test Thoroughly**: Test on multiple shells and platforms
3. **Document Changes**: Update documentation for new features
4. **Maintain Compatibility**: Ensure backward compatibility

## Advanced Usage

### Custom Completion Scripts

You can create custom completion scripts for specific use cases:

```bash
# Custom completion for company codes
_ttr_company_codes() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    COMPREPLY=($(compgen -W "$(ttr list companies --format json | jq -r '.[].code')" -- "$cur"))
}
complete -F _ttr_company_codes ttr
```

### Integration with Other Tools

TTR completions can be integrated with other tools:

- **IDE Integration**: Use completions in IDEs that support shell completion
- **Script Integration**: Use completions in shell scripts
- **Tool Integration**: Use completions with other command-line tools

## Support

For issues with shell completions:

1. **Check Documentation**: Review this documentation
2. **Test Installation**: Verify that completions are installed correctly
3. **Check Shell Configuration**: Ensure your shell is configured properly
4. **Report Issues**: Report issues to the TTR project

## Examples

### Basic Usage

```bash
# Complete commands
ttr <TAB>
# Shows: build, create, delete, init, list, migrate, query, report, serve, template, update, validate, workspace

# Complete subcommands
ttr create <TAB>
# Shows: company, project, resource, task

# Complete arguments
ttr create company --name "My Company" --code <TAB>
# Shows: MY-COMP, TECH-CORP, etc.
```

### Advanced Usage

```bash
# Complete with context
ttr create project --company TECH-CORP --name <TAB>
# Shows: "My Project", "Web App", etc.

# Complete file paths
ttr build --output <TAB>
# Shows: dist/, output/, build/, etc.

# Complete with filters
ttr list projects --company <TAB>
# Shows: TECH-CORP, MY-COMP, etc.
```

This completes the shell completions system for TTR, providing comprehensive support for all major shells and platforms.
