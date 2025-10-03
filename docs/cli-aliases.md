# CLI Aliases

TaskTaskRevolution provides convenient aliases for common commands, following the Hugo-style action-first approach while maintaining full backward compatibility.

## Available Aliases

### Primary Commands

| Alias | Full Command | Description |
|-------|--------------|-------------|
| `new` | `create` | Create new entities (companies, projects, tasks, resources) |
| `ls` | `list` | List entities (companies, projects, tasks, resources) |
| `edit` | `update` | Update existing entities |
| `rm` | `delete` | Delete entities |
| `check` | `validate` | Validate system and data integrity |
| `q` | `query` | Query entities with filtering |
| `tmpl` | `template` | Template management |

### Examples

#### Creating Entities
```bash
# Using aliases
ttr new company --name "Tech Corp" --code "TECH-001"
ttr new project --name "Web App" --company "TECH-001"
ttr new task --name "Login Feature" --project "WEB-APP"
ttr new resource --name "John Doe" --type "Developer" --email "john@example.com"

# Original commands still work
ttr create company --name "Tech Corp" --code "TECH-001"
ttr create project --name "Web App" --company "TECH-001"
```

#### Listing Entities
```bash
# Using aliases
ttr ls companies
ttr ls projects --company "TECH-001"
ttr ls tasks --project "WEB-APP"
ttr ls resources --company "TECH-001"

# Original commands still work
ttr list companies
ttr list projects --company "TECH-001"
```

#### Updating Entities
```bash
# Using aliases
ttr edit project --code "WEB-APP" --name "Updated Web App"
ttr edit task --code "TASK-001" --name "Updated Task"

# Original commands still work
ttr update project --code "WEB-APP" --name "Updated Web App"
ttr update task --code "TASK-001" --name "Updated Task"
```

#### Deleting Entities
```bash
# Using aliases
ttr rm project --code "WEB-APP"
ttr rm task --code "TASK-001"
ttr rm resource --code "RES-001"

# Original commands still work
ttr delete project --code "WEB-APP"
ttr delete task --code "TASK-001"
```

#### Validation
```bash
# Using aliases
ttr check system
ttr check data-integrity
ttr check business-rules

# Original commands still work
ttr validate system
ttr validate data-integrity
ttr validate business-rules
```

#### Querying
```bash
# Using aliases
ttr q --query "status:active" --entity-type "project"
ttr q --query "company:TECH-001" --entity-type "task"

# Original commands still work
ttr query --query "status:active" --entity-type "project"
ttr query --query "company:TECH-001" --entity-type "task"
```

#### Templates
```bash
# Using aliases
ttr tmpl list
ttr tmpl show --template "microservice"
ttr tmpl create --template "data-pipeline" --name "Analytics Pipeline"

# Original commands still work
ttr template list
ttr template show --template "microservice"
ttr template create --template "data-pipeline" --name "Analytics Pipeline"
```

## Backward Compatibility

All aliases are fully backward compatible. You can use either the alias or the full command name interchangeably. The help system recognizes both forms:

```bash
# Both commands show the same help
ttr new --help
ttr create --help

# Both commands work identically
ttr ls projects
ttr list projects
```

## Benefits

1. **Faster Typing**: Shorter commands for common operations
2. **Familiar Syntax**: Follows common CLI patterns (ls, rm, etc.)
3. **Action-First**: Commands start with the action (new, ls, edit, rm)
4. **Full Compatibility**: Original commands continue to work
5. **Consistent**: All aliases follow the same naming pattern

## Getting Help

You can get help for any command using either the alias or the full name:

```bash
# Get help for create command
ttr new --help
ttr create --help

# Get help for specific subcommands
ttr new company --help
ttr create company --help
```
