//! Strict ordered form decoding.

use axum::{
    extract::{FromRequest, RawForm, Request},
    response::{IntoResponse, Response},
};

/// Decoded URL-encoded pairs in their original wire order.
pub struct OrderedForm(pub Vec<(String, String)>);

impl OrderedForm {
    /// Returns exactly one required value and rejects unknown or duplicate keys.
    pub fn required_fields<'a>(
        &'a self,
        allowed: &[&str],
    ) -> Result<Vec<(&'a str, &'a str)>, &'static str> {
        let mut values = Vec::new();
        for (key, value) in &self.0 {
            if !allowed.contains(&key.as_str()) || values.iter().any(|(seen, _)| *seen == key) {
                return Err("Malformed form submission.");
            }
            values.push((key, value));
        }
        if values.len() != allowed.len()
            || allowed
                .iter()
                .any(|key| !values.iter().any(|(seen, _)| seen == key))
        {
            return Err("Malformed form submission.");
        }
        Ok(values
            .into_iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect())
    }
}

impl<S> FromRequest<S> for OrderedForm
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let RawForm(bytes) = RawForm::from_request(req, state)
            .await
            .map_err(IntoResponse::into_response)?;
        validate_percent_encoding(&bytes).map_err(|_| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                "Malformed form encoding.",
            )
                .into_response()
        })?;
        Ok(Self(form_urlencoded::parse(&bytes).into_owned().collect()))
    }
}

fn validate_percent_encoding(bytes: &[u8]) -> Result<(), ()> {
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len()
                || !bytes[index + 1].is_ascii_hexdigit()
                || !bytes[index + 2].is_ascii_hexdigit()
            {
                return Err(());
            }
            index += 3;
        } else {
            index += 1;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_percent_encoding;

    #[test]
    fn rejects_incomplete_or_non_hex_percent_encoding() {
        assert!(validate_percent_encoding(b"value=%").is_err());
        assert!(validate_percent_encoding(b"value=%0").is_err());
        assert!(validate_percent_encoding(b"value=%xz").is_err());
        assert!(validate_percent_encoding(b"value=%20").is_ok());
    }
}
