# TaskTaskRevolution v0.6.0

A powerful, feature-rich project management system built with Rust. TaskTaskRevolution provides comprehensive tools for managing companies, projects, tasks, and resources with advanced querying, searching, and reporting capabilities.

## 🚀 What's New in v0.6.0

### ✨ Major Features
- **🔍 Advanced Search System** - Search across all files with regex, fuzzy matching, and metadata filtering
- **📊 Query Engine** - Powerful querying system with filtering, sorting, and aggregation
- **🐚 Shell Completions** - Auto-completion support for bash, zsh, fish, and PowerShell
- **🧪 Advanced Test Suites** - Performance, stress, security, and E2E testing frameworks
- **⚡ Resource Conflict Detection** - Intelligent conflict detection for task assignments
- **📈 Enhanced Reporting** - Improved HTML reports with better data visualization

## 🏁 Quick Start

### Installation
```bash
# Install from source
cargo install --path .

# Or build locally
cargo build --release
```

### Initialize Workspace
```bash
# Initialize your workspace
ttr init --name "Your Name" --email "your@email.com" --company "Your Company"

# Or use workspace examples for quick start
ttr workspace init --with-examples
```

## 📋 Core Commands

### Company Management
```bash
# List companies
ttr list companies

# Create company
ttr create company --name "Tech Corp" --code "TECH-001"

# Describe company
ttr company describe --code TECH-001
```

### Project Management
```bash
# List projects
ttr list projects --company TECH-001

# Create project
ttr create project --name "Web Application" --company TECH-001 --start-date 2024-01-01 --end-date 2024-12-31

# Update project
ttr update project --code WEB-APP --status "In Progress"

# Describe project
ttr project describe --code WEB-APP
```

### Task Management
```bash
# List tasks
ttr list tasks --project WEB-APP --company TECH-001

# Create task
ttr create task --name "Implement Login" --project WEB-APP --company TECH-001 --start-date 2024-01-01 --due-date 2024-01-15

# Assign resource to task
ttr task assign-resource --task TASK-001 --resource DEV-001

# Link tasks
ttr task link --from TASK-001 --to TASK-002 --type "depends_on"
```

### Resource Management
```bash
# List resources
ttr list resources --company TECH-001

# Create resource
ttr create resource --name "John Doe" --type "Developer" --email "john@example.com" --company TECH-001

# Create time off
ttr resource time-off --resource DEV-001 --start-date 2024-12-25 --end-date 2024-12-31 --type "vacation"

# Deactivate resource
ttr resource deactivate --code DEV-001
```

## 🔍 Advanced Search System

The new search system provides powerful file and content searching capabilities across your entire workspace.

### Basic Search
```bash
# Search for text across all files
ttr search "login implementation"

# Search with case sensitivity
ttr search "Login" --case-sensitive

# Search using regex
ttr search "user_[a-z]+" --regex
```

### Advanced Search Options
```bash
# Search in specific entity types
ttr search "active" --entity-type project

# Search only in metadata (YAML frontmatter)
ttr search "status:active" --metadata-only

# Search only in content (not metadata)
ttr search "implementation" --content-only

# Filter by file type
ttr search "bug" --file-type task

# Set score thresholds
ttr search "feature" --min-score 0.8 --max-score 1.0

# Limit results and context
ttr search "error" --max-results 10 --context-lines 3

# Include/exclude path patterns
ttr search "test" --include-path "**/test*" --exclude-path "**/node_modules/**"

# Show search statistics
ttr search "performance" --stats
```

### Search Output Formats
```bash
# Table format (default)
ttr search "login" --format table

# JSON format
ttr search "login" --format json

# CSV format
ttr search "login" --format csv

# List format
ttr search "login" --format list

# Compact format
ttr search "login" --format compact

# Grouped by file type
ttr search "login" --format grouped

# Highlighted matches
ttr search "login" --format highlighted
```

### Search Examples
```bash
# Find all active projects
ttr search "status:active" --entity-type project --metadata-only

# Find tasks with "bug" in description
ttr search "bug" --entity-type task --content-only

# Find resources with specific email domain
ttr search "@company\.com" --entity-type resource --regex

# Find all TODO comments
ttr search "TODO|FIXME|HACK" --regex --content-only

# Find high-priority tasks
ttr search "priority:high" --entity-type task --metadata-only
```

## 📊 Query Engine

The query engine provides powerful filtering, sorting, and aggregation capabilities for your data.

### Basic Queries
```bash
# Query projects by status
ttr query --query "status = 'active'" --entity-type project

# Query tasks by priority
ttr query --query "priority = 'high'" --entity-type task

# Query resources by type
ttr query --query "resource_type = 'developer'" --entity-type resource
```

### Advanced Queries
```bash
# String contains queries
ttr query --query "name ~ 'web'" --entity-type project

# Comparison queries
ttr query --query "priority > 'medium'" --entity-type task

# Logical operators
ttr query --query "status = 'active' AND priority = 'high'" --entity-type project

# Negation
ttr query --query "NOT status = 'cancelled'" --entity-type project

# Parentheses for complex logic
ttr query --query "(status = 'active' OR status = 'pending') AND priority = 'high'" --entity-type project
```

### Query Output Formats
```bash
# Table format (default)
ttr query --query "status = 'active'" --entity-type project --format table

# JSON format
ttr query --query "status = 'active'" --entity-type project --format json
```

## 🔗 Task Dependencies

Manage complex task dependencies with support for all PMI standard dependency types.

### Dependency Types
- **FS (Finish-to-Start)**: Task B cannot start until Task A finishes
- **SS (Start-to-Start)**: Task B cannot start until Task A starts  
- **FF (Finish-to-Finish)**: Task B cannot finish until Task A finishes
- **SF (Start-to-Finish)**: Task B cannot finish until Task A starts

### Basic Commands
```bash
# Add a dependency
ttr task dependency add --predecessor TASK-1 --successor TASK-2 --dependency-type FS

# Add dependency with lag time
ttr task dependency add --predecessor TASK-1 --successor TASK-2 --dependency-type FS --lag-days 2

# Add dependency with lead time
ttr task dependency add --predecessor TASK-1 --successor TASK-2 --dependency-type FS --lead-days 1

# Add dependency with description
ttr task dependency add --predecessor TASK-1 --successor TASK-2 --dependency-type FS --description "Code review required"
```

### Dependency Management
```bash
# List all dependencies
ttr task dependency list

# List dependencies for a specific task
ttr task dependency list --task TASK-1

# List dependencies for a project
ttr task dependency list --project PROJ-1

# Remove a dependency
ttr task dependency remove --predecessor TASK-1 --successor TASK-2
```

### Validation and Analysis
```bash
# Validate dependency graph
ttr task dependency validate

# Show dependency summary
ttr task dependency summary

# Validate specific project
ttr task dependency validate --project PROJ-1
```

### YAML Configuration
Dependencies are automatically stored in task YAML files:

```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Task
metadata:
  code: TASK-2
spec:
  dependencies:
    - predecessor: TASK-1
      dependencyType: FS
      lagDays: 2
      description: "Code review required"
```

## 🧪 Test Data Management

Validate and manage your data with the new test-data command.

```bash
# Run data validation
ttr test-data run

# Generate validation report
ttr test-data report

# Validate specific entity type
ttr test-data validate --entity-type project
```

## 🐚 Shell Completions

Get auto-completion support for your shell.

### Installation
```bash
# Generate completions for bash
ttr completions --shell bash --install

# Generate completions for zsh
ttr completions --shell zsh --install

# Generate completions for fish
ttr completions --shell fish --install

# Generate completions for PowerShell
ttr completions --shell powershell --install
```

### Manual Installation
```bash
# Show installation instructions
ttr completions --help

# Generate completion file
ttr completions --shell bash > ~/.local/share/bash-completion/completions/ttr
```

## 📈 Reporting and Visualization

### Generate HTML Reports
```bash
# Build static site
ttr build --output dist --base-url "https://your-domain.com"

# Serve locally
ttr serve --port 3000 --directory dist
```

### Task Reports
```bash
# Generate task report
ttr report generate --type task --format csv --output tasks.csv

# Generate project report
ttr report generate --type project --format html --output projects.html
```

## 🔧 Validation and Maintenance

### Data Validation
```bash
# Validate entire system
ttr validate system

# Validate with warnings
ttr validate system --include-warnings

# Validate specific entity
ttr validate project --code WEB-APP
```

### Migration Tools
```bash
# Migrate data format
ttr migrate --from-version 0.5.0 --to-version 0.6.0

# Backup data
ttr migrate backup --output backup.tar.gz
```

## 📁 Directory Structure

```
.
├── config.yaml              # Workspace configuration
├── companies/               # Company data
│   └── TECH-001/           # Company directory
│       ├── company.yaml    # Company manifest
│       ├── projects/       # Project data
│       │   └── web-app/   # Project directory
│       │       ├── project.yaml
│       │       └── tasks/ # Task data
│       │           └── task.yaml
│       └── resources/      # Resource data
│           └── dev-1.yaml
├── templates/              # Project templates
├── docs/                   # Documentation
└── README.md              # This file
```

## 🚀 Performance Features

### Advanced Test Suites
- **Performance Tests**: Load testing with thousands of entities
- **Stress Tests**: Memory and CPU intensive operations
- **Security Tests**: Input validation and security checks
- **Compatibility Tests**: Cross-platform compatibility
- **E2E Tests**: End-to-end workflow testing

### Resource Conflict Detection
- Automatic detection of resource scheduling conflicts
- Vacation and time-off conflict prevention
- Task assignment validation
- Real-time conflict monitoring

## 🛠️ Development

### Building from Source
```bash
# Clone repository
git clone https://github.com/your-org/tasktaskrevolution.git
cd tasktaskrevolution

# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test performance_tests

# Run with verbose output
cargo test --verbose

# Run integration tests
cargo test --test integration
```

## 📚 Documentation

- **API Documentation**: Run `cargo doc --open` for full API docs
- **Command Reference**: Use `ttr --help` for command help
- **Search System Guide**: See `docs/search-system.md`
- **Query Engine Guide**: See `docs/query-engine.md`

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run the test suite
6. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🆘 Getting Help

- **General Help**: `ttr --help`
- **Command Help**: `ttr <command> --help`
- **Issues**: https://github.com/your-org/tasktaskrevolution/issues
- **Discussions**: https://github.com/your-org/tasktaskrevolution/discussions

## 🎯 Roadmap

- [ ] **v0.7.0**: Advanced analytics and dashboards
- [ ] **v0.8.0**: Real-time collaboration features
- [ ] **v0.9.0**: Mobile application
- [ ] **v1.0.0**: Enterprise features and integrations

---

**Happy project managing! 🚀**

*TaskTaskRevolution v0.6.0 - Built with ❤️ in Rust*