use crate::domain::services::error::DomainError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UseCaseError {
    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error("Database operation failed")]
    DatabaseError,

    #[error("External service communication failed")]
    ExternalServiceError,

    #[error("Authentication is required and failed or has not yet been provided")]
    Unauthorized,

    #[error("The request was valid, but the server is refusing action")]
    Forbidden,

    #[error("An unexpected internal server error occurred")]
    InternalError,
}
