use crate::application::search::{SearchExecutor, SearchFilter, SearchResultFormatter};
use crate::domain::shared::search_engine::{FileType, SearchOptions};
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search query/pattern
    #[arg(required = true)]
    pub query: String,

    /// Entity type to search (project, task, resource, company)
    #[arg(long)]
    pub entity_type: Option<String>,

    /// Output format (table, json, csv, list, compact, grouped, highlighted)
    #[arg(long, default_value = "table")]
    pub format: String,

    /// Case sensitive search
    #[arg(long)]
    pub case_sensitive: bool,

    /// Whole word matching
    #[arg(long)]
    pub whole_word: bool,

    /// Use regex pattern
    #[arg(long)]
    pub regex: bool,

    /// Search only in metadata (YAML frontmatter)
    #[arg(long)]
    pub metadata_only: bool,

    /// Search only in content (not metadata)
    #[arg(long)]
    pub content_only: bool,

    /// Maximum number of results
    #[arg(long)]
    pub max_results: Option<usize>,

    /// Number of context lines to show
    #[arg(long, default_value = "2")]
    pub context_lines: usize,

    /// Filter by file type
    #[arg(long)]
    pub file_type: Option<String>,

    /// Minimum score threshold
    #[arg(long)]
    pub min_score: Option<f32>,

    /// Maximum score threshold
    #[arg(long)]
    pub max_score: Option<f32>,

    /// Minimum number of matches per file
    #[arg(long)]
    pub min_matches: Option<usize>,

    /// Maximum number of matches per file
    #[arg(long)]
    pub max_matches: Option<usize>,

    /// Include path pattern
    #[arg(long)]
    pub include_path: Option<String>,

    /// Exclude path pattern
    #[arg(long)]
    pub exclude_path: Option<String>,

    /// Show search statistics
    #[arg(long)]
    pub stats: bool,

    /// Workspace path (defaults to current directory)
    #[arg(long)]
    pub workspace: Option<String>,
}

pub fn execute_search(args: SearchArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Determine workspace path
    let workspace_path = if let Some(ref workspace) = args.workspace {
        PathBuf::from(workspace)
    } else {
        std::env::current_dir()?
    };

    // Create search executor
    let executor = SearchExecutor::new(workspace_path);

    // Build search options
    let search_options = build_search_options(&args);

    // Execute search
    let results = if let Some(ref entity_type) = args.entity_type {
        let entity_type = entity_type.parse()?;
        executor.search_by_entity_type(entity_type, &args.query, search_options)?
    } else {
        executor.search(&args.query, search_options)?
    };

    // Apply filters if specified
    let filtered_results = apply_filters(&results, &args);

    // Show statistics if requested
    if args.stats {
        let stats = executor.get_search_stats(&filtered_results);
        println!("{}", SearchResultFormatter::format_stats(&stats));
        println!();
    }

    // Format and display results
    let output = format_results(&filtered_results, &args.format, &args.query)?;
    print!("{}", output);

    Ok(())
}

fn build_search_options(args: &SearchArgs) -> SearchOptions {
    SearchOptions {
        case_sensitive: args.case_sensitive,
        whole_word: args.whole_word,
        regex: args.regex,
        include_metadata: !args.content_only,
        include_content: !args.metadata_only,
        max_results: args.max_results,
        context_lines: args.context_lines,
    }
}

fn apply_filters(
    results: &[crate::domain::shared::search_engine::SearchResult],
    args: &SearchArgs,
) -> Vec<crate::domain::shared::search_engine::SearchResult> {
    let mut filter = SearchFilter::new();

    // Apply file type filter
    if let Some(file_type) = &args.file_type
        && let Ok(ft) = parse_file_type(file_type)
    {
        filter = filter.file_types(vec![ft]);
    }

    // Apply score filters
    if let Some(min_score) = args.min_score {
        filter = filter.min_score(min_score);
    }
    if let Some(max_score) = args.max_score {
        filter = filter.max_score(max_score);
    }

    // Apply match count filters
    if let Some(min_matches) = args.min_matches {
        filter = filter.min_matches(min_matches);
    }
    if let Some(max_matches) = args.max_matches {
        filter = filter.max_matches(max_matches);
    }

    // Apply path filters
    if let Some(include_path) = &args.include_path {
        filter = filter.include_path(include_path);
    }
    if let Some(exclude_path) = &args.exclude_path {
        filter = filter.exclude_path(exclude_path);
    }

    filter.apply(results)
}

fn parse_file_type(file_type: &str) -> Result<FileType, String> {
    match file_type.to_lowercase().as_str() {
        "project" => Ok(FileType::Project),
        "task" => Ok(FileType::Task),
        "resource" => Ok(FileType::Resource),
        "company" => Ok(FileType::Company),
        "config" => Ok(FileType::Config),
        "other" => Ok(FileType::Other),
        _ => Err(format!("Invalid file type: {}", file_type)),
    }
}

fn format_results(
    results: &[crate::domain::shared::search_engine::SearchResult],
    format: &str,
    pattern: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    match format.to_lowercase().as_str() {
        "table" => Ok(SearchResultFormatter::format_table(results)),
        "json" => Ok(SearchResultFormatter::format_json(results)?),
        "csv" => Ok(SearchResultFormatter::format_csv(results)),
        "list" => Ok(SearchResultFormatter::format_list(results)),
        "compact" => Ok(SearchResultFormatter::format_compact(results)),
        "grouped" => Ok(SearchResultFormatter::format_grouped(results)),
        "highlighted" => Ok(SearchResultFormatter::format_highlighted(results, pattern)),
        _ => Err(format!("Unsupported format: {}", format).into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file_type() {
        assert_eq!(parse_file_type("project").unwrap(), FileType::Project);
        assert_eq!(parse_file_type("task").unwrap(), FileType::Task);
        assert_eq!(parse_file_type("resource").unwrap(), FileType::Resource);
        assert_eq!(parse_file_type("company").unwrap(), FileType::Company);
        assert_eq!(parse_file_type("config").unwrap(), FileType::Config);
        assert_eq!(parse_file_type("other").unwrap(), FileType::Other);
        assert!(parse_file_type("invalid").is_err());
    }

    #[test]
    fn test_build_search_options() {
        let args = SearchArgs {
            query: "test".to_string(),
            entity_type: None,
            format: "table".to_string(),
            case_sensitive: true,
            whole_word: true,
            regex: false,
            metadata_only: false,
            content_only: false,
            max_results: Some(50),
            context_lines: 3,
            file_type: None,
            min_score: None,
            max_score: None,
            min_matches: None,
            max_matches: None,
            include_path: None,
            exclude_path: None,
            stats: false,
            workspace: None,
        };

        let options = build_search_options(&args);
        assert!(options.case_sensitive);
        assert!(options.whole_word);
        assert!(!options.regex);
        assert!(options.include_metadata);
        assert!(options.include_content);
        assert_eq!(options.max_results, Some(50));
        assert_eq!(options.context_lines, 3);
    }

    #[test]
    fn test_format_results() {
        let results = vec![];
        let formatted = format_results(&results, "table", "test").unwrap();
        assert_eq!(formatted, "No results found.\n");
    }
}
