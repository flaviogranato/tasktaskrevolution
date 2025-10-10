# Search System Documentation

## Overview

The TaskTaskRevolution Search System provides powerful full-text search capabilities across all project files. It supports searching in both metadata (YAML frontmatter) and content, with advanced filtering and formatting options.

## Architecture

The Search System follows Clean Architecture principles with clear separation of concerns:

- **Domain Layer**: Core search engine and file processing
- **Application Layer**: Search use cases and result formatting
- **Interface Layer**: CLI commands and user interaction

## Core Components

### 1. Search Engine (`src/domain/shared/search_engine.rs`)

Core search functionality with filesystem-based indexing.

**Features:**
- Full-text search across all files
- Metadata search in YAML frontmatter
- Regex pattern matching
- Case-sensitive and case-insensitive search
- Whole word matching
- Context-aware results
- File type detection
- Relevance scoring

### 2. Search Executor (`src/application/search/search_executor.rs`)

High-level use case for executing searches with various options.

**Search Types:**
- General search across all files
- Entity-specific search (projects, tasks, resources, companies)
- Field-specific search
- Metadata-only search
- Content-only search
- Regex search

### 3. Search Result Formatter (`src/application/search/result_formatter.rs`)

Formats search results in various output formats.

**Supported Formats:**
- Table (default)
- JSON
- CSV
- List
- Compact
- Grouped
- Highlighted

### 4. Search Filter (`src/application/search/search_filter.rs`)

Advanced filtering and refinement of search results.

**Filter Options:**
- File type filtering
- Score-based filtering
- Match count filtering
- Path pattern filtering
- Exclusion patterns

## Usage Examples

### Basic Search

```rust
use task_task_revolution::application::search::{SearchExecutor, SearchOptions};

let executor = SearchExecutor::new(workspace_path);
let options = SearchOptions::default();
let results = executor.search("test project", options)?;
```

### Entity-Specific Search

```rust
// Search only in project files
let results = executor.search_by_entity_type(
    EntityType::Project,
    "active",
    options
)?;
```

### Advanced Search Options

```rust
let options = SearchOptions {
    case_sensitive: true,
    whole_word: true,
    regex: false,
    include_metadata: true,
    include_content: true,
    max_results: Some(100),
    context_lines: 3,
};
```

### Search with Filters

```rust
use task_task_revolution::application::search::{SearchFilter, SearchFilterBuilder};

// Filter by file type
let filter = SearchFilter::new()
    .file_types(vec![FileType::Project])
    .min_score(2.0)
    .build();

let filtered_results = filter.apply(&results);
```

## CLI Usage

### Basic Search Command

```bash
# Search for "test" in all files
ttr search "test"

# Search with case sensitivity
ttr search "Test" --case-sensitive

# Search with whole word matching
ttr search "test" --whole-word

# Search using regex
ttr search "project-\d{3}" --regex
```

### Entity-Specific Search

```bash
# Search only in project files
ttr search "active" --entity-type project

# Search only in task files
ttr search "planned" --entity-type task

# Search only in resource files
ttr search "developer" --entity-type resource
```

### Advanced Search Options

```bash
# Search only in metadata
ttr search "status:active" --metadata-only

# Search only in content
ttr search "description" --content-only

# Limit results
ttr search "test" --max-results 50

# Show context lines
ttr search "error" --context-lines 5
```

### Output Formats

```bash
# Table format (default)
ttr search "test" --format table

# JSON format
ttr search "test" --format json

# CSV format
ttr search "test" --format csv

# List format
ttr search "test" --format list

# Compact format
ttr search "test" --format compact

# Grouped format
ttr search "test" --format grouped

# Highlighted format
ttr search "test" --format highlighted
```

### Filtering Options

```bash
# Filter by file type
ttr search "test" --file-type project

# Filter by score
ttr search "test" --min-score 2.0 --max-score 5.0

# Filter by match count
ttr search "test" --min-matches 2 --max-matches 10

# Include/exclude path patterns
ttr search "test" --include-path "projects" --exclude-path "temp"

# Show statistics
ttr search "test" --stats
```

## File Type Detection

The Search System automatically detects file types based on their location and extension:

- **Project**: `projects/*.yaml`
- **Task**: `tasks/*.yaml`
- **Resource**: `resources/*.yaml`
- **Company**: `companies/*.yaml`
- **Config**: `config.yaml`
- **Other**: All other files

## Search Patterns

### Text Search
- Case-insensitive by default
- Supports partial matches
- Escapes special regex characters

### Regex Search
- Use `--regex` flag for regex patterns
- Full regex syntax support
- Case sensitivity controlled by `--case-sensitive`

### Whole Word Search
- Use `--whole-word` flag
- Matches complete words only
- Useful for avoiding partial matches

## Result Formatting

### Table Format
```
Search Results
==============

1. projects/test.yaml
   Type: project | Score: 2.50
   Matches:
     Line 3: name: Test Project
     Context before: status: active
     Context after: description: A test project

2. tasks/task1.yaml
   Type: task | Score: 1.80
   Matches:
     Line 2: name: Test Task
```

### JSON Format
```json
[
  {
    "file_path": "projects/test.yaml",
    "matches": [
      {
        "line_number": 3,
        "line_content": "name: Test Project",
        "match_start": 6,
        "match_end": 10,
        "context_before": "status: active",
        "context_after": "description: A test project"
      }
    ],
    "score": 2.5,
    "file_type": "Project"
  }
]
```

### Highlighted Format
```
Search Results (Highlighted)
============================

1. projects/test.yaml
   Type: project | Score: 2.50
   Matches:
     Line 3: name: >>>Test<<< Project
```

## Performance Considerations

- Search is performed on-demand (no indexing)
- Large files are processed line by line
- Memory usage scales with file size
- Use `--max-results` to limit output
- Consider using filters to narrow search scope

## Error Handling

The Search System provides comprehensive error handling:

- **SearchError::InvalidPattern**: Invalid regex patterns
- **SearchError::FileReadError**: File access errors
- **SearchError::RegexError**: Regex compilation errors
- **SearchError::NoResults**: No matches found

## Extensibility

The Search System is designed to be extensible:

1. **New File Types**: Extend the `FileType` enum
2. **New Search Options**: Add to the `SearchOptions` struct
3. **New Formatters**: Implement custom formatters
4. **New Filters**: Extend the `SearchFilter` class

## Integration with Query System

The Search System complements the Query System:

- **Query System**: Structured queries on loaded entities
- **Search System**: Full-text search across all files
- **Combined**: Use search to find files, then query to filter entities

Example workflow:
```bash
# Find files containing "test"
ttr search "test" --entity-type project

# Query the found projects
ttr query --field name --operator "~" --value "test" --entity-type project
```

## Testing

The Search System includes comprehensive tests:

- Unit tests for individual components
- Integration tests for end-to-end functionality
- Performance tests for large files
- Error handling tests

Run tests with:
```bash
cargo test search
cargo test integration::search_system_test
```

## Best Practices

1. **Use appropriate search types**:
   - Text search for general content
   - Regex search for structured patterns
   - Entity-specific search for targeted results

2. **Optimize performance**:
   - Use filters to narrow search scope
   - Limit results with `--max-results`
   - Use `--metadata-only` when appropriate

3. **Format output appropriately**:
   - Use table format for human reading
   - Use JSON format for programmatic processing
   - Use CSV format for data analysis

4. **Combine with Query System**:
   - Use search to find relevant files
   - Use query to filter and analyze entities
   - Leverage both systems for comprehensive analysis
