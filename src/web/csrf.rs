use uuid::Uuid;

use crate::web::router::RouteMethod;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsrfToken {
    value: String,
}

impl CsrfToken {
    pub fn generate() -> Self {
        Self {
            value: Uuid::new_v4().to_string(),
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CsrfError {
    #[error("missing csrf token")]
    Missing,
    #[error("csrf token mismatch")]
    Mismatch,
}

pub fn validate_csrf(
    method: RouteMethod,
    provided: Option<&CsrfToken>,
    expected: Option<&CsrfToken>,
) -> Result<(), CsrfError> {
    if !method.is_state_changing() {
        return Ok(());
    }

    let provided = provided.ok_or(CsrfError::Missing)?;
    let expected = expected.ok_or(CsrfError::Missing)?;

    if provided.value() == expected.value() {
        Ok(())
    } else {
        Err(CsrfError::Mismatch)
    }
}
