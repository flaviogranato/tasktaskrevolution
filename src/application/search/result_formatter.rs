use crate::domain::shared::search_engine::{SearchResult, FileType};
use serde_json;
use std::collections::HashMap;

/// Formatador de resultados de busca
pub struct SearchResultFormatter;

impl SearchResultFormatter {
    /// Formata resultados como tabela
    pub fn format_table(results: &[SearchResult]) -> String {
        if results.is_empty() {
            return "No results found.".to_string();
        }

        let mut output = String::new();
        output.push_str("Search Results\n");
        output.push_str("==============\n\n");

        for (idx, result) in results.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", idx + 1, result.file_path.display()));
            output.push_str(&format!("   Type: {} | Score: {:.2}\n", result.file_type, result.score));
            
            if !result.matches.is_empty() {
                output.push_str("   Matches:\n");
                for mat in &result.matches {
                    output.push_str(&format!("     Line {}: {}\n", mat.line_number, mat.line_content));
                    
                    if let Some(context) = &mat.context_before
                        && !context.is_empty() {
                        output.push_str(&format!("     Context before: {}\n", context));
                    }
                    
                    if let Some(context) = &mat.context_after
                        && !context.is_empty() {
                        output.push_str(&format!("     Context after: {}\n", context));
                    }
                }
            }
            output.push('\n');
        }

        output
    }

    /// Formata resultados como JSON
    pub fn format_json(results: &[SearchResult]) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(results)
    }

    /// Formata resultados como CSV
    pub fn format_csv(results: &[SearchResult]) -> String {
        if results.is_empty() {
            return "file_path,file_type,score,matches_count".to_string();
        }

        let mut output = String::new();
        output.push_str("file_path,file_type,score,matches_count,line_number,line_content\n");

        for result in results {
            if result.matches.is_empty() {
                output.push_str(&format!(
                    "{},{},{:.2},0,,\n",
                    result.file_path.display(),
                    result.file_type,
                    result.score
                ));
            } else {
                for mat in &result.matches {
                    output.push_str(&format!(
                        "{},{},{:.2},{},{},\"{}\"\n",
                        result.file_path.display(),
                        result.file_type,
                        result.score,
                        result.matches.len(),
                        mat.line_number,
                        mat.line_content.replace('"', "\"\"")
                    ));
                }
            }
        }

        output
    }

    /// Formata resultados como lista simples
    pub fn format_list(results: &[SearchResult]) -> String {
        if results.is_empty() {
            return "No results found.".to_string();
        }

        let mut output = String::new();
        for result in results {
            output.push_str(&format!("{}\n", result.file_path.display()));
        }

        output
    }

    /// Formata resultados com destaque de matches
    pub fn format_highlighted(results: &[SearchResult], pattern: &str) -> String {
        if results.is_empty() {
            return "No results found.".to_string();
        }

        let mut output = String::new();
        output.push_str("Search Results (Highlighted)\n");
        output.push_str("============================\n\n");

        for (idx, result) in results.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", idx + 1, result.file_path.display()));
            output.push_str(&format!("   Type: {} | Score: {:.2}\n", result.file_type, result.score));
            
            if !result.matches.is_empty() {
                output.push_str("   Matches:\n");
                for mat in &result.matches {
                    let highlighted_line = Self::highlight_match(&mat.line_content, pattern, mat.match_start, mat.match_end);
                    output.push_str(&format!("     Line {}: {}\n", mat.line_number, highlighted_line));
                }
            }
            output.push('\n');
        }

        output
    }

    /// Formata estatísticas de busca
    pub fn format_stats(stats: &crate::application::search::search_executor::SearchStats) -> String {
        let mut output = String::new();
        output.push_str("Search Statistics\n");
        output.push_str("================\n");
        output.push_str(&format!("Total files: {}\n", stats.total_files));
        output.push_str(&format!("Total matches: {}\n", stats.total_matches));
        output.push_str("\nFile types:\n");
        
        for (file_type, count) in &stats.file_types {
            output.push_str(&format!("  {}: {}\n", file_type, count));
        }

        output
    }

    /// Formata resultados agrupados por tipo de arquivo
    pub fn format_grouped(results: &[SearchResult]) -> String {
        if results.is_empty() {
            return "No results found.".to_string();
        }

        let mut grouped: HashMap<FileType, Vec<&SearchResult>> = HashMap::new();
        for result in results {
            grouped.entry(result.file_type).or_default().push(result);
        }

        let mut output = String::new();
        output.push_str("Search Results (Grouped by Type)\n");
        output.push_str("=================================\n\n");

        for (file_type, type_results) in grouped {
            output.push_str(&format!("{} Files ({} results):\n", file_type, type_results.len()));
            for result in type_results {
                output.push_str(&format!("  - {} (score: {:.2})\n", result.file_path.display(), result.score));
            }
            output.push('\n');
        }

        output
    }

    /// Destaca matches em uma linha
    fn highlight_match(line: &str, _pattern: &str, start: usize, end: usize) -> String {
        if start >= line.len() || end > line.len() || start >= end {
            return line.to_string();
        }

        let before = &line[..start];
        let matched = &line[start..end];
        let after = &line[end..];

        format!("{}{}{}{}{}", before, ">>>", matched, "<<<", after)
    }

    /// Formata resultados em formato compacto
    pub fn format_compact(results: &[SearchResult]) -> String {
        if results.is_empty() {
            return "No results found.".to_string();
        }

        let mut output = String::new();
        output.push_str(&format!("Found {} results in {} files:\n\n", 
            results.iter().map(|r| r.matches.len()).sum::<usize>(),
            results.len()
        ));

        for result in results {
            output.push_str(&format!("{} ({}) - {} matches\n", 
                result.file_path.display(),
                result.file_type,
                result.matches.len()
            ));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::shared::search_engine::SearchMatch;
    use std::path::PathBuf;

    fn create_test_result() -> SearchResult {
        SearchResult {
            file_path: PathBuf::from("test.yaml"),
            matches: vec![
                SearchMatch {
                    line_number: 1,
                    line_content: "name: Test Project".to_string(),
                    match_start: 6,
                    match_end: 11,
                    context_before: None,
                    context_after: Some("status: active".to_string()),
                }
            ],
            score: 1.5,
            file_type: FileType::Project,
        }
    }

    #[test]
    fn test_format_table() {
        let results = vec![create_test_result()];
        let formatted = SearchResultFormatter::format_table(&results);
        assert!(formatted.contains("Search Results"));
        assert!(formatted.contains("test.yaml"));
        assert!(formatted.contains("Project"));
    }

    #[test]
    fn test_format_json() {
        let results = vec![create_test_result()];
        let formatted = SearchResultFormatter::format_json(&results).unwrap();
        assert!(formatted.contains("test.yaml"));
        assert!(formatted.contains("Project"));
    }

    #[test]
    fn test_format_csv() {
        let results = vec![create_test_result()];
        let formatted = SearchResultFormatter::format_csv(&results);
        assert!(formatted.contains("file_path,file_type,score"));
        assert!(formatted.contains("test.yaml"));
    }

    #[test]
    fn test_format_list() {
        let results = vec![create_test_result()];
        let formatted = SearchResultFormatter::format_list(&results);
        assert_eq!(formatted.trim(), "test.yaml");
    }

    #[test]
    fn test_format_highlighted() {
        let results = vec![create_test_result()];
        let formatted = SearchResultFormatter::format_highlighted(&results, "Test");
        assert!(formatted.contains(">>>Test<<<"));
    }

    #[test]
    fn test_format_empty_results() {
        let results = vec![];
        let formatted = SearchResultFormatter::format_table(&results);
        assert_eq!(formatted, "No results found.\n");
    }

    #[test]
    fn test_highlight_match() {
        let line = "This is a test line";
        let highlighted = SearchResultFormatter::highlight_match(line, "test", 10, 14);
        assert_eq!(highlighted, "This is a >>>test<<< line");
    }
}
