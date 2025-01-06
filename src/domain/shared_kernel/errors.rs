use std::fmt;

#[derive(Debug, PartialEq)]
pub enum DomainError {
    NotFound(String),
    InvalidInput(String),
    Generic(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::NotFound(msg) => write!(f, "Recurso não encontrado: {}", msg),
            DomainError::InvalidInput(msg) => write!(f, "Entrada inválida: {}", msg),
            DomainError::Generic(msg) => write!(f, "Erro genérico: {}", msg),
        }
    }
}

impl std::error::Error for DomainError {}

pub fn not_found_error(resource: &str, id: &str) -> DomainError {
    DomainError::NotFound(format!("{} com ID {} não encontrado", resource, id))
}
