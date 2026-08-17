//! HTTP request handlers.

mod auth;
mod debts;
mod groups;
mod health;
mod memberships;
pub(crate) mod response;
mod spending_views;
mod spendings;

#[cfg(test)]
pub(crate) mod test_support;

use serde::Deserialize;

pub(crate) use crate::forms::ExpenseForm;

pub(crate) use auth::{login, login_form, logout, root};
pub(crate) use debts::debts;
pub(crate) use groups::{
    archive_group, create_group, delete_group, delete_group_form, group_detail, group_edit_form,
    group_manage, group_transactions, groups, restore_group, update_group,
};
pub(crate) use health::{health, readiness};
pub(crate) use memberships::create_group_participant;
pub(crate) use spendings::{
    create_spending, delete_spending, delete_spending_form, edit_spending_form, spending_detail,
    update_spending,
};

#[derive(Deserialize)]
pub(crate) struct GroupsQuery {
    pub(super) archived: Option<bool>,
}

#[derive(Deserialize)]
pub(crate) struct SpendingQuery {
    pub(super) cursor: Option<String>,
}

#[derive(Deserialize, Default)]
pub(crate) struct ManageQuery {
    pub(super) saved: Option<String>,
    pub(super) participant: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct DebtQuery {
    pub(super) rate_mode: Option<String>,
}
