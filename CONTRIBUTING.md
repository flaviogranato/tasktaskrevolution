# Contributing to TaskTaskRevolution

Thank you for your interest in contributing to TaskTaskRevolution! This document provides guidelines and information for contributors.

## 🎯 How to Contribute

### Reporting Issues

- Use the [GitHub Issues](https://github.com/flaviogranato/tasktaskrevolution/issues) page
- Search existing issues before creating new ones
- Use clear, descriptive titles
- Include steps to reproduce bugs
- Provide system information (OS, Rust version, etc.)

### Suggesting Features

- Use the [GitHub Discussions](https://github.com/flaviogranato/tasktaskrevolution/discussions) for feature requests
- Check existing discussions first
- Provide clear use cases and examples
- Consider the CLI-first philosophy

## 🛠️ Development Setup

### Prerequisites

- Rust 1.70 or later
- Git
- A code editor (VS Code recommended)

### Getting Started

1. **Fork the repository**
   ```bash
   # Fork on GitHub, then clone your fork
   git clone https://github.com/YOUR_USERNAME/tasktaskrevolution.git
   cd tasktaskrevolution
   ```

2. **Add upstream remote**
   ```bash
   git remote add upstream https://github.com/flaviogranato/tasktaskrevolution.git
   ```

3. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

4. **Build and test**
   ```bash
   cargo build
   cargo test
   ```

## 📝 Code Style

### Rust Guidelines

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Write comprehensive tests
- Document public APIs

### CLI Design

- Follow the `ttr [verb] [noun] <parameters[]>` pattern
- Use clear, descriptive command names
- Provide helpful error messages
- Include examples in help text

### File Organization

```
src/
├── domain/           # Business logic
├── application/      # Use cases
├── infrastructure/   # External concerns
└── interface/        # CLI interface
```

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Test Guidelines

- Write unit tests for all public functions
- Write integration tests for CLI commands
- Test error cases and edge cases
- Aim for high test coverage

## 📋 Pull Request Process

### Before Submitting

1. **Update your branch**
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run all checks**
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

3. **Update documentation** if needed

### PR Guidelines

- Use clear, descriptive titles
- Reference related issues
- Include screenshots for UI changes
- Keep PRs focused and small
- Write clear commit messages

### Commit Message Format

```
type(scope): description

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## 🏗️ Architecture

### Domain-Driven Design

- **Domain**: Core business logic
- **Application**: Use cases and workflows
- **Infrastructure**: External dependencies
- **Interface**: CLI and user interaction

### Key Principles

- **CLI-First**: All functionality accessible via CLI
- **Human-Readable**: Data stored in readable formats
- **Transparent**: No hidden data or vendor lock-in
- **Extensible**: Plugin system for customization

## 🎨 UI/UX Guidelines

### HTML Output

- Use semantic HTML
- Ensure accessibility
- Responsive design
- Clean, modern aesthetics
- Consistent with Hugo-inspired themes

### CLI Interface

- Clear, concise output
- Consistent formatting
- Helpful error messages
- Progress indicators for long operations

## 📚 Documentation

### Code Documentation

- Document all public APIs
- Use rustdoc comments
- Include examples
- Explain complex algorithms

### User Documentation

- Update README.md for new features
- Add CLI help text
- Create examples and tutorials
- Document configuration options

## 🐛 Bug Reports

### Required Information

- **OS**: Operating system and version
- **Rust Version**: `rustc --version`
- **TTR Version**: `ttr --version`
- **Steps to Reproduce**: Clear, numbered steps
- **Expected Behavior**: What should happen
- **Actual Behavior**: What actually happens
- **Screenshots**: If applicable

### Bug Report Template

```markdown
**Bug Description**
Brief description of the bug

**Steps to Reproduce**
1. Step one
2. Step two
3. Step three

**Expected Behavior**
What should happen

**Actual Behavior**
What actually happens

**Environment**
- OS: [e.g., Ubuntu 22.04]
- Rust: [e.g., 1.70.0]
- TTR: [e.g., 0.1.0]

**Additional Context**
Any other relevant information
```

## 💡 Feature Requests

### Guidelines

- Check existing issues and discussions
- Provide clear use cases
- Consider the CLI-first philosophy
- Think about implementation complexity
- Consider backward compatibility

### Feature Request Template

```markdown
**Feature Description**
Brief description of the feature

**Use Case**
Why is this feature needed?

**Proposed Solution**
How should this work?

**Alternatives Considered**
What other approaches were considered?

**Additional Context**
Any other relevant information
```

## 🏷️ Labels

We use the following labels for issues and PRs:

- `bug`: Something isn't working
- `enhancement`: New feature or request
- `documentation`: Improvements to documentation
- `good first issue`: Good for newcomers
- `help wanted`: Extra attention is needed
- `priority: high`: High priority
- `priority: medium`: Medium priority
- `priority: low`: Low priority
- `type: feature`: New feature
- `type: bug`: Bug fix
- `type: refactor`: Code refactoring
- `type: docs`: Documentation

## 📞 Getting Help

- **GitHub Discussions**: For questions and discussions
- **GitHub Issues**: For bug reports and feature requests
- **Email**: [Your Email] for private matters

## 🙏 Recognition

Contributors will be recognized in:
- README.md contributors section
- Release notes
- Project documentation

Thank you for contributing to TaskTaskRevolution! 🚀
