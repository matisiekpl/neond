use std::fmt;

#[derive(Debug)]
pub enum AppError {
    NotFound,
    Conflict(String),
    Unauthorized,
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound => write!(f, "Not found"),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::Unauthorized => write!(f, "Unauthorized"),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
