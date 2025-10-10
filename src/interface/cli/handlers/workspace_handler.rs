use crate::interface::cli::commands::workspace::WorkspaceCommand;
use std::path::PathBuf;

pub fn handle_workspace_command(command: WorkspaceCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        WorkspaceCommand::Init {
            name,
            email,
            company_name,
            company_code,
            timezone,
            yes,
        } => execute_workspace_init(name, email, company_name, company_code, timezone, yes),
    }
}

fn execute_workspace_init(
    name: String,
    email: String,
    company_name: String,
    company_code: String,
    timezone: String,
    yes: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Initializing TaskTaskRevolution workspace...");
    
    // Check if workspace already exists
    if PathBuf::from("config.yaml").exists() {
        if !yes {
            println!("⚠️  Workspace already initialized. Use --yes to reinitialize.");
            return Ok(());
        }
        println!("🔄 Reinitializing workspace...");
    }

    // Initialize basic config
    println!("📝 Creating configuration...");
    let config_content = format!(
        r#"apiVersion: tasktaskrevolution.io/v1alpha1
kind: Config
metadata:
  name: "{}"
  email: "{}"
spec:
  timezone: "{}"
  workHours:
    start: "09:00"
    end: "18:00"
  workDays: ["monday", "tuesday", "wednesday", "thursday", "friday"]
"#,
        name, email, timezone
    );
    std::fs::write("config.yaml", config_content)?;

    // Create companies directory
    println!("📁 Creating directory structure...");
    std::fs::create_dir_all("companies")?;

    // Create example company
    println!("🏢 Creating example company...");
    let company_dir = format!("companies/{}", company_code.to_lowercase());
    std::fs::create_dir_all(&company_dir)?;
    
    let company_manifest = format!(
        r#"apiVersion: tasktaskrevolution.io/v1alpha1
kind: Company
metadata:
  id: "{}"
  code: "{}"
  name: "{}"
  createdAt: "{}"
  updatedAt: "{}"
  createdBy: "workspace-init"
spec:
  timezone: "{}"
  size: "small"
  status: "active"
"#,
        uuid7::uuid7(),
        company_code, 
        company_name, 
        chrono::Utc::now().to_rfc3339(),
        chrono::Utc::now().to_rfc3339(),
        timezone
    );
    std::fs::write(format!("{}/company.yaml", company_dir), company_manifest)?;

    // Create example project
    println!("📋 Creating example project...");
    let project_dir = format!("{}/projects", company_dir);
    std::fs::create_dir_all(&project_dir)?;
    
    let project_code = "WEB-APP";
    let project_subdir = format!("{}/{}", project_dir, project_code.to_lowercase());
    std::fs::create_dir_all(&project_subdir)?;
    
    let project_manifest = format!(
        r#"apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  code: "{}"
  name: "Web Application"
  companyCode: "{}"
spec:
  startDate: "2024-01-01"
  endDate: "2024-12-31"
  status: "active"
  description: "Example web application project"
"#,
        project_code, company_code
    );
    std::fs::write(format!("{}/project.yaml", project_subdir), project_manifest)?;

    // Create example resource
    println!("👤 Creating example resource...");
    let resource_manifest = r#"apiVersion: tasktaskrevolution.io/v1alpha1
kind: Resource
metadata:
  code: "DEV-001"
  name: "John Developer"
  status: "available"
spec:
  type: "Developer"
  email: "john@example.com"
  description: "Example developer resource"
"#.to_string();
    std::fs::write(format!("{}/resources", company_dir), resource_manifest)?;

    // Create example task
    println!("✅ Creating example task...");
    let tasks_dir = format!("{}/tasks", project_subdir);
    std::fs::create_dir_all(&tasks_dir)?;
    
    let task_manifest = format!(
        r#"apiVersion: tasktaskrevolution.io/v1alpha1
kind: Task
metadata:
  code: "TASK-001"
  name: "Implement Login Feature"
  companyCode: "{}"
  projectCode: "{}"
spec:
  startDate: "2024-01-01"
  dueDate: "2024-01-15"
  status: "planned"
  priority: "medium"
  description: "Implement user authentication and login functionality"
  assignedResources: ["DEV-001"]
"#,
        company_code, project_code
    );
    std::fs::write(format!("{}/task.yaml", tasks_dir), task_manifest)?;

    // Create README with next steps
    println!("📖 Creating workspace README...");
    let readme_content = format!(
        r#"# TaskTaskRevolution Workspace

Welcome to your TaskTaskRevolution workspace! This directory contains your project management data.

## Quick Start

### View your data
```bash
# List all companies
ttr ls companies

# List projects for a company
ttr ls projects --company {}

# List tasks for a project
ttr ls tasks --project WEB-APP --company {}

# List resources
ttr ls resources --company {}
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
│   └── {}/                 # Company directory
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
"#,
        company_code, company_code, company_code, company_code
    );
    std::fs::write("README.md", readme_content)?;

    println!();
    println!("✅ Workspace initialized successfully!");
    println!();
    println!("📁 Directory structure created:");
    println!("   ├── config.yaml");
    println!("   ├── companies/{}/", company_code.to_lowercase());
    println!("   │   ├── company.yaml");
    println!("   │   ├── projects/web-app/");
    println!("   │   │   ├── project.yaml");
    println!("   │   │   └── tasks/task.yaml");
    println!("   │   └── resources/");
    println!("   └── README.md");
    println!();
    println!("🚀 Next steps:");
    println!("   1. Explore your data: ttr ls companies");
    println!("   2. Create new entities: ttr new company --help");
    println!("   3. Generate reports: ttr build --output dist");
    println!("   4. Validate data: ttr check system");
    println!();
    println!("📖 See README.md for detailed usage instructions.");

    Ok(())
}
