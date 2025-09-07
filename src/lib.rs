//! TaskTaskRevolution Library
//!
//! Esta biblioteca contém a lógica principal do TTR CLI,
//! separada da interface de linha de comando para facilitar
//! testes unitários e reutilização.

use clap::Parser;

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interface;

/// Função principal da biblioteca que pode ser chamada
/// tanto pela CLI quanto por testes
pub fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // Inicializa o sistema de Dependency Injection
    interface::cli::handlers::init_di_handler()
        .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error + Send + Sync + 'static>)?;

    let cli = interface::cli::Cli::parse();
    cli.execute().map_err(|e| {
        Box::new(std::io::Error::other(format!("{}", e))) as Box<dyn std::error::Error + Send + Sync + 'static>
    })
}

/// Configuração da aplicação
pub struct AppConfig {
    pub name: String,
    pub email: String,
    pub company_name: String,
    pub timezone: Option<String>,
}

impl AppConfig {
    pub fn new(name: String, email: String, company_name: String) -> Self {
        Self {
            name,
            email,
            company_name,
            timezone: None,
        }
    }

    pub fn with_timezone(mut self, timezone: String) -> Self {
        self.timezone = Some(timezone);
        self
    }
}

/// Resultado de operações da CLI
pub type CliResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Trait para operações que podem ser testadas
pub trait TestableOperation {
    fn execute(&self) -> CliResult<String>;
    fn validate(&self) -> CliResult<()>;
}

/// Operação de inicialização testável
pub struct InitOperation {
    pub config: AppConfig,
}

impl InitOperation {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }
}

impl TestableOperation for InitOperation {
    fn execute(&self) -> CliResult<String> {
        // Aqui você chamaria a lógica real de inicialização
        // Por enquanto, retornamos uma string simulada
        Ok(format!(
            "Manager: {} ({})\nCompany: {}\nTimezone: {}",
            self.config.name,
            self.config.email,
            self.config.company_name,
            self.config.timezone.as_deref().unwrap_or("UTC")
        ))
    }

    fn validate(&self) -> CliResult<()> {
        if self.config.name.is_empty() {
            return Err("Name cannot be empty".into());
        }
        if self.config.email.is_empty() {
            return Err("Email cannot be empty".into());
        }
        if self.config.company_name.is_empty() {
            return Err("Company name cannot be empty".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_operation_validation() {
        let config = AppConfig::new(
            "Test Manager".to_string(),
            "test@example.com".to_string(),
            "Test Company".to_string(),
        );
        let operation = InitOperation::new(config);

        assert!(operation.validate().is_ok());
    }

    #[test]
    fn test_init_operation_validation_empty_name() {
        let config = AppConfig::new(
            "".to_string(),
            "test@example.com".to_string(),
            "Test Company".to_string(),
        );
        let operation = InitOperation::new(config);

        assert!(operation.validate().is_err());
    }

    #[test]
    fn test_init_operation_execute() {
        let config = AppConfig::new(
            "Test Manager".to_string(),
            "test@example.com".to_string(),
            "Test Company".to_string(),
        )
        .with_timezone("America/Sao_Paulo".to_string());

        let operation = InitOperation::new(config);
        let result = operation.execute().unwrap();

        assert!(result.contains("Test Manager"));
        assert!(result.contains("test@example.com"));
        assert!(result.contains("Test Company"));
        assert!(result.contains("America/Sao_Paulo"));
    }
}
