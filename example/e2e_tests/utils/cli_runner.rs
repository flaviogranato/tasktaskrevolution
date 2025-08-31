use std::process::{Command, Output, Stdio};
use std::path::Path;
use tempfile::TempDir;
use std::io::Write;

/// Runner para executar comandos CLI TTR nos testes
pub struct CliRunner {
    temp_dir: TempDir,
    ttr_binary: String,
}

impl CliRunner {
    /// Cria um novo runner com diretório temporário
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let ttr_binary = "ttr".to_string();
        
        Ok(Self {
            temp_dir,
            ttr_binary,
        })
    }
    
    /// Executa um comando CLI TTR
    pub fn run(&self, args: &[&str]) -> Result<Output, Box<dyn std::error::Error>> {
        let output = Command::new(&self.ttr_binary)
            .args(args)
            .current_dir(self.temp_dir.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        
        Ok(output)
    }
    
    /// Executa comando e retorna apenas o stdout como string
    pub fn run_capture(&self, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
        let output = self.run(args)?;
        
        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            let stderr = String::from_utf8(output.stderr)?;
            Err(format!("Command failed: {}", stderr).into())
        }
    }
    
    /// Executa comando e verifica se foi bem-sucedido
    pub fn run_success(&self, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        let output = self.run(args)?;
        
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8(output.stderr)?;
            Err(format!("Command failed: {}", stderr).into())
        }
    }
    
    /// Executa comando e espera que falhe
    pub fn run_expect_failure(&self, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
        let output = self.run(args)?;
        
        if !output.status.success() {
            Ok(String::from_utf8(output.stderr)?)
        } else {
            Err("Command succeeded but was expected to fail".into())
        }
    }
    
    /// Inicializa o diretório TTR
    pub fn init_ttr(&self, company_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.run_success(&["init", "--company", company_name])
    }
    
    /// Cria um projeto
    pub fn create_project(&self, code: &str, name: &str, company: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.run_success(&["project", "create", "--code", code, "--name", name, "--company", company])
    }
    
    /// Adiciona uma tarefa
    pub fn add_task(&self, project: &str, name: &str, duration: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.run_success(&["task", "add", "--project", project, "--name", name, "--duration", duration])
    }
    
    /// Cria um recurso
    pub fn create_resource(&self, code: &str, name: &str, resource_type: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.run_success(&["resource", "create", "--code", code, "--name", name, "--type", resource_type])
    }
    
    /// Atribui recurso a tarefa
    pub fn assign_resource(&self, project: &str, task: &str, resource: &str, allocation: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.run_success(&["resource", "assign", "--project", project, "--task", task, "--resource", resource, "--allocation", allocation])
    }
    
    /// Obtém status do projeto
    pub fn get_project_status(&self, project: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.run_capture(&["project", "status", "--project", project])
    }
    
    /// Exporta projeto para CSV
    pub fn export_project_csv(&self, project: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.run_success(&["project", "export", "--project", project, "--format", "csv", "--output", output])
    }
    
    /// Gera Gantt chart
    pub fn generate_gantt(&self, project: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.run_success(&["project", "gantt", "--project", project, "--output", output])
    }
    
    /// Obtém o caminho do diretório temporário
    pub fn temp_path(&self) -> &Path {
        self.temp_dir.path()
    }
    
    /// Obtém o caminho do diretório .ttr
    pub fn ttr_path(&self) -> std::path::PathBuf {
        self.temp_dir.path().join(".ttr")
    }
    
    /// Obtém o caminho dos projetos
    pub fn projects_path(&self) -> std::path::PathBuf {
        self.ttr_path().join("projects")
    }
    
    /// Obtém o caminho dos recursos
    pub fn resources_path(&self) -> std::path::PathBuf {
        self.ttr_path().join("resources")
    }
    
    /// Obtém o caminho das empresas
    pub fn companies_path(&self) -> std::path::PathBuf {
        self.ttr_path().join("companies")
    }
    
    /// Verifica se o diretório .ttr foi criado
    pub fn has_ttr_directory(&self) -> bool {
        self.ttr_path().exists()
    }
    
    /// Verifica se um projeto existe
    pub fn project_exists(&self, project_code: &str) -> bool {
        let project_file = self.projects_path().join(format!("{}.yaml", project_code));
        project_file.exists()
    }
    
    /// Verifica se um recurso existe
    pub fn resource_exists(&self, resource_code: &str) -> bool {
        let resource_file = self.resources_path().join(format!("{}.yaml", resource_code));
        resource_file.exists()
    }
    
    /// Verifica se uma empresa existe
    pub fn company_exists(&self, company_code: &str) -> bool {
        let company_file = self.companies_path().join(format!("{}.yaml", company_code));
        company_file.exists()
    }
}

impl Drop for CliRunner {
    fn drop(&mut self) {
        // O diretório temporário será limpo automaticamente
    }
}

/// Builder para criar cenários de teste complexos
pub struct TestScenarioBuilder {
    runner: CliRunner,
}

impl TestScenarioBuilder {
    /// Cria um novo builder
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let runner = CliRunner::new()?;
        Ok(Self { runner })
    }
    
    /// Configura um cenário básico com empresa e projeto
    pub fn setup_basic_scenario(mut self, company: &str, project_code: &str, project_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        self.runner.init_ttr(company)?;
        self.runner.create_project(project_code, project_name, company)?;
        Ok(self)
    }
    
    /// Adiciona tarefas ao projeto
    pub fn with_tasks(mut self, tasks: &[(&str, &str)]) -> Result<Self, Box<dyn std::error::Error>> {
        for (name, duration) in tasks {
            self.runner.add_task("PROJ-001", name, duration)?;
        }
        Ok(self)
    }
    
    /// Adiciona recursos
    pub fn with_resources(mut self, resources: &[(&str, &str, &str)]) -> Result<Self, Box<dyn std::error::Error>> {
        for (code, name, resource_type) in resources {
            self.runner.create_resource(code, name, resource_type)?;
        }
        Ok(self)
    }
    
    /// Atribui recursos às tarefas
    pub fn with_assignments(mut self, assignments: &[(&str, &str, &str, &str)]) -> Result<Self, Box<dyn std::error::Error>> {
        for (task, resource, allocation) in assignments {
            self.runner.assign_resource("PROJ-001", task, resource, allocation)?;
        }
        Ok(self)
    }
    
    /// Finaliza o cenário e retorna o runner
    pub fn build(self) -> CliRunner {
        self.runner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cli_runner_creation() {
        let runner = CliRunner::new();
        assert!(runner.is_ok());
    }
    
    #[test]
    fn test_temp_directory_creation() {
        let runner = CliRunner::new().unwrap();
        assert!(runner.temp_path().exists());
    }
    
    #[test]
    fn test_ttr_directory_path() {
        let runner = CliRunner::new().unwrap();
        let ttr_path = runner.ttr_path();
        assert_eq!(ttr_path.file_name().unwrap(), ".ttr");
    }
}
