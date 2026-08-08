use std::collections::{BTreeMap, BTreeSet, HashMap};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use chrono::NaiveDate;
use debtor_application::{
    PayerInput, ShareInput, SpendingCursor, SpendingInput, SpendingPageDirection,
};
use debtor_domain::{
    currency::Currency,
    expenses::{PayerMode, ShareMode, infer_payer_mode, infer_share_mode},
    model::{Allocation, Spending, SpendingType},
};
use tower_sessions::Session;

use super::{
    ExpenseForm,
    auth::{csrf, require_auth},
    groups::require_writable_group,
    response::{error_response, map_error, render},
};
use crate::{
    forms::{CsrfValidatedForm, parse_expense_form},
    participant_color::suggested_participant_color,
    state::AppState,
    templates::{
        AllocationRow, ConfirmTemplate, ExpenseFormView, GroupTemplate, MemberRow, SelectOption,
        SpendingDetailTemplate, SpendingRow,
    },
};

pub(crate) async fn create_spending(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    form: CsrfValidatedForm,
) -> Response {
    save_spending(state, session, id, None, form).await
}
pub(crate) async fn update_spending(
    State(state): State<AppState>,
    session: Session,
    Path((group_id, spending_id)): Path<(i64, i64)>,
    form: CsrfValidatedForm,
) -> Response {
    save_spending(state, session, group_id, Some(spending_id), form).await
}

pub(crate) async fn edit_spending_form(
    State(state): State<AppState>,
    session: Session,
    Path((group_id, spending_id)): Path<(i64, i64)>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    if let Err(response) = require_writable_group(&state, group_id).await {
        return response;
    }
    let spending = match state.spendings.spending(group_id, spending_id).await {
        Ok(value) => value,
        Err(error) => return map_error(error),
    };
    match build_group_template(
        &state,
        &session,
        group_id,
        None,
        Some(&spending),
        None,
        None,
        None,
    )
    .await
    {
        Ok(template) => render(&template),
        Err(error) => map_group_template_error(error),
    }
}

pub(crate) async fn spending_detail(
    State(state): State<AppState>,
    session: Session,
    Path((group_id, spending_id)): Path<(i64, i64)>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let group = match state.groups.group(group_id).await {
        Ok(value) => value,
        Err(error) => return map_error(error),
    };
    let spending = match state.spendings.spending(group_id, spending_id).await {
        Ok(value) => value,
        Err(error) => return map_error(error),
    };
    let members = match state.participants.members(group_id).await {
        Ok(value) => value,
        Err(error) => return map_error(error),
    };
    let names = members
        .into_iter()
        .map(|(p, _)| (p.id, p.name.to_string()))
        .collect();
    render(&SpendingDetailTemplate {
        group_id,
        spending_id,
        archived: group.is_archived,
        description: spending.description.as_str().to_owned(),
        total: spending.total.to_string(),
        currency: spending.currency.to_string(),
        spending_type: spending.spending_type.to_string(),
        spent_date: spending.spent_date.to_string(),
        payers: named_allocations(&spending.payers, &names),
        shares: named_allocations(&spending.shares, &names),
        csrf: match csrf(&session).await {
            Ok(token) => token,
            Err(response) => return response,
        },
    })
}

pub(crate) async fn delete_spending_form(
    State(state): State<AppState>,
    session: Session,
    Path((group_id, spending_id)): Path<(i64, i64)>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    if let Err(response) = require_writable_group(&state, group_id).await {
        return response;
    }
    let spending = match state.spendings.spending(group_id, spending_id).await {
        Ok(value) => value,
        Err(error) => return map_error(error),
    };
    render(&ConfirmTemplate {
        heading: "Delete expense".into(),
        message: format!(
            "Delete '{}' ({} {})?",
            spending.description.as_str(),
            spending.total,
            spending.currency
        ),
        action: format!("/groups/{group_id}/spendings/{spending_id}/delete"),
        cancel: format!("/groups/{group_id}/spendings/{spending_id}"),
        csrf: match csrf(&session).await {
            Ok(token) => token,
            Err(response) => return response,
        },
    })
}

pub(crate) async fn delete_spending(
    State(state): State<AppState>,
    session: Session,
    Path((group_id, spending_id)): Path<(i64, i64)>,
    _form: CsrfValidatedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    if let Err(response) = require_writable_group(&state, group_id).await {
        return response;
    }
    match state.spendings.delete(group_id, spending_id).await {
        Ok(()) => Redirect::to(&format!("/groups/{group_id}")).into_response(),
        Err(error) => map_error(error),
    }
}

async fn save_spending(
    state: AppState,
    session: Session,
    group_id: i64,
    spending_id: Option<i64>,
    form: CsrfValidatedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let form = form.into_inner();
    if let Err(response) = require_writable_group(&state, group_id).await {
        return response;
    }
    let editing = if let Some(id) = spending_id {
        let spending = match state.spendings.spending(group_id, id).await {
            Ok(spending) => spending,
            Err(error) => return map_error(error),
        };
        Some(spending)
    } else {
        None
    };
    let form = match parse_expense_form(form) {
        Ok(value) => value,
        Err(error) => return error_response(error.status, error.message),
    };
    let input = match parse_expense(group_id, &form) {
        Ok(value) => value,
        Err(message) => {
            return form_error(
                &state,
                &session,
                group_id,
                None,
                editing.as_ref(),
                message,
                &form,
            )
            .await;
        }
    };
    let result = if let Some(id) = spending_id {
        state.spendings.update_input(id, input).await
    } else {
        state.spendings.create_input(input).await
    };
    match result {
        Ok(_) => Redirect::to(&format!("/groups/{group_id}")).into_response(),
        Err(debtor_application::ApplicationError::Validation(error)) => {
            form_error(
                &state,
                &session,
                group_id,
                None,
                editing.as_ref(),
                error.to_string(),
                &form,
            )
            .await
        }
        Err(error) => map_error(error),
    }
}

async fn form_error(
    state: &AppState,
    session: &Session,
    group_id: i64,
    cursor: Option<SpendingCursor>,
    editing: Option<&Spending>,
    message: String,
    form: &ExpenseForm,
) -> Response {
    match build_group_template(
        state,
        session,
        group_id,
        cursor,
        editing,
        Some(message),
        Some(form),
        None,
    )
    .await
    {
        Ok(template) => (StatusCode::UNPROCESSABLE_ENTITY, render(&template)).into_response(),
        Err(error) => map_group_template_error(error),
    }
}

fn parse_expense(group_id: i64, form: &ExpenseForm) -> Result<SpendingInput, String> {
    let payers = match form.payer_mode.as_str() {
        "single" => PayerInput::Single(form.single_payer_id.unwrap_or_default()),
        "multiple" => PayerInput::Exact(raw_allocations(&form.extra, "payer_")),
        _ => return Err("Choose how many people paid.".into()),
    };
    let shares = match form.split_mode.as_str() {
        "equal" => ShareInput::Equal(raw_ids(&form.extra, "share_")),
        "exact" => ShareInput::Exact(raw_allocations(&form.extra, "exact_")),
        _ => return Err("Choose how the expense is split.".into()),
    };
    Ok(SpendingInput {
        group_id,
        description: form.description.clone(),
        total: form.total.clone(),
        currency: form.currency.clone(),
        spending_type: form.spending_type.clone(),
        spent_date: form.spent_date.clone(),
        payers,
        shares,
    })
}

fn raw_ids(fields: &HashMap<String, String>, prefix: &str) -> Vec<i64> {
    let mut ids = fields
        .keys()
        .filter_map(|key| key.strip_prefix(prefix).and_then(|id| id.parse().ok()))
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids
}

fn raw_allocations(fields: &HashMap<String, String>, prefix: &str) -> Vec<(i64, String)> {
    let mut values = fields
        .iter()
        .filter_map(|(key, value)| {
            key.strip_prefix(prefix)
                .and_then(|id| id.parse().ok())
                .map(|id| (id, value.clone()))
        })
        .filter(|(_, value)| !value.trim().is_empty())
        .collect::<Vec<_>>();
    values.sort_unstable_by_key(|(id, _)| *id);
    values
}

pub(super) fn parse_cursor(raw: Option<&str>) -> Result<Option<SpendingCursor>, &'static str> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    let mut fields = raw.split(':');
    let direction = match fields.next() {
        Some("older") => SpendingPageDirection::Older,
        Some("newer") => SpendingPageDirection::Newer,
        _ => return Err("Invalid spending history cursor."),
    };
    let spent_date = fields
        .next()
        .and_then(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
        .ok_or("Invalid spending history cursor.")?;
    let id = fields
        .next()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
        .ok_or("Invalid spending history cursor.")?;
    if fields.next().is_some() {
        return Err("Invalid spending history cursor.");
    }
    Ok(Some(SpendingCursor {
        direction,
        spent_date,
        id,
    }))
}

fn encode_cursor(cursor: SpendingCursor) -> String {
    let direction = match cursor.direction {
        SpendingPageDirection::Older => "older",
        SpendingPageDirection::Newer => "newer",
    };
    format!("{direction}:{}:{}", cursor.spent_date, cursor.id)
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn build_group_template(
    state: &AppState,
    session: &Session,
    id: i64,
    cursor: Option<SpendingCursor>,
    editing: Option<&Spending>,
    error: Option<String>,
    submitted: Option<&ExpenseForm>,
    participant_draft: Option<ParticipantDraft>,
) -> Result<GroupTemplate, GroupTemplateError> {
    let group = state.groups.group(id).await?;
    let members = state.participants.members(id).await?;
    let all_participants = state.participants.list_participants(false).await?;
    let active_ids: BTreeSet<_> = members
        .iter()
        .filter(|(_, m)| m.is_active)
        .map(|(p, _)| p.id)
        .collect();
    let inactive_ids: BTreeSet<_> = members
        .iter()
        .filter(|(_, m)| !m.is_active)
        .map(|(p, _)| p.id)
        .collect();
    let active_members: Vec<MemberRow> = members
        .iter()
        .filter(|(p, m)| m.is_active && !p.is_archived)
        .map(|(p, _)| member_row(p, true, false))
        .collect();
    let inactive_members = members
        .iter()
        .filter(|(p, m)| !m.is_active && !p.is_archived)
        .map(|(p, _)| member_row(p, false, false))
        .collect();
    let available = all_participants
        .into_iter()
        .filter(|p| !active_ids.contains(&p.id) && !inactive_ids.contains(&p.id))
        .map(|p| member_row(&p, false, false))
        .collect();
    let had_cursor = cursor.is_some();
    let spending_page = state.spendings.spending_page(id, cursor).await?;
    let show_newest_spendings = had_cursor && spending_page.items.is_empty();
    let mut form_members = active_members.clone();
    if let Some(spending) = editing {
        let historical_ids: BTreeSet<_> = spending
            .payers
            .iter()
            .chain(&spending.shares)
            .map(|a| a.participant_id)
            .collect();
        for (participant, membership) in &members {
            if historical_ids.contains(&participant.id)
                && !form_members
                    .iter()
                    .any(|member| member.id == participant.id)
            {
                form_members.push(member_row(participant, membership.is_active, false));
            }
        }
    }
    let mut expense = expense_view(state, group.currency, &form_members, editing, submitted);
    if editing.is_none() {
        expense.action = format!("/groups/{id}/spendings");
    }
    let (create_name, create_color) = participant_draft.map_or_else(
        || (String::new(), suggested_participant_color().to_owned()),
        |draft| (draft.name, draft.color),
    );
    Ok(GroupTemplate {
        name: group.name.to_string(),
        group_id: id,
        currency: group.currency.to_string(),
        csrf: csrf(session)
            .await
            .map_err(|_| GroupTemplateError::Session)?,
        members: active_members,
        inactive_members,
        available_participants: available,
        spendings: spending_page
            .items
            .into_iter()
            .map(|s| SpendingRow {
                id: s.id,
                description: s.description.as_str().to_owned(),
                total: s.total.to_string(),
                currency: s.currency.to_string(),
                spent_date: s.spent_date.to_string(),
            })
            .collect(),
        older_spendings: spending_page.older.map(encode_cursor),
        newer_spendings: spending_page.newer.map(encode_cursor),
        show_newest_spendings,
        archived: group.is_archived,
        error,
        create_name,
        create_color,
        expense,
    })
}

pub(super) struct ParticipantDraft {
    pub(super) name: String,
    pub(super) color: String,
}

pub(super) enum GroupTemplateError {
    Application(debtor_application::ApplicationError),
    Session,
}

impl From<debtor_application::ApplicationError> for GroupTemplateError {
    fn from(error: debtor_application::ApplicationError) -> Self {
        Self::Application(error)
    }
}

pub(super) fn map_group_template_error(error: GroupTemplateError) -> Response {
    match error {
        GroupTemplateError::Application(error) => map_error(error),
        GroupTemplateError::Session => super::response::session_error(),
    }
}
fn member_row(p: &debtor_domain::model::Participant, active: bool, selected: bool) -> MemberRow {
    MemberRow {
        id: p.id,
        name: p.name.to_string(),
        color: p.color.as_str().to_owned(),
        active,
        archived: p.is_archived,
        selected,
        amount: String::new(),
    }
}
#[allow(clippy::too_many_lines)]
fn expense_view(
    state: &AppState,
    currency: Currency,
    members: &[MemberRow],
    spending: Option<&Spending>,
    submitted: Option<&ExpenseForm>,
) -> ExpenseFormView {
    let mut view = ExpenseFormView {
        action: String::new(),
        heading: if spending.is_some() {
            "Edit expense".into()
        } else {
            "Add expense".into()
        },
        submit_label: if spending.is_some() {
            "Save expense".into()
        } else {
            "Add expense".into()
        },
        description: String::new(),
        total: String::new(),
        currency: currency.to_string(),
        currencies: Currency::ALL
            .iter()
            .map(|c| SelectOption {
                value: c.to_string(),
                label: c.to_string(),
                selected: *c == currency,
            })
            .collect(),
        spending_type: "other".into(),
        categories: SpendingType::ALL
            .iter()
            .map(|c| SelectOption {
                value: c.code().into(),
                label: c.code().to_string(),
                selected: *c == SpendingType::Other,
            })
            .collect(),
        spent_date: state.clock.now().date_naive().to_string(),
        payer_mode: "single".into(),
        split_mode: "equal".into(),
        single_payer_id: if members.len() == 1 { members[0].id } else { 0 },
        payer_rows: members.to_vec(),
        share_rows: members
            .iter()
            .map(|m| {
                let mut row = m.clone();
                row.selected = true;
                row
            })
            .collect(),
        exact_rows: members.to_vec(),
        error: None,
    };
    if let Some(s) = spending {
        s.description.as_str().clone_into(&mut view.description);
        view.total = s.total.to_string();
        view.currency = s.currency.to_string();
        view.spending_type = s.spending_type.to_string();
        view.spent_date = s.spent_date.to_string();
        view.action = format!("/groups/{}/spendings/{}", s.group_id, s.id);
        if infer_payer_mode(s) == PayerMode::Single {
            view.single_payer_id = s.payers[0].participant_id;
        } else {
            view.payer_mode = "multiple".into();
        }
        view.split_mode = match infer_share_mode(s) {
            ShareMode::Equal => "equal",
            ShareMode::Exact => "exact",
        }
        .into();
        let payer_map: BTreeMap<_, _> = s
            .payers
            .iter()
            .map(|a| (a.participant_id, a.amount.to_string()))
            .collect();
        view.payer_rows
            .iter_mut()
            .for_each(|m| m.amount = payer_map.get(&m.id).cloned().unwrap_or_default());
        let share_map: BTreeSet<_> = s.shares.iter().map(|a| a.participant_id).collect();
        view.share_rows
            .iter_mut()
            .for_each(|m| m.selected = share_map.contains(&m.id));
        let exact_map: BTreeMap<_, _> = s
            .shares
            .iter()
            .map(|a| (a.participant_id, a.amount.to_string()))
            .collect();
        view.exact_rows
            .iter_mut()
            .for_each(|m| m.amount = exact_map.get(&m.id).cloned().unwrap_or_default());
    }
    if let Some(form) = submitted {
        apply_submitted_expense(&mut view, form);
    }
    view
}

fn apply_submitted_expense(view: &mut ExpenseFormView, form: &ExpenseForm) {
    view.description.clone_from(&form.description);
    view.total.clone_from(&form.total);
    view.currency.clone_from(&form.currency);
    view.spending_type.clone_from(&form.spending_type);
    view.spent_date.clone_from(&form.spent_date);
    view.payer_mode.clone_from(&form.payer_mode);
    view.split_mode.clone_from(&form.split_mode);
    view.single_payer_id = form.single_payer_id.unwrap_or(0);
    view.currencies
        .iter_mut()
        .for_each(|o| o.selected = o.value == view.currency);
    view.categories
        .iter_mut()
        .for_each(|o| o.selected = o.value == view.spending_type);
    view.payer_rows.iter_mut().for_each(|r| {
        r.amount = form
            .extra
            .get(&format!("payer_{}", r.id))
            .cloned()
            .unwrap_or_default();
    });
    view.share_rows
        .iter_mut()
        .for_each(|r| r.selected = form.extra.contains_key(&format!("share_{}", r.id)));
    view.exact_rows.iter_mut().for_each(|r| {
        r.amount = form
            .extra
            .get(&format!("exact_{}", r.id))
            .cloned()
            .unwrap_or_default();
    });
}
fn named_allocations(items: &[Allocation], names: &BTreeMap<i64, String>) -> Vec<AllocationRow> {
    items
        .iter()
        .map(|a| AllocationRow {
            participant: names
                .get(&a.participant_id)
                .cloned()
                .unwrap_or_else(|| format!("Participant {}", a.participant_id)),
            amount: a.amount.to_string(),
        })
        .collect()
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{SpendingPageDirection, parse_cursor};

    #[test]
    fn cursor_parser_accepts_only_strict_direction_date_and_positive_id() {
        let cursor = parse_cursor(Some("older:2026-01-02:7"))
            .expect("valid cursor")
            .expect("cursor value");
        assert_eq!(cursor.direction, SpendingPageDirection::Older);
        assert_eq!(cursor.id, 7);
        assert!(parse_cursor(Some("sideways:2026-01-02:7")).is_err());
        assert!(parse_cursor(Some("older:2026-01-02:0")).is_err());
        assert!(parse_cursor(Some("older:2026-01-02:7:extra")).is_err());
    }
}
