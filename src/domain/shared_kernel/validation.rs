use validator::{Validate, ValidationError};
use crate::domain::shared_kernel::errors::DomainError;

/// Trait para validação de entidades do domínio
pub trait DomainValidation: Validate {
    /// Valida a entidade e retorna um Result
    fn validate_domain(&self) -> Result<(), DomainError> {
        self.validate().map_err(|e| DomainError::ValidationError(e))
    }
}

/// Função auxiliar para criar erros de validação
pub fn create_validation_error(field: &str, message: &str) -> ValidationError {
    let mut error = ValidationError::new("validation_error");
    error.message = Some(message.to_string().into());
    error.params = std::collections::HashMap::new();
    error
}

/// Função para validar datas
pub fn validate_date(date: &str) -> Result<(), ValidationError> {
    if chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_err() {
        return Err(create_validation_error("date", "Data inválida. Use o formato YYYY-MM-DD"));
    }
    Ok(())
}

/// Função para validar UUIDs
pub fn validate_uuid(uuid: &str) -> Result<(), ValidationError> {
    if uuid::Uuid::parse_str(uuid).is_err() {
        return Err(create_validation_error("uuid", "UUID inválido"));
    }
    Ok(())
} 