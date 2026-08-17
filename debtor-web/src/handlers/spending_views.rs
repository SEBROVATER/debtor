use std::collections::{BTreeMap, BTreeSet};

use axum::response::Response;
use debtor_application::SpendingCursor;
use debtor_domain::{
    currency::Currency,
    expenses::{PayerMode, ShareMode, infer_payer_mode, infer_share_mode},
    model::{Allocation, Spending, SpendingType},
};
use tower_sessions::Session;

use super::{ExpenseForm, auth::authenticated_shell, response::map_error};
use crate::{
    participant_color::suggested_participant_color,
    state::AppState,
    templates::{
        AllocationRow, ExpenseFormView, GroupTemplate, MemberRow, SelectOption, SpendingRow,
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
            exact_rows: Vec::new(),
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
        amount: String::new(),
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
        spending_type: "other".into(),
        categories: SpendingType::ALL
            .iter()
            .map(|category| SelectOption {
                value: category.code().into(),
                label: category.code().to_string(),
                selected: *category == SpendingType::Other,
            })
            .collect(),
        spent_date: state.clock.now().date_naive().to_string(),
        payer_mode: "single".into(),
        split_mode: "equal".into(),
        single_payer_id: if members.len() == 1 { members[0].id } else { 0 },
        payer_rows: members.to_vec(),
        share_rows: members
            .iter()
            .map(|member| {
                let mut row = member.clone();
                row.selected = true;
                row
            })
            .collect(),
        exact_rows: members.to_vec(),
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
        if infer_payer_mode(spending) == PayerMode::Single {
            view.single_payer_id = spending.payers[0].participant_id;
        } else {
            view.payer_mode = "multiple".into();
        }
        view.split_mode = match infer_share_mode(spending) {
            ShareMode::Equal => "equal",
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
        view.exact_rows.iter_mut().for_each(|member| {
            member.amount = exact_map.get(&member.id).cloned().unwrap_or_default();
        });
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
    view.share_rows
        .iter_mut()
        .for_each(|row| row.selected = form.extra.contains_key(&format!("share_{}", row.id)));
    view.exact_rows.iter_mut().for_each(|row| {
        row.amount = form
            .extra
            .get(&format!("exact_{}", row.id))
            .cloned()
            .unwrap_or_default();
    });
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
