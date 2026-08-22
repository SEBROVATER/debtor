use axum::{
    extract::{Path, Query, State, rejection::QueryRejection},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use debtor_application::RateMode;
use tower_sessions::Session;

use super::{
    DebtQuery,
    auth::{authenticated_shell, require_auth},
    debt_mode,
    response::{debt_error_response, error_response, render},
};
use crate::{
    state::AppState,
    templates::{BalanceRow, DebtsResultsTemplate, DebtsTemplate, RateRow, TransferRow},
};

fn format_money(
    amount: impl std::fmt::Display,
    currency: debtor_domain::currency::Currency,
) -> String {
    format!(
        "{}{:.*} {}",
        currency.symbol(),
        currency.minor_unit_scale() as usize,
        amount,
        currency
    )
}

#[allow(clippy::too_many_lines)]
pub(crate) async fn debts(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
    Path(id): Path<i64>,
    query: Result<Query<DebtQuery>, QueryRejection>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let Ok(Query(query)) = query else {
        return crate::handlers::response::debt_mode_error_response(
            headers.contains_key("hx-request"),
        );
    };
    let Ok(mode) = debt_mode(query.rate_mode.as_deref()) else {
        return crate::handlers::response::debt_mode_error_response(
            headers.contains_key("hx-request"),
        );
    };
    let calculated_at = state.clock.now();
    let result = match state.debts.calculate(id, mode).await {
        Ok(value) => value,
        Err(error) => {
            return debt_error_response(
                error,
                mode,
                calculated_at,
                headers.contains_key("hx-request"),
            );
        }
    };
    let participants = result
        .participants
        .iter()
        .map(|(participant, _)| (participant.id, participant))
        .collect::<std::collections::BTreeMap<_, _>>();
    let balances = result
        .balances
        .iter()
        .map(|(participant_id, amount)| {
            let participant = participants.get(participant_id).ok_or(())?;
            let (direction, signed_amount) = if amount.is_zero() {
                (
                    "is settled".to_owned(),
                    format_money(*amount, result.currency),
                )
            } else if amount.is_sign_positive() {
                (
                    "is owed".to_owned(),
                    format!("+{}", format_money(*amount, result.currency)),
                )
            } else {
                ("owes".to_owned(), format_money(*amount, result.currency))
            };
            Ok(BalanceRow {
                participant: participant.name.to_string(),
                archived: participant.is_archived,
                color: participant.color.as_str().to_owned(),
                amount: signed_amount,
                direction,
            })
        })
        .collect::<Result<Vec<_>, ()>>();
    let Ok(balances) = balances else {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Unable to render debts.");
    };
    let participants = result
        .participants
        .into_iter()
        .map(|(participant, _)| {
            (
                participant.id,
                (participant.name.to_string(), participant.is_archived),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let has_stale = result.rates.iter().any(|rate| rate.is_stale);
    let has_provisional = result.rates.iter().any(|rate| rate.is_provisional);
    let has_synthetic = result.rates.iter().any(|rate| rate.base == rate.quote);
    let mut warnings = Vec::new();
    if has_stale {
        warnings.push("stale rates");
    }
    if has_provisional {
        warnings.push("provisional rates");
    }
    if has_synthetic {
        warnings.push("exact synthetic same-currency rates");
    }
    let warning =
        (!warnings.is_empty()).then(|| format!("Some conversions use {}.", warnings.join(" and ")));
    let status = match mode {
        RateMode::Historical => {
            "Calculation complete. Historical rates are selected by default.".to_owned()
        }
        RateMode::Current => {
            "Calculation complete. Current rates are selected for this result.".to_owned()
        }
    };
    let status = if let Some(warning) = &warning {
        format!("{status} {warning}")
    } else {
        status
    };
    let transfers = result
        .transfers
        .into_iter()
        .map(|transfer| {
            let (from, from_archived) = participants
                .get(&transfer.from_participant_id)
                .cloned()
                .ok_or(())?;
            let (to, to_archived) = participants
                .get(&transfer.to_participant_id)
                .cloned()
                .ok_or(())?;
            Ok(TransferRow {
                from,
                from_archived,
                to,
                to_archived,
                amount: format_money(transfer.amount, result.currency),
            })
        })
        .collect::<Result<Vec<_>, ()>>();
    let Ok(transfers) = transfers else {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Unable to render debts.");
    };
    let enhanced = headers.contains_key("hx-request");
    if enhanced {
        return render(&DebtsResultsTemplate {
            has_spendings: result.has_spendings,
            balances,
            transfers,
            mode: if mode == RateMode::Current {
                "current".into()
            } else {
                "historical".into()
            },
            warning,
            status,
            focus_results: false,
            calculated_at: result.calculated_at.to_rfc3339(),
            rates: result
                .rates
                .into_iter()
                .map(|r| RateRow {
                    base: r.base.to_string(),
                    quote: r.quote.to_string(),
                    requested_date: r.requested_date.to_string(),
                    fetch_date: r.fetch_date.to_string(),
                    effective_date: r.effective_date.to_string(),
                    rate: r.rate.to_string(),
                    stale: r.is_stale,
                    provisional: r.is_provisional,
                })
                .collect(),
        });
    }
    let shell = match authenticated_shell(&state, &session).await {
        Ok(shell) => shell,
        Err(response) => return response,
    };
    render(&DebtsTemplate {
        group_id: id,
        archived: result.group_is_archived,
        currency: result.currency.to_string(),
        has_spendings: result.has_spendings,
        balances,
        transfers,
        mode: if mode == RateMode::Current {
            "current".into()
        } else {
            "historical".into()
        },
        warning,
        status,
        focus_results: true,
        calculated_at: result.calculated_at.to_rfc3339(),
        rates: result
            .rates
            .into_iter()
            .map(|r| RateRow {
                base: r.base.to_string(),
                quote: r.quote.to_string(),
                requested_date: r.requested_date.to_string(),
                fetch_date: r.fetch_date.to_string(),
                effective_date: r.effective_date.to_string(),
                rate: r.rate.to_string(),
                stale: r.is_stale,
                provisional: r.is_provisional,
            })
            .collect(),
        shell,
    })
}
