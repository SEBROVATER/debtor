//! Askama template types.

use askama::Template;

/// Password gate page.
#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate<'a> {
    /// Generic error.
    pub error: Option<&'a str>,
    /// CSRF token.
    pub csrf: &'a str,
}

/// Group list page.
#[derive(Template)]
#[template(path = "groups.html")]
pub struct GroupsTemplate {
    /// Group rows.
    pub groups: Vec<GroupRow>,
    /// Token.
    pub csrf: String,
    /// Archive state.
    pub archived: bool,
}

/// Renderable group row.
pub struct GroupRow {
    /// ID.
    pub id: i64,
    /// Name.
    pub name: String,
    /// Currency.
    pub currency: String,
}

/// Debt view page.
#[derive(Template)]
#[template(path = "debts.html")]
pub struct DebtsTemplate {
    /// Currency.
    pub currency: String,
    /// Transfers.
    pub transfers: Vec<TransferRow>,
    /// Mode.
    pub mode: String,
    /// Warning.
    pub warning: Option<String>,
}

/// Renderable transfer row.
pub struct TransferRow {
    /// Payer.
    pub from: i64,
    /// Recipient.
    pub to: i64,
    /// Amount.
    pub amount: String,
}
