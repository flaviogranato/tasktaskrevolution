# TaskTaskRevolution Workspace

Welcome to your TaskTaskRevolution workspace! This directory contains your project management data.

## Quick Start

### View your data
```bash
# List all companies
ttr ls companies

# List projects for a company
ttr ls projects --company TECH-001

# List tasks for a project
ttr ls tasks --project WEB-APP --company TECH-001

# List resources
ttr ls resources --company TECH-001
```

### Create new entities
```bash
# Create a new company
ttr new company --name "New Company" --code "NEW-001"

# Create a new project
ttr new project --name "New Project" --company NEW-001 --start-date 2024-01-01 --end-date 2024-12-31

# Create a new task
ttr new task --name "New Task" --project NEW-PROJ --company NEW-001 --start-date 2024-01-01 --due-date 2024-01-15

# Create a new resource
ttr new resource --name "Jane Doe" --type "Designer" --email "jane@example.com" --company NEW-001
```

### Generate reports
```bash
# Generate HTML reports
ttr build --output dist

# Generate task reports
ttr report generate --type task --format csv --output tasks.csv
```

### Validate your data
```bash
# Validate entire system
ttr check system

# Validate with warnings
ttr check system --include-warnings
```

## Directory Structure

```
.
├── config.yaml              # Workspace configuration
├── companies/               # Company data
│   └── TECH-001/                 # Company directory
│       ├── company.yaml    # Company manifest
│       ├── projects/       # Project data
│       │   └── web-app/   # Project directory
│       │       ├── project.yaml
│       │       └── tasks/ # Task data
│       │           └── task.yaml
│       └── resources/      # Resource data
└── README.md               # This file
```

## Next Steps

1. **Explore the examples**: Check out the generated company, project, task, and resource
2. **Create your own data**: Use the commands above to create your entities
3. **Generate reports**: Run `ttr build` to create HTML reports
4. **Validate data**: Use `ttr check system` to ensure data integrity

## Getting Help

- Run `ttr --help` for general help
- Run `ttr <command> --help` for command-specific help
- Check the documentation at: https://github.com/your-org/tasktaskrevolution

Happy project managing! 🚀
