use sea_orm::DbErr;

use crate::web::csrf::CsrfError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] DbErr),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("csrf error: {0:?}")]
    Csrf(#[from] CsrfError),
    #[error("conversion blocked")]
    ConversionBlocked,
}

impl AppError {
    pub fn status_code(&self) -> u16 {
        match self {
            AppError::Validation(_) => 422,
            AppError::Unauthorized => 401,
            AppError::Forbidden => 403,
            AppError::NotFound => 404,
            AppError::Csrf(_) => 403,
            AppError::ConversionBlocked => 503,
            AppError::Database(_) => 500,
        }
    }
}
