//! Strict ordered form decoding.

use std::collections::HashMap;

use axum::{
    extract::{FromRequest, FromRequestParts, RawForm, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tower_sessions::Session;

use crate::session;

/// Decoded URL-encoded pairs in their original wire order.
pub struct OrderedForm(pub Vec<(String, String)>);

/// An ordered form whose synchronizer token was validated before the handler ran.
pub(crate) struct CsrfValidatedForm(pub(crate) OrderedForm);

impl CsrfValidatedForm {
    /// Returns the validated ordered form for route-specific parsing.
    pub(crate) fn into_inner(self) -> OrderedForm {
        self.0
    }
}

impl<S> FromRequest<S> for CsrfValidatedForm
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let session = Session::from_request_parts(&mut parts, state)
            .await
            .map_err(|_| session_rejection())?;
        let form = OrderedForm::from_request(Request::from_parts(parts, body), state).await?;
        let tokens = form
            .0
            .iter()
            .filter(|(key, _)| key == "csrf")
            .map(|(_, value)| value.as_str())
            .collect::<Vec<_>>();
        if tokens.len() != 1
            || !session::matches_csrf(&session, tokens[0])
                .await
                .map_err(|_| session_rejection())?
        {
            return Err(csrf_rejection());
        }
        Ok(Self(form))
    }
}

fn session_rejection() -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, "Session error.").into_response()
}

fn csrf_rejection() -> Response {
    (StatusCode::FORBIDDEN, "Invalid form token.").into_response()
}

pub(crate) struct GroupForm {
    pub(crate) name: String,
    pub(crate) currency: String,
}

pub(crate) struct ParticipantForm {
    pub(crate) name: String,
    pub(crate) color: String,
}

pub(crate) struct MemberForm {
    pub(crate) participant_id: i64,
}

/// Expense form values after strict field-name and duplicate validation.
pub(crate) struct ExpenseForm {
    pub(crate) description: String,
    pub(crate) total: String,
    pub(crate) currency: String,
    pub(crate) spending_type: String,
    pub(crate) spent_date: String,
    pub(crate) payer_mode: String,
    pub(crate) single_payer_id: Option<i64>,
    pub(crate) split_mode: String,
    pub(crate) extra: HashMap<String, String>,
}

#[derive(Debug)]
pub(crate) struct FormError {
    pub(crate) status: StatusCode,
    pub(crate) message: &'static str,
}

impl OrderedForm {
    /// Returns exactly one required value and rejects unknown or duplicate keys.
    ///
    /// # Errors
    ///
    /// Returns an error for missing, unknown, or duplicated keys.
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

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn parse_group_form(form: OrderedForm) -> Result<GroupForm, FormError> {
    let fields = form
        .required_fields(&["name", "currency", "csrf"])
        .map_err(malformed_form)?;
    Ok(GroupForm {
        name: value(&fields, "name"),
        currency: value(&fields, "currency"),
    })
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn parse_participant_form(form: OrderedForm) -> Result<ParticipantForm, FormError> {
    let fields = form
        .required_fields(&["name", "color", "csrf"])
        .map_err(malformed_form)?;
    Ok(ParticipantForm {
        name: value(&fields, "name"),
        color: value(&fields, "color"),
    })
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn parse_member_form(form: OrderedForm) -> Result<MemberForm, FormError> {
    let fields = form
        .required_fields(&["participant_id", "csrf"])
        .map_err(malformed_form)?;
    let participant_id = value(&fields, "participant_id")
        .parse()
        .map_err(|_| FormError {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: "Invalid participant.",
        })?;
    Ok(MemberForm { participant_id })
}

/// Parses expense field structure without applying financial eligibility policy.
pub(crate) fn parse_expense_form(form: OrderedForm) -> Result<ExpenseForm, FormError> {
    const SCALARS: [&str; 9] = [
        "description",
        "total",
        "currency",
        "spending_type",
        "spent_date",
        "payer_mode",
        "single_payer_id",
        "split_mode",
        "csrf",
    ];

    let mut scalars = HashMap::new();
    let mut extra = HashMap::new();
    for (key, value) in form.0 {
        if SCALARS.contains(&key.as_str()) {
            if scalars.insert(key, value).is_some() {
                return Err(malformed_form("Malformed form submission."));
            }
            continue;
        }
        let Some((prefix, id)) = dynamic_expense_field(&key) else {
            return Err(malformed_form("Malformed form submission."));
        };
        if extra.insert(format!("{prefix}{id}"), value).is_some() {
            return Err(malformed_form("Malformed form submission."));
        }
    }
    if SCALARS.iter().any(|key| !scalars.contains_key(*key)) {
        return Err(malformed_form("Malformed form submission."));
    }

    let single_payer_id = scalar(&scalars, "single_payer_id")
        .parse()
        .map_err(|_| malformed_form("Malformed form submission."))?;
    Ok(ExpenseForm {
        description: scalar(&scalars, "description"),
        total: scalar(&scalars, "total"),
        currency: scalar(&scalars, "currency"),
        spending_type: scalar(&scalars, "spending_type"),
        spent_date: scalar(&scalars, "spent_date"),
        payer_mode: scalar(&scalars, "payer_mode"),
        single_payer_id: (single_payer_id != 0).then_some(single_payer_id),
        split_mode: scalar(&scalars, "split_mode"),
        extra,
    })
}

fn dynamic_expense_field(key: &str) -> Option<(&str, i64)> {
    ["payer_", "share_", "exact_"]
        .into_iter()
        .find_map(|prefix| {
            key.strip_prefix(prefix)
                .and_then(|id| id.parse::<i64>().ok())
                .map(|id| (prefix, id))
        })
}

fn malformed_form(message: &'static str) -> FormError {
    FormError {
        status: StatusCode::BAD_REQUEST,
        message,
    }
}

fn value(fields: &[(&str, &str)], key: &str) -> String {
    fields
        .iter()
        .find(|(field, _)| *field == key)
        .map_or_else(String::new, |(_, value)| (*value).to_owned())
}

fn scalar(values: &HashMap<String, String>, key: &str) -> String {
    values.get(key).cloned().unwrap_or_default()
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
        validate_percent_encoding(&bytes).map_err(|()| {
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
#[allow(clippy::expect_used)]
mod tests {
    use axum::http::StatusCode;

    use super::{
        OrderedForm, parse_expense_form, parse_group_form, parse_member_form,
        validate_percent_encoding,
    };

    #[test]
    fn rejects_incomplete_or_non_hex_percent_encoding() {
        assert!(validate_percent_encoding(b"value=%").is_err());
        assert!(validate_percent_encoding(b"value=%0").is_err());
        assert!(validate_percent_encoding(b"value=%xz").is_err());
        assert!(validate_percent_encoding(b"value=%20").is_ok());
    }

    #[test]
    fn parsers_reject_duplicate_and_unknown_fields() {
        let duplicate = OrderedForm(vec![
            ("csrf".into(), "first".into()),
            ("csrf".into(), "second".into()),
        ]);
        let unknown = OrderedForm(vec![
            ("name".into(), "Trip".into()),
            ("currency".into(), "USD".into()),
            ("csrf".into(), "token".into()),
            ("extra".into(), "value".into()),
        ]);

        assert!(duplicate.required_fields(&["csrf"]).is_err());
        assert_eq!(
            parse_group_form(unknown).err().map(|error| error.status),
            Some(StatusCode::BAD_REQUEST)
        );
    }

    #[test]
    fn member_parser_reports_invalid_ids_as_validation_errors() {
        let form = OrderedForm(vec![
            ("participant_id".into(), "not-an-id".into()),
            ("csrf".into(), "token".into()),
        ]);

        assert_eq!(
            parse_member_form(form).err().map(|error| error.status),
            Some(StatusCode::UNPROCESSABLE_ENTITY)
        );
    }

    fn expense_fields() -> OrderedForm {
        OrderedForm(vec![
            ("description".into(), "Lunch".into()),
            ("total".into(), "12.00".into()),
            ("currency".into(), "USD".into()),
            ("spending_type".into(), "other".into()),
            ("spent_date".into(), "2025-01-01".into()),
            ("payer_mode".into(), "single".into()),
            ("single_payer_id".into(), "1".into()),
            ("split_mode".into(), "equal".into()),
            ("csrf".into(), "token".into()),
        ])
    }

    #[test]
    fn expense_parser_accepts_structural_dynamic_fields_without_eligibility_policy() {
        let mut form = expense_fields();
        form.0.push(("payer_1".into(), "12.00".into()));
        form.0.push(("share_2".into(), "on".into()));

        let parsed = parse_expense_form(form).expect("valid expense form");
        assert_eq!(
            parsed.extra.get("payer_1").map(String::as_str),
            Some("12.00")
        );
        assert_eq!(parsed.extra.get("share_2").map(String::as_str), Some("on"));
    }

    #[test]
    fn expense_parser_rejects_duplicate_unknown_and_malformed_dynamic_fields() {
        for field in ["payer_1", "payer_not-an-id", "unexpected"] {
            let mut form = expense_fields();
            form.0.push((field.into(), "12.00".into()));
            if field == "payer_1" {
                form.0.push((field.into(), "13.00".into()));
            }
            assert_eq!(
                parse_expense_form(form).err().map(|error| error.status),
                Some(StatusCode::BAD_REQUEST),
                "{field}"
            );
        }
    }
}
