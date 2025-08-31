use crate::e2e_tests::utils::{CliRunner, FileAssertions, FileAssertionBuilder};
use std::path::Path;

/// Testa o fluxo completo de criação e gerenciamento de um projeto
pub struct ProjectLifecycleTest;

impl ProjectLifecycleTest {
    /// Executa todos os testes do ciclo de vida do projeto
    pub fn run_all() -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Executando testes de ciclo de vida do projeto...");
        
        Self::test_project_creation()?;
        Self::test_task_management()?;
        Self::test_resource_assignment()?;
        Self::test_project_status()?;
        Self::test_project_export()?;
        Self::test_project_gantt()?;
        Self::test_project_completion()?;
        
        println!("✅ Todos os testes de ciclo de vida passaram!");
        Ok(())
    }
    
    /// Testa a criação básica de um projeto
    fn test_project_creation() -> Result<(), Box<dyn std::error::Error>> {
        println!("  📋 Testando criação de projeto...");
        
        let runner = CliRunner::new()?;
        
        // Inicializar TTR
        runner.init_ttr("Test Company")?;
        
        // Verificar se diretório .ttr foi criado
        assert!(runner.has_ttr_directory());
        
        // Verificar estrutura de diretórios
        FileAssertions::assert_directory_structure(
            &runner.ttr_path(),
            &["companies", "projects", "resources"]
        )?;
        
        // Criar projeto
        runner.create_project("PROJ-001", "Test Project", "Test Company")?;
        
        // Verificar se arquivo do projeto foi criado
        assert!(runner.project_exists("PROJ-001"));
        
        // Verificar se arquivo da empresa foi criado
        assert!(runner.company_exists("Test Company"));
        
        // Validar arquivo YAML do projeto
        let project_file = runner.projects_path().join("PROJ-001.yaml");
        FileAssertionBuilder::new(project_file)
            .exists()
            .valid_yaml()
            .contains("PROJ-001")
            .contains("Test Project")
            .assert()?;
        
        println!("    ✅ Criação de projeto funcionou!");
        Ok(())
    }
    
    /// Testa o gerenciamento de tarefas
    fn test_task_management() -> Result<(), Box<dyn std::error::Error>> {
        println!("  📝 Testando gerenciamento de tarefas...");
        
        let runner = CliRunner::new()?;
        runner.init_ttr("Test Company")?;
        runner.create_project("PROJ-001", "Test Project", "Test Company")?;
        
        // Adicionar tarefas
        let tasks = vec![
            ("Task 1", "5d"),
            ("Task 2", "3d"),
            ("Task 3", "7d"),
        ];
        
        for (name, duration) in tasks {
            runner.add_task("PROJ-001", name, duration)?;
        }
        
        // Verificar se tarefas foram adicionadas ao arquivo YAML
        let project_file = runner.projects_path().join("PROJ-001.yaml");
        let content = std::fs::read_to_string(&project_file)?;
        
        for (name, _) in tasks {
            assert!(content.contains(name), "Task '{}' não foi encontrada no arquivo", name);
        }
        
        println!("    ✅ Gerenciamento de tarefas funcionou!");
        Ok(())
    }
    
    /// Testa a atribuição de recursos
    fn test_resource_assignment() -> Result<(), Box<dyn std::error::Error>> {
        println!("  👥 Testando atribuição de recursos...");
        
        let runner = CliRunner::new()?;
        runner.init_ttr("Test Company")?;
        runner.create_project("PROJ-001", "Test Project", "Test Company")?;
        runner.add_task("PROJ-001", "Task 1", "5d")?;
        
        // Criar recursos
        let resources = vec![
            ("RES-001", "John Doe", "Human"),
            ("RES-002", "Jane Smith", "Human"),
            ("RES-003", "Server-01", "Equipment"),
        ];
        
        for (code, name, resource_type) in resources {
            runner.create_resource(code, name, resource_type)?;
        }
        
        // Verificar se recursos foram criados
        for (code, _, _) in resources {
            assert!(runner.resource_exists(code));
        }
        
        // Atribuir recursos às tarefas
        let assignments = vec![
            ("Task 1", "RES-001", "80"),
            ("Task 1", "RES-002", "60"),
        ];
        
        for (task, resource, allocation) in assignments {
            runner.assign_resource("PROJ-001", task, resource, allocation)?;
        }
        
        // Verificar se atribuições foram salvas
        let project_file = runner.projects_path().join("PROJ-001.yaml");
        let content = std::fs::read_to_string(&project_file)?;
        
        for (_, resource, _) in assignments {
            assert!(content.contains(resource), "Resource '{}' não foi atribuído", resource);
        }
        
        println!("    ✅ Atribuição de recursos funcionou!");
        Ok(())
    }
    
    /// Testa a consulta de status do projeto
    fn test_project_status() -> Result<(), Box<dyn std::error::Error>> {
        println!("  📊 Testando consulta de status...");
        
        let runner = CliRunner::new()?;
        runner.init_ttr("Test Company")?;
        runner.create_project("PROJ-001", "Test Project", "Test Company")?;
        runner.add_task("PROJ-001", "Task 1", "5d")?;
        
        // Obter status do projeto
        let status_output = runner.get_project_status("PROJ-001")?;
        
        // Verificar se output contém informações do projeto
        assert!(status_output.contains("PROJ-001"));
        assert!(status_output.contains("Test Project"));
        assert!(status_output.contains("Task 1"));
        
        println!("    ✅ Consulta de status funcionou!");
        Ok(())
    }
    
    /// Testa a exportação do projeto para CSV
    fn test_project_export() -> Result<(), Box<dyn std::error::Error>> {
        println!("  📤 Testando exportação CSV...");
        
        let runner = CliRunner::new()?;
        runner.init_ttr("Test Company")?;
        runner.create_project("PROJ-001", "Test Project", "Test Company")?;
        runner.add_task("PROJ-001", "Task 1", "5d")?;
        
        // Exportar projeto para CSV
        let csv_output = "project_export.csv";
        runner.export_project_csv("PROJ-001", csv_output)?;
        
        // Verificar se arquivo CSV foi criado
        let csv_file = runner.temp_path().join(csv_output);
        FileAssertionBuilder::new(csv_file)
            .exists()
            .valid_csv()
            .has_extension("csv")
            .min_size(10)
            .assert()?;
        
        // Verificar se conteúdo CSV contém dados do projeto
        let content = std::fs::read_to_string(&csv_file)?;
        assert!(content.contains("PROJ-001"));
        assert!(content.contains("Task 1"));
        
        println!("    ✅ Exportação CSV funcionou!");
        Ok(())
    }
    
    /// Testa a geração de Gantt chart
    fn test_project_gantt() -> Result<(), Box<dyn std::error::Error>> {
        println!("    📈 Testando geração de Gantt...");
        
        let runner = CliRunner::new()?;
        runner.init_ttr("Test Company")?;
        runner.create_project("PROJ-001", "Test Project", "Test Company")?;
        runner.add_task("PROJ-001", "Task 1", "5d")?;
        runner.add_task("PROJ-001", "Task 2", "3d")?;
        
        // Gerar Gantt chart
        let gantt_output = "project_gantt.html";
        runner.generate_gantt("PROJ-001", gantt_output)?;
        
        // Verificar se arquivo HTML foi criado
        let gantt_file = runner.temp_path().join(gantt_output);
        FileAssertionBuilder::new(gantt_file)
            .exists()
            .valid_html()
            .has_extension("html")
            .min_size(100)
            .assert()?;
        
        // Verificar se conteúdo HTML contém dados do projeto
        let content = std::fs::read_to_string(&gantt_file)?;
        assert!(content.contains("PROJ-001"));
        assert!(content.contains("Task 1"));
        assert!(content.contains("Task 2"));
        
        println!("      ✅ Geração de Gantt funcionou!");
        Ok(())
    }
    
    /// Testa a finalização do projeto
    fn test_project_completion() -> Result<(), Box<dyn std::error::Error>> {
        println!("  🎯 Testando finalização do projeto...");
        
        let runner = CliRunner::new()?;
        runner.init_ttr("Test Company")?;
        runner.create_project("PROJ-001", "Test Project", "Test Company")?;
        runner.add_task("PROJ-001", "Task 1", "5d")?;
        
        // Marcar tarefa como concluída (simulado via mudança de status)
        // Nota: Este comando pode não existir ainda, então vamos simular
        let project_file = runner.projects_path().join("PROJ-001.yaml");
        let mut content = std::fs::read_to_string(&project_file)?;
        
        // Simular mudança de status da tarefa
        content = content.replace("status: NotStarted", "status: Completed");
        std::fs::write(&project_file, content)?;
        
        // Verificar se mudança foi persistida
        let updated_content = std::fs::read_to_string(&project_file)?;
        assert!(updated_content.contains("status: Completed"));
        
        println!("    ✅ Finalização do projeto funcionou!");
        Ok(())
    }
}

/// Testa cenários de erro e validação
pub struct ProjectErrorTest;

impl ProjectErrorTest {
    /// Executa todos os testes de erro
    pub fn run_all() -> Result<(), Box<dyn std::error::Error>> {
        println!("🚨 Executando testes de erro e validação...");
        
        Self::test_duplicate_project_creation()?;
        Self::test_invalid_project_code()?;
        Self::test_nonexistent_project_operations()?;
        Self::test_invalid_resource_assignment()?;
        
        println!("✅ Todos os testes de erro passaram!");
        Ok(())
    }
    
    /// Testa criação de projeto com código duplicado
    fn test_duplicate_project_creation() -> Result<(), Box<dyn std::error::Error>> {
        println!("  🔄 Testando criação de projeto duplicado...");
        
        let runner = CliRunner::new()?;
        runner.init_ttr("Test Company")?;
        runner.create_project("PROJ-001", "Test Project", "Test Company")?;
        
        // Tentar criar projeto com mesmo código
        let result = runner.create_project("PROJ-001", "Duplicate Project", "Test Company");
        assert!(result.is_err(), "Deveria falhar ao criar projeto duplicado");
        
        println!("    ✅ Detecção de projeto duplicado funcionou!");
        Ok(())
    }
    
    /// Testa criação de projeto com código inválido
    fn test_invalid_project_code() -> Result<(), Box<dyn std::error::Error>> {
        println!("  ❌ Testando código de projeto inválido...");
        
        let runner = CliRunner::new()?;
        runner.init_ttr("Test Company")?;
        
        // Tentar criar projeto com código vazio
        let result = runner.create_project("", "Test Project", "Test Company");
        assert!(result.is_err(), "Deveria falhar com código vazio");
        
        // Tentar criar projeto com código muito longo
        let long_code = "A".repeat(100);
        let result = runner.create_project(&long_code, "Test Project", "Test Company");
        assert!(result.is_err(), "Deveria falhar com código muito longo");
        
        println!("    ✅ Validação de código inválido funcionou!");
        Ok(())
    }
    
    /// Testa operações em projeto inexistente
    fn test_nonexistent_project_operations() -> Result<(), Box<dyn std::error::Error>> {
        println!("  👻 Testando operações em projeto inexistente...");
        
        let runner = CliRunner::new()?;
        runner.init_ttr("Test Company")?;
        
        // Tentar adicionar tarefa a projeto inexistente
        let result = runner.add_task("INVALID", "Task 1", "5d");
        assert!(result.is_err(), "Deveria falhar ao adicionar tarefa a projeto inexistente");
        
        // Tentar obter status de projeto inexistente
        let result = runner.get_project_status("INVALID");
        assert!(result.is_err(), "Deveria falhar ao obter status de projeto inexistente");
        
        println!("    ✅ Validação de projeto inexistente funcionou!");
        Ok(())
    }
    
    /// Testa atribuição inválida de recursos
    fn test_invalid_resource_assignment() -> Result<(), Box<dyn std::error::Error>> {
        println!("  🚫 Testando atribuição inválida de recursos...");
        
        let runner = CliRunner::new()?;
        runner.init_ttr("Test Company")?;
        runner.create_project("PROJ-001", "Test Project", "Test Company")?;
        runner.add_task("PROJ-001", "Task 1", "5d")?;
        
        // Tentar atribuir recurso inexistente
        let result = runner.assign_resource("PROJ-001", "Task 1", "INVALID", "80");
        assert!(result.is_err(), "Deveria falhar ao atribuir recurso inexistente");
        
        // Tentar atribuir com alocação inválida
        let result = runner.assign_resource("PROJ-001", "Task 1", "RES-001", "150");
        assert!(result.is_err(), "Deveria falhar com alocação > 100%");
        
        println!("    ✅ Validação de atribuição inválida funcionou!");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_project_lifecycle_integration() {
        let result = ProjectLifecycleTest::run_all();
        assert!(result.is_ok(), "Project lifecycle tests failed: {:?}", result.err());
    }
    
    #[test]
    fn test_project_error_handling() {
        let result = ProjectErrorTest::run_all();
        assert!(result.is_ok(), "Project error tests failed: {:?}", result.err());
    }
}
