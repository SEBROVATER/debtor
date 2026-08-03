use std::collections::{BTreeMap, BTreeSet, HashMap};

use axum::{
    Form,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use chrono::NaiveDate;
use debtor_application::{EqualSpendingCommand, ExactSpendingCommand};
use debtor_domain::{
    currency::Currency,
    model::{Allocation, Spending, SpendingType},
};
use rust_decimal::Decimal;
use tower_sessions::Session;

use super::{
    ExpenseForm,
    auth::{csrf, require_auth, require_csrf},
    response::{error_response, map_error, render},
};
use crate::{
    forms::{OrderedForm, parse_expense_csrf, parse_expense_form},
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
    form: OrderedForm,
) -> Response {
    save_spending(state, session, id, None, form).await
}
pub(crate) async fn update_spending(
    State(state): State<AppState>,
    session: Session,
    Path((group_id, spending_id)): Path<(i64, i64)>,
    form: OrderedForm,
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
    let spending = match state.spendings.spending(group_id, spending_id).await {
        Ok(value) => value,
        Err(error) => return map_error(error),
    };
    match build_group_template(&state, &session, group_id, Some(&spending), None, None).await {
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
    Form(form): Form<super::CsrfForm>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    if let Err(response) = require_csrf(&session, &form.csrf).await {
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
    form: OrderedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let csrf_token = match parse_expense_csrf(&form) {
        Ok(value) => value,
        Err(error) => return error_response(error.status, error.message),
    };
    if let Err(response) = require_csrf(&session, &csrf_token).await {
        return response;
    }
    let mut member_ids = match state.participants.members(group_id).await {
        Ok(members) => members
            .into_iter()
            .filter(|(p, m)| m.is_active && !p.is_archived)
            .map(|(p, _)| p.id)
            .collect::<Vec<_>>(),
        Err(error) => return map_error(error),
    };
    let editing = if let Some(id) = spending_id {
        let spending = match state.spendings.spending(group_id, id).await {
            Ok(spending) => {
                member_ids.extend(
                    spending
                        .payers
                        .iter()
                        .chain(&spending.shares)
                        .map(|a| a.participant_id),
                );
                spending
            }
            Err(error) => return map_error(error),
        };
        member_ids.sort_unstable();
        member_ids.dedup();
        Some(spending)
    } else {
        None
    };
    let form = match parse_expense_form(form, &member_ids) {
        Ok(value) => value,
        Err(error) => return error_response(error.status, error.message),
    };
    let parsed = match parse_expense(group_id, &member_ids, &form) {
        Ok(value) => value,
        Err(message) => {
            return form_error(&state, &session, group_id, editing.as_ref(), message, &form).await;
        }
    };
    let result = match parsed {
        ParsedExpense::Equal(command) => {
            if let Some(id) = spending_id {
                state.spendings.update_equal(id, command).await
            } else {
                state.spendings.create_equal(command).await
            }
        }
        ParsedExpense::Exact(command) => {
            if let Some(id) = spending_id {
                state.spendings.update_exact(id, command).await
            } else {
                state.spendings.create_exact(command).await
            }
        }
    };
    match result {
        Ok(_) => Redirect::to(&format!("/groups/{group_id}")).into_response(),
        Err(debtor_application::ApplicationError::Validation(error)) => {
            form_error(
                &state,
                &session,
                group_id,
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
    editing: Option<&Spending>,
    message: String,
    form: &ExpenseForm,
) -> Response {
    match build_group_template(state, session, group_id, editing, Some(message), Some(form)).await {
        Ok(template) => (StatusCode::UNPROCESSABLE_ENTITY, render(&template)).into_response(),
        Err(error) => map_group_template_error(error),
    }
}

enum ParsedExpense {
    Equal(EqualSpendingCommand),
    Exact(ExactSpendingCommand),
}
fn parse_expense(
    group_id: i64,
    member_ids: &[i64],
    form: &ExpenseForm,
) -> Result<ParsedExpense, String> {
    let total = form
        .total
        .parse::<Decimal>()
        .map_err(|_| "Total must be a valid amount.".to_string())?;
    let currency = form
        .currency
        .parse::<Currency>()
        .map_err(|_| "Currency is invalid.".to_string())?;
    let spending_type = form
        .spending_type
        .parse::<SpendingType>()
        .map_err(|_| "Category is invalid.".to_string())?;
    let spent_date = NaiveDate::parse_from_str(&form.spent_date, "%Y-%m-%d")
        .map_err(|_| "Date must be a valid ISO date.".to_string())?;
    let payers = match form.payer_mode.as_str() {
        "single" => vec![Allocation {
            participant_id: form
                .single_payer_id
                .filter(|id| *id > 0)
                .ok_or("Choose who paid.")?,
            amount: total,
        }],
        "multiple" => strict_allocations(member_ids, &form.extra, "payer_", "paid amounts")?,
        _ => return Err("Choose how many people paid.".into()),
    };
    let share_participant_ids = member_ids
        .iter()
        .copied()
        .filter(|id| form.extra.contains_key(&format!("share_{id}")))
        .collect();
    match form.split_mode.as_str() {
        "equal" => Ok(ParsedExpense::Equal(EqualSpendingCommand {
            group_id,
            description: form.description.clone(),
            total,
            currency,
            spending_type,
            spent_date,
            payers,
            share_participant_ids,
        })),
        "exact" => Ok(ParsedExpense::Exact(ExactSpendingCommand {
            group_id,
            description: form.description.clone(),
            total,
            currency,
            spending_type,
            spent_date,
            payers,
            shares: strict_allocations(member_ids, &form.extra, "exact_", "owed amounts")?,
        })),
        _ => Err("Choose how the expense is split.".into()),
    }
}
fn strict_allocations(
    ids: &[i64],
    fields: &HashMap<String, String>,
    prefix: &str,
    field: &str,
) -> Result<Vec<Allocation>, String> {
    let mut seen = BTreeSet::new();
    let mut values = Vec::new();
    for id in ids {
        let raw = fields
            .get(&format!("{prefix}{id}"))
            .map_or("", String::as_str)
            .trim();
        if raw.is_empty() {
            continue;
        }
        if !seen.insert(*id) {
            return Err(format!("A participant appears twice in {field}."));
        }
        let amount = raw
            .parse::<Decimal>()
            .map_err(|_| format!("Every {field} entry must be a valid amount."))?;
        if amount <= Decimal::ZERO {
            return Err(format!("Every {field} entry must be positive."));
        }
        values.push(Allocation {
            participant_id: *id,
            amount,
        });
    }
    Ok(values)
}

pub(super) async fn build_group_template(
    state: &AppState,
    session: &Session,
    id: i64,
    editing: Option<&Spending>,
    error: Option<String>,
    submitted: Option<&ExpenseForm>,
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
    let spendings = state.spendings.list_spendings(id).await?;
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
        spendings: spendings
            .into_iter()
            .map(|s| SpendingRow {
                id: s.id,
                description: s.description.as_str().to_owned(),
                total: s.total.to_string(),
                currency: s.currency.to_string(),
                spent_date: s.spent_date.to_string(),
            })
            .collect(),
        archived: group.is_archived,
        error,
        create_name: String::new(),
        create_color: suggested_participant_color().to_owned(),
        expense,
    })
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
        if s.payers.len() == 1 && s.payers[0].amount == s.total {
            view.single_payer_id = s.payers[0].participant_id;
        } else {
            view.payer_mode = "multiple".into();
        }
        if s.shares.iter().all(|a| a.amount > Decimal::ZERO) {
            let ids = s
                .shares
                .iter()
                .map(|a| a.participant_id)
                .collect::<Vec<_>>();
            if let Ok(equal) =
                debtor_domain::expenses::splitting::equal_split(s.total, s.currency, &ids)
            {
                if equal == s.shares {
                    view.split_mode = "equal".into();
                } else {
                    view.split_mode = "exact".into();
                }
            }
        }
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
