use std::fmt;

#[derive(Debug)]
pub enum DomainError {
    Generic(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::Generic(msg) => write!(f, "Erro genérico: {}", msg),
        }
    }
}

impl std::error::Error for DomainError {}
