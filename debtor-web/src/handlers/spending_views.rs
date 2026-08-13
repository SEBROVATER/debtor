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
        .filter(|(_, member)| member.is_active)
        .map(|(participant, _)| participant.id)
        .collect();
    let inactive_ids: BTreeSet<_> = members
        .iter()
        .filter(|(_, member)| !member.is_active)
        .map(|(participant, _)| participant.id)
        .collect();
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
    let available = all_participants
        .into_iter()
        .filter(|participant| {
            !active_ids.contains(&participant.id) && !inactive_ids.contains(&participant.id)
        })
        .map(|participant| member_row(&participant, false, false))
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
        .map_err(|_| GroupTemplateError::Session)?;
    Ok(GroupTemplate {
        name: group.name.to_string(),
        group_id: id,
        currency: group.currency.to_string(),
        csrf: shell.csrf.clone(),
        shell,
        members: active_members,
        inactive_members,
        available_participants: available,
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
        create_name,
        create_color,
        expense,
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
