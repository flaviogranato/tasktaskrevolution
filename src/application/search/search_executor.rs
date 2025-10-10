use crate::application::errors::AppError;
use crate::domain::shared::search_engine::{FileType, SearchEngine, SearchOptions, SearchQuery, SearchResult};
use std::path::PathBuf;

/// Executor de busca integrado com o SearchEngine
pub struct SearchExecutor {
    search_engine: SearchEngine,
    workspace_path: PathBuf,
}

impl SearchExecutor {
    /// Cria um novo SearchExecutor
    pub fn new(workspace_path: PathBuf) -> Self {
        let search_engine = SearchEngine::new(workspace_path.clone());
        Self {
            search_engine,
            workspace_path,
        }
    }

    /// Executa uma busca geral
    pub fn search(&self, query: &str, options: SearchOptions) -> Result<Vec<SearchResult>, AppError> {
        let search_query = SearchQuery {
            pattern: query.to_string(),
            file_types: vec![], // Buscar em todos os tipos
            field_filters: std::collections::HashMap::new(),
            options,
        };

        self.search_engine
            .search(search_query)
            .map_err(|e| AppError::search_error(e.to_string()))
    }

    /// Executa busca por tipo de entidade
    pub fn search_by_entity_type(
        &self,
        entity_type: EntityType,
        query: &str,
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>, AppError> {
        let file_types = match entity_type {
            EntityType::Project => vec![FileType::Project],
            EntityType::Task => vec![FileType::Task],
            EntityType::Resource => vec![FileType::Resource],
            EntityType::Company => vec![FileType::Company],
        };

        let search_query = SearchQuery {
            pattern: query.to_string(),
            file_types,
            field_filters: std::collections::HashMap::new(),
            options,
        };

        self.search_engine
            .search(search_query)
            .map_err(|e| AppError::search_error(e.to_string()))
    }

    /// Executa busca por campo específico
    pub fn search_by_field(
        &self,
        field: &str,
        value: &str,
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>, AppError> {
        let mut field_filters = std::collections::HashMap::new();
        field_filters.insert(field.to_string(), value.to_string());

        let search_query = SearchQuery {
            pattern: value.to_string(),
            file_types: vec![],
            field_filters,
            options,
        };

        self.search_engine
            .search(search_query)
            .map_err(|e| AppError::search_error(e.to_string()))
    }

    /// Executa busca com filtros de campo
    pub fn search_with_filters(
        &self,
        query: &str,
        field_filters: std::collections::HashMap<String, String>,
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>, AppError> {
        let search_query = SearchQuery {
            pattern: query.to_string(),
            file_types: vec![],
            field_filters,
            options,
        };

        self.search_engine
            .search(search_query)
            .map_err(|e| AppError::search_error(e.to_string()))
    }

    /// Busca por padrão regex
    pub fn search_regex(&self, pattern: &str, options: SearchOptions) -> Result<Vec<SearchResult>, AppError> {
        let mut regex_options = options;
        regex_options.regex = true;

        self.search(pattern, regex_options)
    }

    /// Busca case-sensitive
    pub fn search_case_sensitive(&self, query: &str, options: SearchOptions) -> Result<Vec<SearchResult>, AppError> {
        let mut case_sensitive_options = options;
        case_sensitive_options.case_sensitive = true;

        self.search(query, case_sensitive_options)
    }

    /// Busca por palavra completa
    pub fn search_whole_word(&self, query: &str, options: SearchOptions) -> Result<Vec<SearchResult>, AppError> {
        let mut whole_word_options = options;
        whole_word_options.whole_word = true;

        self.search(query, whole_word_options)
    }

    /// Busca apenas em metadados
    pub fn search_metadata_only(&self, query: &str, options: SearchOptions) -> Result<Vec<SearchResult>, AppError> {
        let mut metadata_options = options;
        metadata_options.include_content = false;
        metadata_options.include_metadata = true;

        self.search(query, metadata_options)
    }

    /// Busca apenas em conteúdo
    pub fn search_content_only(&self, query: &str, options: SearchOptions) -> Result<Vec<SearchResult>, AppError> {
        let mut content_options = options;
        content_options.include_content = true;
        content_options.include_metadata = false;

        self.search(query, content_options)
    }

    /// Retorna estatísticas de busca
    pub fn get_search_stats(&self, results: &[SearchResult]) -> SearchStats {
        let total_files = results.len();
        let total_matches: usize = results.iter().map(|r| r.matches.len()).sum();
        let file_types: std::collections::HashMap<FileType, usize> =
            results
                .iter()
                .fold(std::collections::HashMap::new(), |mut acc, result| {
                    *acc.entry(result.file_type).or_insert(0) += 1;
                    acc
                });

        SearchStats {
            total_files,
            total_matches,
            file_types,
        }
    }

    /// Retorna o caminho do workspace
    pub fn workspace_path(&self) -> &PathBuf {
        &self.workspace_path
    }
}

/// Tipos de entidades para busca
#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    Project,
    Task,
    Resource,
    Company,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityType::Project => write!(f, "project"),
            EntityType::Task => write!(f, "task"),
            EntityType::Resource => write!(f, "resource"),
            EntityType::Company => write!(f, "company"),
        }
    }
}

impl std::str::FromStr for EntityType {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "project" => Ok(EntityType::Project),
            "task" => Ok(EntityType::Task),
            "resource" => Ok(EntityType::Resource),
            "company" => Ok(EntityType::Company),
            _ => Err(AppError::validation_error(
                "entity_type",
                format!("Invalid entity type: {}", s),
            )),
        }
    }
}

/// Estatísticas de busca
#[derive(Debug, Clone, PartialEq)]
pub struct SearchStats {
    pub total_files: usize,
    pub total_matches: usize,
    pub file_types: std::collections::HashMap<FileType, usize>,
}

impl std::fmt::Display for SearchStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Search Statistics:")?;
        writeln!(f, "  Total files: {}", self.total_files)?;
        writeln!(f, "  Total matches: {}", self.total_matches)?;
        writeln!(f, "  File types:")?;
        for (file_type, count) in &self.file_types {
            writeln!(f, "    {}: {}", file_type, count)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Project => write!(f, "project"),
            FileType::Task => write!(f, "task"),
            FileType::Resource => write!(f, "resource"),
            FileType::Company => write!(f, "company"),
            FileType::Config => write!(f, "config"),
            FileType::Other => write!(f, "other"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_search_executor_creation() {
        let temp_dir = tempdir().unwrap();
        let executor = SearchExecutor::new(temp_dir.path().to_path_buf());
        assert_eq!(executor.workspace_path(), temp_dir.path());
    }

    #[test]
    fn test_entity_type_parsing() {
        assert_eq!("project".parse::<EntityType>().unwrap(), EntityType::Project);
        assert_eq!("task".parse::<EntityType>().unwrap(), EntityType::Task);
        assert_eq!("resource".parse::<EntityType>().unwrap(), EntityType::Resource);
        assert_eq!("company".parse::<EntityType>().unwrap(), EntityType::Company);
        assert!("invalid".parse::<EntityType>().is_err());
    }

    #[test]
    fn test_search_stats() {
        let temp_dir = tempdir().unwrap();
        let executor = SearchExecutor::new(temp_dir.path().to_path_buf());

        // Create test files
        let project_file = temp_dir.path().join("projects").join("test.yaml");
        fs::create_dir_all(project_file.parent().unwrap()).unwrap();
        fs::write(&project_file, "name: Test Project\nstatus: active").unwrap();

        let task_file = temp_dir.path().join("tasks").join("test.yaml");
        fs::create_dir_all(task_file.parent().unwrap()).unwrap();
        fs::write(&task_file, "name: Test Task\nstatus: planned").unwrap();

        // Mock results
        let results = vec![
            SearchResult {
                file_path: project_file,
                matches: vec![],
                score: 1.0,
                file_type: FileType::Project,
            },
            SearchResult {
                file_path: task_file,
                matches: vec![],
                score: 1.0,
                file_type: FileType::Task,
            },
        ];

        let stats = executor.get_search_stats(&results);
        assert_eq!(stats.total_files, 2);
        assert_eq!(stats.file_types.get(&FileType::Project), Some(&1));
        assert_eq!(stats.file_types.get(&FileType::Task), Some(&1));
    }
}
