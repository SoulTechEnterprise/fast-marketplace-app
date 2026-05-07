use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("The provided input data is invalid")]
    InvalidInput,

    #[error("A required field is missing")]
    MissingField,

    #[error("A business rule was violated")]
    RuleViolation,

    #[error("The requested resource was not found")]
    NotFound,

    #[error("The resource already exists")]
    AlreadyExists,

    #[error("The maximum allowed limit has been reached")]
    LimitReached,

    #[error("Automation failed at step: {0}")]
    AutomationError(String),
}
