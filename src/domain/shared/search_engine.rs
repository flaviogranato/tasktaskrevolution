use crate::application::errors::AppError;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Resultado de uma busca
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub file_path: PathBuf,
    pub matches: Vec<SearchMatch>,
    pub score: f32,
    pub file_type: FileType,
}

/// Match individual encontrado em um arquivo
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchMatch {
    pub line_number: usize,
    pub line_content: String,
    pub match_start: usize,
    pub match_end: usize,
    pub context_before: Option<String>,
    pub context_after: Option<String>,
}

/// Tipo de arquivo
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FileType {
    Project,
    Task,
    Resource,
    Company,
    Config,
    Other,
}

/// Opções de busca
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchOptions {
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub regex: bool,
    pub include_metadata: bool,
    pub include_content: bool,
    pub max_results: Option<usize>,
    pub context_lines: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            whole_word: false,
            regex: false,
            include_metadata: true,
            include_content: true,
            max_results: Some(100),
            context_lines: 2,
        }
    }
}

/// Query de busca
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchQuery {
    pub pattern: String,
    pub file_types: Vec<FileType>,
    pub field_filters: HashMap<String, String>,
    pub options: SearchOptions,
}

/// Erro de busca
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SearchError {
    InvalidPattern(String),
    FileReadError(String),
    RegexError(String),
    NoResults,
    InvalidFileType(String),
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchError::InvalidPattern(msg) => write!(f, "Invalid search pattern: {}", msg),
            SearchError::FileReadError(msg) => write!(f, "File read error: {}", msg),
            SearchError::RegexError(msg) => write!(f, "Regex error: {}", msg),
            SearchError::NoResults => write!(f, "No results found"),
            SearchError::InvalidFileType(msg) => write!(f, "Invalid file type: {}", msg),
        }
    }
}

impl std::error::Error for SearchError {}

/// Engine de busca baseado em filesystem
pub struct SearchEngine {
    root_path: PathBuf,
}

impl SearchEngine {
    /// Cria um novo SearchEngine
    pub fn new(root_path: PathBuf) -> Self {
        Self { root_path }
    }

    /// Busca por padrão em todos os arquivos
    pub fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        let mut results = Vec::new();
        let pattern = self.compile_pattern(&query.pattern, &query.options)?;

        for entry in WalkDir::new(&self.root_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();
            let file_type = self.detect_file_type(file_path)?;

            // Filtrar por tipo de arquivo se especificado
            if !query.file_types.is_empty() && !query.file_types.contains(&file_type) {
                continue;
            }

            // Buscar no arquivo
            if let Ok(matches) = self.search_in_file(file_path, &pattern, &query.options) {
                if !matches.is_empty() {
                    let score = self.calculate_score(&matches, &query.pattern);
                    results.push(SearchResult {
                        file_path: file_path.to_path_buf(),
                        matches,
                        score,
                        file_type,
                    });
                }
            }
        }

        // Ordenar por score (maior primeiro)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Aplicar limite de resultados
        if let Some(max_results) = query.options.max_results {
            results.truncate(max_results);
        }

        if results.is_empty() {
            Err(SearchError::NoResults)
        } else {
            Ok(results)
        }
    }

    /// Busca por padrão em um arquivo específico
    pub fn search_in_file(
        &self,
        file_path: &Path,
        pattern: &Regex,
        options: &SearchOptions,
    ) -> Result<Vec<SearchMatch>, SearchError> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| SearchError::FileReadError(e.to_string()))?;

        let mut matches = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let line_number = line_idx + 1;
            let search_line = if options.case_sensitive {
                line.to_string()
            } else {
                line.to_lowercase()
            };

            for mat in pattern.find_iter(&search_line) {
                let match_start = mat.start();
                let match_end = mat.end();

                // Adicionar contexto se solicitado
                let context_before = if options.context_lines > 0 && line_idx > 0 {
                    Some(lines[line_idx.saturating_sub(options.context_lines)..line_idx]
                        .join("\n"))
                } else {
                    None
                };

                let context_after = if options.context_lines > 0 && line_idx < lines.len() - 1 {
                    Some(lines[line_idx + 1..=std::cmp::min(
                        line_idx + options.context_lines,
                        lines.len() - 1,
                    )]
                    .join("\n"))
                } else {
                    None
                };

                matches.push(SearchMatch {
                    line_number,
                    line_content: line.to_string(),
                    match_start,
                    match_end,
                    context_before,
                    context_after,
                });
            }
        }

        Ok(matches)
    }

    /// Busca em metadados (frontmatter YAML)
    pub fn search_metadata(
        &self,
        file_path: &Path,
        filters: &HashMap<String, String>,
    ) -> Result<Vec<SearchMatch>, SearchError> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| SearchError::FileReadError(e.to_string()))?;

        let mut matches = Vec::new();

        // Parse YAML frontmatter
        if let Some(frontmatter) = self.extract_frontmatter(&content) {
            for (key, value) in filters {
                if let Some(field_value) = frontmatter.get(key) {
                    if field_value.contains(value) {
                        matches.push(SearchMatch {
                            line_number: 1, // Frontmatter está no início
                            line_content: format!("{}: {}", key, field_value),
                            match_start: 0,
                            match_end: field_value.len(),
                            context_before: None,
                            context_after: None,
                        });
                    }
                }
            }
        }

        Ok(matches)
    }

    /// Compila o padrão de busca em regex
    fn compile_pattern(&self, pattern: &str, options: &SearchOptions) -> Result<Regex, SearchError> {
        let mut regex_pattern = if options.regex {
            pattern.to_string()
        } else {
            // Escapar caracteres especiais do regex
            regex::escape(pattern)
        };

        // Adicionar word boundaries se solicitado
        if options.whole_word && !options.regex {
            regex_pattern = format!(r"\b{}\b", regex_pattern);
        }

        // Adicionar flags de case sensitivity
        let mut flags = String::new();
        if !options.case_sensitive {
            flags.push('i');
        }

        let final_pattern = if flags.is_empty() {
            regex_pattern
        } else {
            format!("(?{}){}", flags, regex_pattern)
        };

        Regex::new(&final_pattern)
            .map_err(|e| SearchError::RegexError(e.to_string()))
    }

    /// Detecta o tipo de arquivo baseado no caminho
    fn detect_file_type(&self, file_path: &Path) -> Result<FileType, SearchError> {
        let path_str = file_path.to_string_lossy().to_lowercase();

        if path_str.contains("projects/") && file_path.extension().map_or(false, |ext| ext == "yaml") {
            Ok(FileType::Project)
        } else if path_str.contains("tasks/") && file_path.extension().map_or(false, |ext| ext == "yaml") {
            Ok(FileType::Task)
        } else if path_str.contains("resources/") && file_path.extension().map_or(false, |ext| ext == "yaml") {
            Ok(FileType::Resource)
        } else if path_str.contains("companies/") && file_path.extension().map_or(false, |ext| ext == "yaml") {
            Ok(FileType::Company)
        } else if file_path.file_name().map_or(false, |name| name == "config.yaml") {
            Ok(FileType::Config)
        } else {
            Ok(FileType::Other)
        }
    }

    /// Extrai frontmatter YAML de um arquivo
    fn extract_frontmatter(&self, content: &str) -> Option<HashMap<String, String>> {
        if content.starts_with("---\n") {
            if let Some(end_pos) = content.find("\n---\n") {
                let frontmatter = &content[4..end_pos];
                // Parse simples do YAML (implementação básica)
                let mut result = HashMap::new();
                for line in frontmatter.lines() {
                    if let Some(colon_pos) = line.find(':') {
                        let key = line[..colon_pos].trim().to_string();
                        let value = line[colon_pos + 1..].trim().to_string();
                        result.insert(key, value);
                    }
                }
                Some(result)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Calcula score de relevância para um resultado
    fn calculate_score(&self, matches: &[SearchMatch], pattern: &str) -> f32 {
        let mut score = 0.0;

        for mat in matches {
            // Score baseado no número de matches
            score += 1.0;

            // Score baseado na posição do match (início da linha = maior score)
            let position_score = 1.0 - (mat.match_start as f32 / mat.line_content.len() as f32);
            score += position_score * 0.5;

            // Score baseado no tamanho do match (matches mais longos = maior score)
            let length_score = (mat.match_end - mat.match_start) as f32 / pattern.len() as f32;
            score += length_score * 0.3;
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_search_engine_creation() {
        let temp_dir = tempdir().unwrap();
        let engine = SearchEngine::new(temp_dir.path().to_path_buf());
        assert_eq!(engine.root_path, temp_dir.path().to_path_buf());
    }

    #[test]
    fn test_detect_file_type() {
        let temp_dir = tempdir().unwrap();
        let engine = SearchEngine::new(temp_dir.path().to_path_buf());

        // Test project file
        let project_path = temp_dir.path().join("projects").join("test.yaml");
        fs::create_dir_all(project_path.parent().unwrap()).unwrap();
        fs::write(&project_path, "test content").unwrap();
        assert_eq!(engine.detect_file_type(&project_path).unwrap(), FileType::Project);

        // Test config file
        let config_path = temp_dir.path().join("config.yaml");
        fs::write(&config_path, "test content").unwrap();
        assert_eq!(engine.detect_file_type(&config_path).unwrap(), FileType::Config);
    }

    #[test]
    fn test_compile_pattern() {
        let temp_dir = tempdir().unwrap();
        let engine = SearchEngine::new(temp_dir.path().to_path_buf());

        let options = SearchOptions::default();
        let pattern = engine.compile_pattern("test", &options).unwrap();
        assert!(pattern.is_match("This is a test"));
        assert!(!pattern.is_match("This is not"));

        // Test case insensitive
        let case_insensitive_options = SearchOptions {
            case_sensitive: false,
            ..Default::default()
        };
        let pattern = engine.compile_pattern("TEST", &case_insensitive_options).unwrap();
        assert!(pattern.is_match("this is a test"));
    }

    #[test]
    fn test_search_in_file() {
        let temp_dir = tempdir().unwrap();
        let engine = SearchEngine::new(temp_dir.path().to_path_buf());

        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Line 1: test content\nLine 2: another test\nLine 3: no match").unwrap();

        let options = SearchOptions::default();
        let pattern = engine.compile_pattern("test", &options).unwrap();
        let matches = engine.search_in_file(&test_file, &pattern, &options).unwrap();

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line_number, 1);
        assert_eq!(matches[1].line_number, 2);
    }

    #[test]
    fn test_extract_frontmatter() {
        let temp_dir = tempdir().unwrap();
        let engine = SearchEngine::new(temp_dir.path().to_path_buf());

        let content = "---\nname: Test Project\nstatus: active\n---\nContent here";
        let frontmatter = engine.extract_frontmatter(content).unwrap();
        
        assert_eq!(frontmatter.get("name"), Some(&"Test Project".to_string()));
        assert_eq!(frontmatter.get("status"), Some(&"active".to_string()));
    }
}
