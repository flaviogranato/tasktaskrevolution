use crate::domain::shared::search_engine::{FileType, SearchResult};

/// Filtro de busca para refinar resultados
pub struct SearchFilter {
    file_types: Vec<FileType>,
    min_score: Option<f32>,
    max_score: Option<f32>,
    min_matches: Option<usize>,
    max_matches: Option<usize>,
    path_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
}

impl SearchFilter {
    /// Cria um novo SearchFilter
    pub fn new() -> Self {
        Self {
            file_types: vec![],
            min_score: None,
            max_score: None,
            min_matches: None,
            max_matches: None,
            path_patterns: vec![],
            exclude_patterns: vec![],
        }
    }

    /// Filtra por tipos de arquivo
    pub fn file_types(mut self, types: Vec<FileType>) -> Self {
        self.file_types = types;
        self
    }

    /// Filtra por score mínimo
    pub fn min_score(mut self, score: f32) -> Self {
        self.min_score = Some(score);
        self
    }

    /// Filtra por score máximo
    pub fn max_score(mut self, score: f32) -> Self {
        self.max_score = Some(score);
        self
    }

    /// Filtra por número mínimo de matches
    pub fn min_matches(mut self, matches: usize) -> Self {
        self.min_matches = Some(matches);
        self
    }

    /// Filtra por número máximo de matches
    pub fn max_matches(mut self, matches: usize) -> Self {
        self.max_matches = Some(matches);
        self
    }

    /// Adiciona padrão de caminho para incluir
    pub fn include_path(mut self, pattern: &str) -> Self {
        self.path_patterns.push(pattern.to_string());
        self
    }

    /// Adiciona padrão de caminho para excluir
    pub fn exclude_path(mut self, pattern: &str) -> Self {
        self.exclude_patterns.push(pattern.to_string());
        self
    }

    /// Aplica o filtro aos resultados
    pub fn apply(&self, results: &[SearchResult]) -> Vec<SearchResult> {
        results
            .iter()
            .filter(|result| self.matches_file_type(result))
            .filter(|result| self.matches_score(result))
            .filter(|result| self.matches_match_count(result))
            .filter(|result| self.matches_path_patterns(result))
            .filter(|result| !self.matches_exclude_patterns(result))
            .cloned()
            .collect()
    }

    /// Verifica se o resultado corresponde aos tipos de arquivo
    fn matches_file_type(&self, result: &SearchResult) -> bool {
        self.file_types.is_empty() || self.file_types.contains(&result.file_type)
    }

    /// Verifica se o resultado corresponde aos critérios de score
    fn matches_score(&self, result: &SearchResult) -> bool {
        if let Some(min_score) = self.min_score
            && result.score < min_score
        {
            return false;
        }

        if let Some(max_score) = self.max_score
            && result.score > max_score
        {
            return false;
        }

        true
    }

    /// Verifica se o resultado corresponde aos critérios de número de matches
    fn matches_match_count(&self, result: &SearchResult) -> bool {
        let match_count = result.matches.len();

        if let Some(min_matches) = self.min_matches
            && match_count < min_matches
        {
            return false;
        }

        if let Some(max_matches) = self.max_matches
            && match_count > max_matches
        {
            return false;
        }

        true
    }

    /// Verifica se o resultado corresponde aos padrões de caminho incluídos
    fn matches_path_patterns(&self, result: &SearchResult) -> bool {
        if self.path_patterns.is_empty() {
            return true;
        }

        let path_str = result.file_path.to_string_lossy().to_lowercase();
        self.path_patterns
            .iter()
            .any(|pattern| path_str.contains(&pattern.to_lowercase()))
    }

    /// Verifica se o resultado corresponde aos padrões de caminho excluídos
    fn matches_exclude_patterns(&self, result: &SearchResult) -> bool {
        if self.exclude_patterns.is_empty() {
            return false;
        }

        let path_str = result.file_path.to_string_lossy().to_lowercase();
        self.exclude_patterns
            .iter()
            .any(|pattern| path_str.contains(&pattern.to_lowercase()))
    }
}

impl Default for SearchFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder para SearchFilter com métodos fluentes
pub struct SearchFilterBuilder {
    filter: SearchFilter,
}

impl SearchFilterBuilder {
    /// Cria um novo SearchFilterBuilder
    pub fn new() -> Self {
        Self {
            filter: SearchFilter::new(),
        }
    }

    /// Filtra apenas projetos
    pub fn projects_only(mut self) -> Self {
        self.filter.file_types = vec![FileType::Project];
        self
    }

    /// Filtra apenas tarefas
    pub fn tasks_only(mut self) -> Self {
        self.filter.file_types = vec![FileType::Task];
        self
    }

    /// Filtra apenas recursos
    pub fn resources_only(mut self) -> Self {
        self.filter.file_types = vec![FileType::Resource];
        self
    }

    /// Filtra apenas empresas
    pub fn companies_only(mut self) -> Self {
        self.filter.file_types = vec![FileType::Company];
        self
    }

    /// Filtra por score alto (>= 2.0)
    pub fn high_score(mut self) -> Self {
        self.filter.min_score = Some(2.0);
        self
    }

    /// Filtra por score médio (1.0 - 2.0)
    pub fn medium_score(mut self) -> Self {
        self.filter.min_score = Some(1.0);
        self.filter.max_score = Some(2.0);
        self
    }

    /// Filtra por score baixo (< 1.0)
    pub fn low_score(mut self) -> Self {
        self.filter.max_score = Some(1.0);
        self
    }

    /// Filtra por múltiplos matches
    pub fn multiple_matches(mut self) -> Self {
        self.filter.min_matches = Some(2);
        self
    }

    /// Filtra por match único
    pub fn single_match(mut self) -> Self {
        self.filter.max_matches = Some(1);
        self
    }

    /// Exclui arquivos de configuração
    pub fn exclude_config(mut self) -> Self {
        self.filter.exclude_patterns.push("config.yaml".to_string());
        self
    }

    /// Exclui arquivos temporários
    pub fn exclude_temp(mut self) -> Self {
        self.filter.exclude_patterns.push(".tmp".to_string());
        self.filter.exclude_patterns.push(".temp".to_string());
        self
    }

    /// Inclui apenas arquivos em diretórios específicos
    pub fn in_directory(mut self, dir: &str) -> Self {
        self.filter.path_patterns.push(dir.to_string());
        self
    }

    /// Constrói o SearchFilter
    pub fn build(self) -> SearchFilter {
        self.filter
    }
}

impl Default for SearchFilterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilitários para filtros comuns
impl SearchFilter {
    /// Cria um filtro para projetos com alta relevância
    pub fn high_relevance_projects() -> Self {
        SearchFilterBuilder::new()
            .projects_only()
            .high_score()
            .multiple_matches()
            .build()
    }

    /// Cria um filtro para tarefas ativas
    pub fn active_tasks() -> Self {
        SearchFilterBuilder::new()
            .tasks_only()
            .medium_score()
            .exclude_config()
            .build()
    }

    /// Cria um filtro para recursos disponíveis
    pub fn available_resources() -> Self {
        SearchFilterBuilder::new()
            .resources_only()
            .medium_score()
            .exclude_temp()
            .build()
    }

    /// Cria um filtro para busca geral
    pub fn general_search() -> Self {
        SearchFilterBuilder::new().exclude_config().exclude_temp().build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::shared::search_engine::SearchMatch;
    use std::path::PathBuf;

    fn create_test_results() -> Vec<SearchResult> {
        vec![
            SearchResult {
                file_path: PathBuf::from("projects/test1.yaml"),
                matches: vec![
                    SearchMatch {
                        line_number: 1,
                        line_content: "name: Test Project".to_string(),
                        match_start: 0,
                        match_end: 4,
                        context_before: None,
                        context_after: None,
                    },
                    SearchMatch {
                        line_number: 2,
                        line_content: "status: active".to_string(),
                        match_start: 0,
                        match_end: 6,
                        context_before: None,
                        context_after: None,
                    },
                ],
                score: 2.5,
                file_type: FileType::Project,
            },
            SearchResult {
                file_path: PathBuf::from("tasks/test2.yaml"),
                matches: vec![],
                score: 1.5,
                file_type: FileType::Task,
            },
            SearchResult {
                file_path: PathBuf::from("config.yaml"),
                matches: vec![],
                score: 0.5,
                file_type: FileType::Config,
            },
        ]
    }

    #[test]
    fn test_filter_by_file_type() {
        let results = create_test_results();
        let filter = SearchFilter::new().file_types(vec![FileType::Project]);
        let filtered = filter.apply(&results);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].file_type, FileType::Project);
    }

    #[test]
    fn test_filter_by_score() {
        let results = create_test_results();
        let filter = SearchFilter::new().min_score(2.0);
        let filtered = filter.apply(&results);

        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].score >= 2.0);
    }

    #[test]
    fn test_filter_by_path_pattern() {
        let results = create_test_results();
        let filter = SearchFilter::new().include_path("projects");
        let filtered = filter.apply(&results);

        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].file_path.to_string_lossy().contains("projects"));
    }

    #[test]
    fn test_filter_exclude_pattern() {
        let results = create_test_results();
        let filter = SearchFilter::new().exclude_path("config");
        let filtered = filter.apply(&results);

        assert_eq!(filtered.len(), 2);
        assert!(
            !filtered
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("config"))
        );
    }

    #[test]
    fn test_filter_builder() {
        let results = create_test_results();
        let filter = SearchFilterBuilder::new()
            .projects_only()
            .high_score()
            .exclude_config()
            .build();

        let filtered = filter.apply(&results);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].file_type, FileType::Project);
    }

    #[test]
    fn test_predefined_filters() {
        let results = create_test_results();

        let high_relevance = SearchFilter::high_relevance_projects();
        let filtered = high_relevance.apply(&results);
        assert_eq!(filtered.len(), 1);

        let general = SearchFilter::general_search();
        let filtered = general.apply(&results);
        assert_eq!(filtered.len(), 2); // Exclui config.yaml
    }
}
