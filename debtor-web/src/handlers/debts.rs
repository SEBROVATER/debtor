use std::collections::BTreeMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Response,
};
use debtor_application::RateMode;
use tower_sessions::Session;

use super::{
    DebtQuery,
    auth::require_auth,
    response::{error_response, map_error, render},
};
use crate::{
    state::AppState,
    templates::{DebtsTemplate, RateRow, TransferRow},
};

pub(crate) async fn debts(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Query(query): Query<DebtQuery>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let mode = match query.rate_mode.as_deref() {
        None | Some("historical") => RateMode::Historical,
        Some("current") => RateMode::Current,
        Some(_) => return error_response(StatusCode::BAD_REQUEST, "Unknown rate mode."),
    };
    let result = match state.debts.calculate(id, mode).await {
        Ok(value) => value,
        Err(error) => return map_error(error),
    };
    let members = match state.participants.members(id).await {
        Ok(value) => value,
        Err(error) => return map_error(error),
    };
    let names: BTreeMap<_, _> = members
        .into_iter()
        .map(|(p, _)| (p.id, p.name.to_string()))
        .collect();
    let warning = result
        .rates
        .iter()
        .any(|r| r.is_stale || r.is_provisional)
        .then(|| "Some conversions use stale or provisional rates.".to_string());
    render(&DebtsTemplate {
        currency: result.currency.to_string(),
        transfers: result
            .transfers
            .into_iter()
            .map(|t| TransferRow {
                from: names
                    .get(&t.from_participant_id)
                    .cloned()
                    .unwrap_or_else(|| format!("Participant {}", t.from_participant_id)),
                to: names
                    .get(&t.to_participant_id)
                    .cloned()
                    .unwrap_or_else(|| format!("Participant {}", t.to_participant_id)),
                amount: t.amount.to_string(),
            })
            .collect(),
        mode: if mode == RateMode::Current {
            "current".into()
        } else {
            "historical".into()
        },
        warning,
        calculated_at: result.calculated_at.to_rfc3339(),
        rates: result
            .rates
            .into_iter()
            .map(|r| RateRow {
                base: r.base.to_string(),
                quote: r.quote.to_string(),
                requested_date: r.requested_date.to_string(),
                effective_date: r.effective_date.to_string(),
                rate: r.rate.to_string(),
                stale: r.is_stale,
                provisional: r.is_provisional,
            })
            .collect(),
    })
}
