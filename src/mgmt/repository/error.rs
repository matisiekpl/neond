use crate::mgmt::dto::error::AppError;

impl From<diesel::result::Error> for AppError {
    fn from(e: diesel::result::Error) -> Self {
        match e {
            diesel::result::Error::NotFound => AppError::NotFound,
            other => AppError::Internal(other.to_string()),
        }
    }
}
