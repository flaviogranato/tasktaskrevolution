use std::fmt;

#[derive(Debug)]
pub enum DomainError {
    Generic(String),
    Io(String),
    Serialization(String),
    NotFound(String),
    InvalidState(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::Generic(msg) => write!(f, "Erro genérico: {msg}"),
            DomainError::Io(msg) => write!(f, "Erro de E/S: {msg}"),
            DomainError::Serialization(msg) => write!(f, "Erro de serialização: {msg}"),
            DomainError::NotFound(msg) => write!(f, "Não encontrado: {msg}"),
            DomainError::InvalidState(msg) => write!(f, "Estado inválido: {msg}"),
        }
    }
}

impl std::error::Error for DomainError {}
