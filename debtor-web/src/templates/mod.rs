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

/// Participant list page.
#[derive(Template)]
#[template(path = "participants.html")]
pub struct ParticipantsTemplate {
    /// Participant rows.
    pub participants: Vec<ParticipantRow>,
    /// CSRF token.
    pub csrf: String,
    /// Whether this is the archive view.
    pub archived: bool,
}

/// Renderable participant row.
pub struct ParticipantRow {
    /// Database ID.
    pub id: i64,
    /// Name.
    pub name: String,
    /// Color.
    pub color: String,
}

/// Group spending page.
#[derive(Template)]
#[template(path = "group.html")]
pub struct GroupTemplate {
    /// Group name.
    pub name: String,
    /// Group identifier.
    pub group_id: i64,
    /// Target currency.
    pub currency: String,
    /// CSRF token.
    pub csrf: String,
    /// Active member rows.
    pub members: Vec<MemberRow>,
    /// Globally active participants not currently active in the group.
    pub available_participants: Vec<MemberRow>,
    /// Spending rows.
    pub spendings: Vec<SpendingRow>,
    /// Whether mutations are blocked.
    pub archived: bool,
}

/// Renderable active member.
pub struct MemberRow {
    /// Participant identifier.
    pub id: i64,
    /// Display name.
    pub name: String,
}

/// Renderable spending row.
pub struct SpendingRow {
    /// Spending identifier.
    pub id: i64,
    /// Description.
    pub description: String,
    /// Source amount.
    pub total: String,
    /// Source currency.
    pub currency: String,
    /// Spending date.
    pub spent_date: String,
}
