# TaskTaskRevolution 🚀

**A powerful, flexible, and transparent project management tool that puts you in control.**

TaskTaskRevolution is a CLI-first project management system that combines the power of TaskJuggler with the flexibility of Hugo's template system. Built with Rust for performance and reliability, it stores everything in human-readable files while providing beautiful HTML outputs.

## ✨ Key Features

- **CLI-First Design**: Complete command-line interface for maximum efficiency
- **Human-Readable Storage**: All data stored in YAML/Markdown files
- **Beautiful HTML Output**: Hugo-inspired template system for stunning visualizations
- **Gantt Charts**: Comprehensive project timeline visualization
- **Multi-Project Support**: Manage multiple companies, projects, and teams
- **Resource Management**: Track team members, skills, and availability
- **Task Management**: Full lifecycle task tracking with dependencies
- **Transparent & Open**: No vendor lock-in, your data stays yours

## 🎯 Vision

TaskTaskRevolution aims to be the most flexible and transparent project management tool available, combining:

- **TaskJuggler's** robust project management capabilities
- **Hugo's** beautiful and flexible template system
- **Modern CLI** efficiency and power
- **Open Source** transparency and community

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/flaviogranato/tasktaskrevolution.git
cd tasktaskrevolution

# Build the project
cargo build --release

# Run the binary
./target/release/ttr --help
```

### Basic Usage

```bash
# Create a new company
ttr create company "My Company"

# Create a project
ttr create project "My Project" --company "My Company"

# Add tasks
ttr create task "Design UI" --project "My Project" --duration "2d"

# Generate HTML output
ttr build
```

## 📁 Project Structure

```
tasktaskrevolution/
├── src/                    # Rust source code
├── templates/              # HTML templates
├── dist/                   # Generated HTML output
├── data/                   # Project data (YAML files)
└── themes/                 # Custom themes
```

## 🛠️ Development

### Prerequisites

- Rust 1.70+
- Git

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run
```

## 📋 Roadmap

### Phase 1: Core Foundation
- [x] Basic CLI interface
- [x] Company and project management
- [x] Task management with dependencies
- [x] Resource management
- [x] HTML generation
- [x] Gantt chart visualization

### Phase 2: Advanced Features
- [ ] ID-based refactoring for scalability
- [ ] Advanced query engine
- [ ] Hybrid YAML+Markdown document system
- [ ] Custom themes and templates
- [ ] Advanced reporting and analytics

### Phase 3: Enterprise Features
- [ ] Integration with Jira, MS Project, Kanbanize
- [ ] Advanced automation
- [ ] Team collaboration features
- [ ] Compliance and audit tools

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Workflow

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📄 License

License information will be added soon. Please check back for updates.

## 💡 Commercial License

For commercial use and integrations, please contact us for licensing options.

## 🙏 Acknowledgments

- [TaskJuggler](http://www.taskjuggler.org/) for project management inspiration
- [Hugo](https://gohugo.io/) for template system inspiration
- [Rust](https://www.rust-lang.org/) for the amazing language and ecosystem

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/flaviogranato/tasktaskrevolution/issues)
- **Discussions**: [GitHub Discussions](https://github.com/flaviogranato/tasktaskrevolution/discussions)
- **Email**: [Your Email]

## 🌟 Star History

[![Star History Chart](https://api.star-history.com/svg?repos=flaviogranato/tasktaskrevolution&type=Date)](https://star-history.com/#flaviogranato/tasktaskrevolution&Date)

---

**Made with ❤️ by [Your Name]**

*TaskTaskRevolution - Revolutionizing project management, one task at a time.*
