# Query System Documentation

## Overview

The TaskTaskRevolution Query System provides a powerful and flexible way to query and filter entities (projects, tasks, resources) using a domain-specific query language. The system supports complex filtering, aggregation, sorting, and pagination operations.

## Architecture

The Query System follows Clean Architecture principles with clear separation of concerns:

- **Domain Layer**: Core query parsing and execution logic
- **Application Layer**: Use cases and business logic
- **Interface Layer**: CLI commands and user interaction

## Core Components

### 1. Query Parser (`src/domain/shared/query_parser.rs`)

Parses query strings into an Abstract Syntax Tree (AST) representation.

**Supported Operators:**
- `=` - Equal
- `!=` - Not equal
- `>` - Greater than
- `<` - Less than
- `>=` - Greater than or equal
- `<=` - Less than or equal
- `~` - Contains
- `!~` - Does not contain

**Logical Operators:**
- `AND` - Logical AND
- `OR` - Logical OR
- `NOT` - Logical NOT

**Aggregations:**
- `COUNT` - Count records
- `SUM(field)` - Sum numeric field
- `AVG(field)` - Average numeric field
- `MIN(field)` - Minimum value
- `MAX(field)` - Maximum value

### 2. Query Engine (`src/domain/shared/query_engine.rs`)

Executes queries against collections of entities that implement the `Queryable` trait.

**Features:**
- Expression evaluation
- Aggregation calculations
- Sorting and pagination
- Type-safe field access

### 3. Query Executor (`src/application/query/query_executor.rs`)

High-level use case for executing queries with repository integration.

**Supported Entity Types:**
- `Project` - Project entities
- `Task` - Task entities
- `Resource` - Resource entities

### 4. Query Builder (`src/application/query/query_builder.rs`)

Fluent API for building queries programmatically.

### 5. Query Validator (`src/application/query/query_validator.rs`)

Validates queries against entity schemas and business rules.

## Usage Examples

### Basic Filtering

```rust
// Simple filter
let query = QueryBuilder::new()
    .filter("status", "=", QueryValue::String("active".to_string()))
    .build()?;

// Complex filter with logical operators
let query = QueryBuilder::new()
    .filter("status", "=", QueryValue::String("active".to_string()))
    .filter("priority", ">", QueryValue::String("medium".to_string()))
    .build()?;
```

### Aggregations

```rust
// Count active projects
let query = QueryBuilder::new()
    .filter("status", "=", QueryValue::String("active".to_string()))
    .count()
    .build()?;

// Sum task counts
let query = QueryBuilder::new()
    .filter("status", "=", QueryValue::String("active".to_string()))
    .sum("task_count")
    .build()?;
```

### Sorting and Pagination

```rust
// Sort by name ascending, limit to 10 results
let query = QueryBuilder::new()
    .filter("status", "=", QueryValue::String("active".to_string()))
    .sort_asc("name")
    .limit(10)
    .build()?;

// Sort by priority descending with offset
let query = QueryBuilder::new()
    .filter("status", "=", QueryValue::String("active".to_string()))
    .sort_desc("priority")
    .limit(20)
    .offset(10)
    .build()?;
```

### String Queries

```rust
// Parse query string
let mut parser = QueryParser::new("status:active AND priority:high".to_string());
let query = parser.parse()?;
```

## CLI Usage

### Basic Query Command

```bash
# Query active projects
ttr query --query "status:active" --entity-type project

# Query with individual parameters
ttr query --field status --operator "=" --value active --entity-type project

# Show available fields
ttr query --entity-type project --show-fields
```

### Aggregation Queries

```bash
# Count active projects
ttr query --field status --operator "=" --value active --aggregate count

# Sum task counts
ttr query --field status --operator "=" --value active --aggregate sum --aggregate-field task_count
```

### Sorting and Pagination

```bash
# Sort by name descending
ttr query --field status --operator "=" --value active --sort name --order desc

# Paginate results
ttr query --field status --operator "=" --value active --limit 10 --offset 0
```

## Available Fields

### Project Fields
- `id`, `code`, `name`, `description`
- `status`, `priority`, `start_date`, `end_date`
- `actual_start_date`, `actual_end_date`
- `company_code`, `manager_id`, `created_by`
- `created_at`, `updated_at`
- `task_count`, `resource_count`, `is_active`, `priority_weight`

### Task Fields
- `id`, `project_code`, `code`, `name`, `description`
- `status`, `start_date`, `due_date`, `actual_end_date`
- `priority`, `category`
- `dependency_count`, `assigned_resource_count`
- `is_overdue`, `days_until_due`, `priority_weight`

### Resource Fields
- `id`, `code`, `name`, `email`, `resource_type`, `scope`
- `project_id`, `start_date`, `end_date`
- `time_off_balance`, `vacation_count`, `time_off_history_count`
- `task_assignment_count`, `active_task_count`
- `current_allocation_percentage`, `is_wip_limits_exceeded`
- `wip_status`, `is_available`

## Query Language Syntax

### Simple Filters
```
field:value
field = value
field > value
field < value
field >= value
field <= value
field ~ value
field !~ value
```

### Logical Operators
```
field1:value1 AND field2:value2
field1:value1 OR field2:value2
NOT field:value
(field1:value1 OR field2:value2) AND field3:value3
```

### Examples
```
# Active projects with high priority
status:active AND priority:high

# Projects with task count greater than 5
task_count > 5

# Projects containing "test" in name
name ~ "test"

# Not cancelled projects
NOT status:cancelled

# Complex expression
(status:active OR status:pending) AND priority:high
```

## Error Handling

The Query System provides comprehensive error handling:

- **QueryParseError**: Syntax errors in query strings
- **QueryExecutionError**: Runtime errors during execution
- **ValidationError**: Field validation errors

## Performance Considerations

- Queries are executed in-memory for optimal performance
- Large datasets should use pagination to limit memory usage
- Aggregations are calculated on filtered results only
- Sorting is performed after filtering for efficiency

## Extensibility

The Query System is designed to be extensible:

1. **New Entity Types**: Implement the `Queryable` trait
2. **New Operators**: Extend the `ComparisonOperator` enum
3. **New Aggregations**: Add to the `AggregationType` enum
4. **Custom Validators**: Extend the `QueryValidator` class

## Testing

The Query System includes comprehensive tests:

- Unit tests for individual components
- Integration tests for end-to-end functionality
- Performance tests for large datasets
- Error handling tests

Run tests with:
```bash
cargo test query
cargo test integration::query_engine_test
```
