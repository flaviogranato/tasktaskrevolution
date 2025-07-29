use crate::domain::{
    project_management::{project::Project, repository::ProjectRepository},
    shared::{convertable::Convertable, errors::DomainError},
};
use crate::infrastructure::persistence::manifests::project_manifest::ProjectManifest;
use globwalk::glob;
use serde_yaml;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

/// `FileProjectRepository` é uma implementação da trait `ProjectRepository`
/// que persiste os dados do projeto no sistema de arquivos.
///
/// A estrutura de diretórios esperada é:
/// /<base_path>/<project_name>/project.yaml
pub struct FileProjectRepository {
    base_path: PathBuf,
}

impl FileProjectRepository {
    /// Cria uma nova instância do repositório que opera a partir do diretório de trabalho atual.
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::from("."),
        }
    }

    /// Cria uma nova instância do repositório que opera a partir de um diretório base específico.
    /// Esta função é primariamente para uso em testes.
    pub fn with_base_path(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Carrega e deserializa o manifesto de um projeto de um arquivo YAML.
    fn load_manifest(&self, path: &Path) -> Result<ProjectManifest, Box<dyn Error>> {
        let yaml = fs::read_to_string(path)?;
        serde_yaml::from_str(&yaml).map_err(|e| e.into())
    }
}

impl ProjectRepository for FileProjectRepository {
    /// Salva um projeto.
    /// Cria um diretório com o nome do projeto e salva um arquivo `project.yaml` dentro dele.
    fn save(&self, project: Project) -> Result<(), DomainError> {
        let project_dir = self.base_path.join(&project.name);

        fs::create_dir_all(&project_dir)
            .map_err(|e| DomainError::Generic(format!("Erro ao criar diretório do projeto: {e}")))?;

        let manifest_path = project_dir.join("project.yaml");
        let project_manifest = <ProjectManifest as Convertable<Project>>::from(project);
        let yaml = serde_yaml::to_string(&project_manifest)
            .map_err(|e| DomainError::Generic(format!("Erro ao serializar projeto: {e}")))?;

        fs::write(&manifest_path, yaml)
            .map_err(|e| DomainError::Generic(format!("Erro ao salvar arquivo do projeto: {e}")))?;

        Ok(())
    }

    /// Carrega um projeto.
    /// `path` deve ser o caminho para o diretório do projeto.
    fn load(&self) -> Result<Project, DomainError> {
        let pattern = self.base_path.join("**/project.yaml");
        let walker = glob(pattern.to_str().unwrap()).map_err(|e| DomainError::Generic(e.to_string()))?;

        if let Some(Ok(entry)) = walker.into_iter().next() {
            let manifest_path = entry.path();
            match self.load_manifest(&manifest_path) {
                Ok(manifest) => Ok(manifest.to()),
                Err(e) => Err(DomainError::Generic(format!(
                    "Falha ao carregar ou deserializar o arquivo do projeto: {e}"
                ))),
            }
        } else {
            Err(DomainError::Generic(
                "Nenhum arquivo 'project.yaml' encontrado nos subdiretórios.".to_string(),
            ))
        }
    }
}

// ===================================
// TESTES
// ===================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::project_management::project::ProjectStatus;
    use tempfile::tempdir;

    /// Cria um projeto de teste simples.
    fn create_test_project(name: &str) -> Project {
        Project::new(
            Some(format!("id-{name}")),
            name.to_string(),
            Some(format!("Descrição para {name}")),
            Some("2025-01-01".to_string()),
            Some("2025-12-31".to_string()),
            ProjectStatus::Planned,
            None,
        )
    }

    #[test]
    fn test_save_and_load_project() {
        // 1. Setup
        let temp_dir = tempdir().expect("Não foi possível criar diretório temporário");
        let repo = FileProjectRepository::with_base_path(temp_dir.path().to_path_buf());
        let original_project = create_test_project("MeuProjetoDeTeste");
        let project_name = original_project.name.clone();

        // 2. Salvar o projeto
        let save_result = repo.save(original_project.clone());
        assert!(save_result.is_ok());

        // 3. Verificar se a estrutura de arquivos foi criada corretamente
        let project_dir_path = temp_dir.path().join(&project_name);
        assert!(project_dir_path.exists(), "O diretório do projeto deve existir");
        assert!(project_dir_path.is_dir());

        let manifest_path = project_dir_path.join("project.yaml");
        assert!(manifest_path.exists(), "O arquivo project.yaml deve existir");
        assert!(manifest_path.is_file());

        // 4. Carregar o projeto de volta
        // A função `load` espera o nome do diretório do projeto (relativo ao base_path)
        let loaded_project = repo.load().expect("O carregamento do projeto não deve falhar");

        // 5. Verificar se os dados são consistentes
        // A conversão para/de manifesto pode alterar alguns campos (como o ID, que pode não ser salvo),
        // então comparamos os campos importantes.
        assert_eq!(original_project.name, loaded_project.name);
        assert_eq!(original_project.description, loaded_project.description);
        assert_eq!(original_project.status, loaded_project.status);
        assert_eq!(original_project.start_date, loaded_project.start_date);
        assert_eq!(original_project.end_date, loaded_project.end_date);
    }

    #[test]
    fn test_load_non_existent_project() {
        // 1. Setup
        let temp_dir = tempdir().expect("Não foi possível criar diretório temporário");
        let repo = FileProjectRepository::with_base_path(temp_dir.path().to_path_buf());
        let project_path = PathBuf::from("projeto-que-nao-existe");

        // 2. Tentar carregar
        let result = repo.load();

        // 3. Verificar
        assert!(result.is_err());
        if let Err(DomainError::Generic(msg)) = result {
            assert!(msg.contains("Arquivo de manifesto não encontrado"));
        } else {
            panic!("Esperado um DomainError::Generic");
        }
    }
}
