use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Access denied")]
    Forbidden,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Duplicate entity: {0}")]
    Duplicate(String),
}
