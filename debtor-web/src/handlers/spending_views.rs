use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use axum::response::Response;
use debtor_application::{SpendingCursor, parse_unsigned_decimal};
use debtor_domain::{
    currency::Currency,
    expenses::{ShareMode, infer_share_mode, splitting::equal_split},
    model::{Allocation, Spending, SpendingType, ValidationError},
};
use tower_sessions::Session;

use super::{ExpenseForm, auth::authenticated_shell, response::map_error};
use crate::{
    participant_color::suggested_participant_color,
    state::AppState,
    templates::{
        AllocationRow, ExpenseFormView, GroupTemplate, MemberRow, SelectOption,
        SpendingFormTemplate, SpendingRow,
    },
};

pub(super) struct ParticipantDraft {
    pub(super) name: String,
    pub(super) color: String,
}

pub(super) enum GroupTemplateError {
    Application(debtor_application::ApplicationError),
    Response(Response),
}

impl From<debtor_application::ApplicationError> for GroupTemplateError {
    fn from(error: debtor_application::ApplicationError) -> Self {
        Self::Application(error)
    }
}

pub(super) fn map_group_template_error(error: GroupTemplateError) -> Response {
    match error {
        GroupTemplateError::Application(error) => map_error(error),
        GroupTemplateError::Response(response) => response,
    }
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
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
    let active_members = members
        .iter()
        .filter(|(participant, member)| member.is_active && !participant.is_archived)
        .map(|(participant, _)| member_row(participant, true, false))
        .collect::<Vec<_>>();
    let inactive_members = members
        .iter()
        .filter(|(participant, member)| !member.is_active && !participant.is_archived)
        .map(|(participant, _)| member_row(participant, false, false))
        .collect();
    let had_cursor = cursor.is_some();
    let spending_page = state.spendings.spending_page(id, cursor).await?;
    let can_delete = spending_page.items.is_empty() && cursor.is_none();
    let show_newest_spendings = had_cursor && spending_page.items.is_empty();
    let mut form_members = active_members.clone();
    if let Some(spending) = editing {
        let historical_ids: BTreeSet<_> = spending
            .payers
            .iter()
            .chain(&spending.shares)
            .map(|allocation| allocation.participant_id)
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
    let shell = authenticated_shell(state, session)
        .await
        .map_err(GroupTemplateError::Response)?;
    Ok(GroupTemplate {
        name: group.name.to_string(),
        group_id: id,
        section: "summary".to_owned(),
        currency: group.currency.to_string(),
        settings_name: group.name.to_string(),
        settings_currency: group.currency.to_string(),
        settings_currencies: super::groups::currency_options(&group.currency.to_string()),
        settings_error: None,
        settings_notice: None,
        settings_invalid_field: None,
        csrf: shell.csrf.clone(),
        shell,
        members: active_members,
        inactive_members,
        spendings: spending_page
            .items
            .into_iter()
            .map(|spending| SpendingRow {
                id: spending.id,
                description: spending.description.as_str().to_owned(),
                total: spending.total.to_string(),
                currency: spending.currency.to_string(),
                spent_date: spending.spent_date.to_string(),
            })
            .collect(),
        older_spendings: spending_page.older.map(encode_cursor),
        newer_spendings: spending_page.newer.map(encode_cursor),
        show_newest_spendings,
        archived: group.is_archived,
        can_delete,
        error,
        participant_invalid_field: None,
        focus_participant: None,
        participant_notice: None,
        create_name,
        create_color,
        expense,
    })
}

pub(super) async fn build_group_manage_template(
    state: &AppState,
    session: &Session,
    id: i64,
    settings_draft: Option<(String, String)>,
    settings_error: Option<String>,
    settings_invalid_field: Option<String>,
    settings_notice: Option<String>,
) -> Result<GroupTemplate, GroupTemplateError> {
    let mut template =
        match build_group_template(state, session, id, None, None, None, None, None).await {
            Ok(template) => template,
            Err(_error) if settings_draft.is_some() || settings_error.is_some() => {
                build_group_settings_fallback(state, session, id).await?
            }
            Err(error) => return Err(error),
        };
    "manage".clone_into(&mut template.section);
    if let Some((name, currency)) = settings_draft {
        template.settings_name = name;
        template.settings_currency.clone_from(&currency);
        template.settings_currencies = super::groups::currency_options(&currency);
    }
    template.settings_error = settings_error;
    template.settings_invalid_field = settings_invalid_field;
    template.settings_notice = settings_notice;
    Ok(template)
}

pub(super) async fn build_spending_form_template(
    state: &AppState,
    session: &Session,
    id: i64,
    submitted: Option<&ExpenseForm>,
    spending: Option<&Spending>,
    status: Option<String>,
    reviewed: bool,
) -> Result<SpendingFormTemplate, GroupTemplateError> {
    let group = state.groups.group(id).await?;
    let mut page =
        build_group_template(state, session, id, None, spending, None, submitted, None).await?;
    let action = spending.map_or_else(
        || {
            if reviewed {
                format!("/groups/{id}/spendings")
            } else {
                format!("/groups/{id}/spendings/preview")
            }
        },
        |value| format!("/groups/{id}/spendings/{}", value.id),
    );
    page.expense.action.clone_from(&action);
    Ok(SpendingFormTemplate {
        group_name: group.name.to_string(),
        group_id: id,
        shell: page.shell,
        expense: page.expense,
        action,
        reviewed,
        status,
        focus_heading: true,
    })
}

async fn build_group_settings_fallback(
    state: &AppState,
    session: &Session,
    id: i64,
) -> Result<GroupTemplate, GroupTemplateError> {
    let group = state.groups.group(id).await?;
    let shell = authenticated_shell(state, session)
        .await
        .map_err(GroupTemplateError::Response)?;
    let currency = group.currency.to_string();
    Ok(GroupTemplate {
        name: group.name.to_string(),
        group_id: id,
        section: "manage".to_owned(),
        currency: currency.clone(),
        settings_name: group.name.to_string(),
        settings_currency: currency.clone(),
        settings_currencies: super::groups::currency_options(&currency),
        settings_error: None,
        settings_notice: None,
        settings_invalid_field: None,
        csrf: shell.csrf.clone(),
        shell,
        members: Vec::new(),
        inactive_members: Vec::new(),
        spendings: Vec::new(),
        older_spendings: None,
        newer_spendings: None,
        show_newest_spendings: false,
        archived: group.is_archived,
        can_delete: false,
        error: None,
        participant_invalid_field: None,
        focus_participant: None,
        participant_notice: None,
        create_name: String::new(),
        create_color: String::new(),
        expense: ExpenseFormView {
            action: format!("/groups/{id}/spendings"),
            heading: String::new(),
            submit_label: String::new(),
            description: String::new(),
            total: String::new(),
            currency: currency.clone(),
            currencies: super::groups::currency_options(&currency),
            spending_type: String::new(),
            categories: Vec::new(),
            spent_date: String::new(),
            payer_mode: String::new(),
            split_mode: String::new(),
            single_payer_id: 0,
            payer_rows: Vec::new(),
            share_rows: Vec::new(),
            allocation_status: String::new(),
            error: None,
        },
    })
}

fn encode_cursor(cursor: SpendingCursor) -> String {
    let direction = match cursor.direction {
        debtor_application::SpendingPageDirection::Older => "older",
        debtor_application::SpendingPageDirection::Newer => "newer",
    };
    format!("{direction}:{}:{}", cursor.spent_date, cursor.id)
}

fn member_row(
    participant: &debtor_domain::model::Participant,
    active: bool,
    selected: bool,
) -> MemberRow {
    MemberRow {
        id: participant.id,
        name: participant.name.to_string(),
        color: participant.color.as_str().to_owned(),
        active,
        archived: participant.is_archived,
        selected,
        allocation_error: None,
        amount: String::new(),
        derived_amount: String::new(),
        editing: false,
        edit_name: participant.name.to_string(),
        edit_color: participant.color.as_str().to_owned(),
        edit_error: None,
        edit_invalid_field: None,
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
            "Edit expense"
        } else {
            "Add expense"
        }
        .into(),
        submit_label: if spending.is_some() {
            "Save expense"
        } else {
            "Add expense"
        }
        .into(),
        description: String::new(),
        total: String::new(),
        currency: currency.to_string(),
        currencies: Currency::ALL
            .iter()
            .map(|option| SelectOption {
                value: option.to_string(),
                label: option.to_string(),
                selected: *option == currency,
            })
            .collect(),
        spending_type: String::new(),
        categories: SpendingType::ALL
            .iter()
            .map(|category| SelectOption {
                value: category.code().into(),
                label: category.code().to_string(),
                selected: false,
            })
            .collect(),
        spent_date: state.clock.now().date_naive().to_string(),
        payer_mode: "single".into(),
        split_mode: "proportional".into(),
        single_payer_id: 0,
        payer_rows: members.to_vec(),
        share_rows: members
            .iter()
            .map(|member| {
                let mut row = member.clone();
                row.selected = true;
                row.amount = "1".into();
                row
            })
            .collect(),
        allocation_status: String::new(),
        error: None,
    };
    if let Some(spending) = spending {
        spending
            .description
            .as_str()
            .clone_into(&mut view.description);
        view.total = spending.total.to_string();
        view.currency = spending.currency.to_string();
        view.spending_type = spending.spending_type.to_string();
        view.spent_date = spending.spent_date.to_string();
        view.action = format!("/groups/{}/spendings/{}", spending.group_id, spending.id);
        view.single_payer_id = spending.payers[0].participant_id;
        view.split_mode = match infer_share_mode(spending) {
            ShareMode::Exact => "exact",
        }
        .into();
        let payer_map: BTreeMap<_, _> = spending
            .payers
            .iter()
            .map(|allocation| (allocation.participant_id, allocation.amount.to_string()))
            .collect();
        view.payer_rows.iter_mut().for_each(|member| {
            member.amount = payer_map.get(&member.id).cloned().unwrap_or_default();
        });
        let share_map: BTreeSet<_> = spending
            .shares
            .iter()
            .map(|allocation| allocation.participant_id)
            .collect();
        view.share_rows
            .iter_mut()
            .for_each(|member| member.selected = share_map.contains(&member.id));
        let exact_map: BTreeMap<_, _> = spending
            .shares
            .iter()
            .map(|allocation| (allocation.participant_id, allocation.amount.to_string()))
            .collect();
        view.share_rows.iter_mut().for_each(|member| {
            member.amount = exact_map.get(&member.id).cloned().unwrap_or_default();
        });
        view.allocation_status = "Exact Shares must equal the Total.".into();
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
        .for_each(|option| option.selected = option.value == view.currency);
    view.categories
        .iter_mut()
        .for_each(|option| option.selected = option.value == view.spending_type);
    view.payer_rows.iter_mut().for_each(|row| {
        row.amount = form
            .extra
            .get(&format!("payer_{}", row.id))
            .cloned()
            .unwrap_or_default();
    });
    view.share_rows.iter_mut().for_each(|row| {
        if form.split_mode == "proportional" {
            row.selected = form.extra.contains_key(&format!("included_{}", row.id));
            row.amount = form
                .extra
                .get(&format!("weight_{}", row.id))
                .cloned()
                .unwrap_or_default();
        } else {
            row.selected = form.extra.contains_key(&format!("included_{}", row.id));
            row.amount = form
                .extra
                .get(&format!("exact_{}", row.id))
                .cloned()
                .unwrap_or_default();
        }
        row.allocation_error = if form.split_mode == "exact" && row.selected {
            match parse_unsigned_decimal(&row.amount, "owed amount") {
                Ok(value) if !value.is_zero() => None,
                Ok(_) => Some("Share must be greater than zero.".into()),
                Err(_) => Some("Enter a valid exact Share.".into()),
            }
        } else {
            None
        };
    });
    view.allocation_status = exact_allocation_status(form);
}

fn exact_allocation_status(form: &ExpenseForm) -> String {
    if form.split_mode != "exact" {
        return String::new();
    }
    let Ok(total) = parse_unsigned_decimal(&form.total, "total") else {
        return String::new();
    };
    let mut values = form
        .extra
        .iter()
        .filter(|(key, _)| key.starts_with("included_"))
        .filter_map(|(key, _)| {
            key.strip_prefix("included_")
                .and_then(|id| form.extra.get(&format!("exact_{id}")))
        })
        .filter_map(|value| parse_unsigned_decimal(value, "owed amount").ok())
        .collect::<Vec<_>>();
    let Some(mut sum) = values.pop() else {
        return String::new();
    };
    for value in values {
        sum = match sum.checked_add(value) {
            Some(value) => value,
            None => return "Exact Share total is too large.".into(),
        };
    }
    let Some(difference) = total.checked_sub(sum) else {
        return "Exact Share difference is too large.".into();
    };
    if difference.is_sign_positive() && !difference.is_zero() {
        format!("Remaining: {difference} {}", form.currency)
    } else if difference.is_sign_negative() {
        let amount = difference.to_string();
        let amount = amount.strip_prefix('-').unwrap_or(&amount);
        format!("Excess: {amount} {}", form.currency)
    } else {
        "Exact Shares close the Total.".into()
    }
}

pub(super) fn initialize_exact_defaults(
    form: &mut ExpenseForm,
    member_ids: &[i64],
) -> Result<(), String> {
    if form.split_mode != "exact" {
        return Ok(());
    }
    let selected = member_ids
        .iter()
        .copied()
        .filter(|id| form.extra.contains_key(&format!("included_{id}")))
        .collect::<Vec<_>>();
    if selected.is_empty()
        || selected.iter().any(|id| {
            form.extra
                .get(&format!("exact_{id}"))
                .is_some_and(|value| !value.trim().is_empty())
        })
    {
        return Ok(());
    }
    let total = parse_unsigned_decimal(&form.total, "total")
        .map_err(|_| "Enter a valid Total.".to_owned())?;
    let currency = Currency::from_str(&form.currency)
        .map_err(|_| "Choose a supported Source Currency.".to_owned())?;
    let allocations = match equal_split(total, currency, &selected) {
        Ok(allocations) => allocations,
        Err(ValidationError::InsufficientMinorUnits { .. }) => {
            for participant_id in selected {
                form.extra
                    .insert(format!("exact_{participant_id}"), "0".into());
            }
            return Ok(());
        }
        Err(error) => return Err(error.to_string()),
    };
    for allocation in allocations {
        form.extra.insert(
            format!("exact_{}", allocation.participant_id),
            allocation.amount.to_string(),
        );
    }
    Ok(())
}

pub(super) fn named_allocations(
    items: &[Allocation],
    names: &BTreeMap<i64, String>,
) -> Vec<AllocationRow> {
    items
        .iter()
        .map(|allocation| AllocationRow {
            participant: names
                .get(&allocation.participant_id)
                .cloned()
                .unwrap_or_else(|| format!("Participant {}", allocation.participant_id)),
            amount: allocation.amount.to_string(),
        })
        .collect()
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::collections::HashMap;

    use super::{ExpenseForm, exact_allocation_status};

    fn exact_form(total: &str, amounts: &[(&str, &str)]) -> ExpenseForm {
        let mut extra = HashMap::new();
        for (id, amount) in amounts {
            extra.insert(format!("included_{id}"), "on".into());
            extra.insert(format!("exact_{id}"), (*amount).into());
        }
        ExpenseForm {
            description: String::new(),
            total: total.into(),
            currency: "USD".into(),
            spending_type: "food".into(),
            spent_date: "2026-08-17".into(),
            payer_mode: "single".into(),
            single_payer_id: Some(1),
            split_mode: "exact".into(),
            extra,
        }
    }

    #[test]
    fn exact_status_reports_remaining_and_excess() {
        assert_eq!(
            exact_allocation_status(&exact_form("10.00", &[("1", "4.00")])),
            "Remaining: 6.00 USD"
        );
        assert_eq!(
            exact_allocation_status(&exact_form("10.00", &[("1", "12.00")])),
            "Excess: 2.00 USD"
        );
        assert_eq!(
            exact_allocation_status(&exact_form("10.00", &[("1", "4.00"), ("2", "6.00")])),
            "Exact Shares close the Total."
        );
        assert_eq!(
            exact_allocation_status(&exact_form(
                "1",
                &[
                    ("1", "79228162514264337593543950335"),
                    ("2", "79228162514264337593543950335"),
                ]
            )),
            "Exact Share total is too large."
        );
    }
}
