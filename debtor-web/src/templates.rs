//! Askama template types.

use askama::Template;

/// Shared protection values for an authenticated page shell.
#[derive(Clone)]
pub struct AuthenticatedShell {
    /// Current synchronizer token.
    pub csrf: String,
    /// Single-use Sign out token.
    pub submission_token: String,
}

/// Password gate page.
#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate<'a> {
    /// Generic error.
    pub error: Option<&'a str>,
    /// CSRF token.
    pub csrf: &'a str,
    /// Single-use anonymous Login submission token.
    pub submission_token: &'a str,
    /// Whether forward/recovery focus should land on the heading.
    pub focus_heading: bool,
}

/// Generic escaped error page.
#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate<'a> {
    /// Status-safe message.
    pub message: &'a str,
    /// Whether the error can be retried from the anonymous Login route.
    pub login_recovery: bool,
}

/// Group list page.
#[derive(Template)]
#[template(path = "groups.html")]
pub struct GroupsTemplate {
    /// Group rows.
    pub groups: Vec<GroupRow>,
    /// Token.
    pub csrf: String,
    /// Shared authenticated shell protection values.
    pub shell: AuthenticatedShell,
    /// Archive state.
    pub archived: bool,
    /// Group name draft for the create form.
    pub create_name: String,
    /// Group currency draft for the create form.
    pub create_currency: String,
    /// Currency options for the create form.
    pub currencies: Vec<SelectOption>,
    /// Inline validation error.
    pub error: Option<String>,
}

/// Group settings page.
#[derive(Template)]
#[template(path = "group_edit.html")]
pub struct GroupEditTemplate {
    /// Group ID.
    pub id: i64,
    /// Name.
    pub name: String,
    /// Currency.
    pub currency: String,
    /// Currency options.
    pub currencies: Vec<SelectOption>,
    /// CSRF token.
    pub csrf: String,
    /// Shared authenticated shell protection values.
    pub shell: AuthenticatedShell,
    /// Error.
    pub error: Option<String>,
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
    /// Calculation timestamp.
    pub calculated_at: String,
    /// Unique rates used by the calculation.
    pub rates: Vec<RateRow>,
    /// Shared authenticated shell protection values.
    pub shell: AuthenticatedShell,
}

/// Renderable transfer row.
pub struct TransferRow {
    /// Payer.
    pub from: String,
    /// Recipient.
    pub to: String,
    /// Amount.
    pub amount: String,
}

/// Renderable exchange-rate disclosure row.
pub struct RateRow {
    /// Base currency.
    pub base: String,
    /// Target currency.
    pub quote: String,
    /// Requested date.
    pub requested_date: String,
    /// Provider effective date.
    pub effective_date: String,
    /// Exact rate.
    pub rate: String,
    /// Stale marker.
    pub stale: bool,
    /// Provisional marker.
    pub provisional: bool,
}

/// Participant list page.
#[derive(Template)]
#[template(path = "participants.html")]
pub struct ParticipantsTemplate {
    /// Participant rows.
    pub participants: Vec<ParticipantRow>,
    /// CSRF token.
    pub csrf: String,
    /// Shared authenticated shell protection values.
    pub shell: AuthenticatedShell,
    /// Whether this is the archive view.
    pub archived: bool,
    /// Participant name draft for the create form.
    pub create_name: String,
    /// Suggested color for a fresh participant form.
    pub create_color: String,
    /// Inline validation error.
    pub error: Option<String>,
}

/// Participant edit page.
#[derive(Template)]
#[template(path = "participant_edit.html")]
pub struct ParticipantEditTemplate {
    /// ID.
    pub id: i64,
    /// Name value.
    pub name: String,
    /// Color value.
    pub color: String,
    /// CSRF token.
    pub csrf: String,
    /// Shared authenticated shell protection values.
    pub shell: AuthenticatedShell,
    /// Error message.
    pub error: Option<String>,
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
    /// Shared authenticated shell protection values.
    pub shell: AuthenticatedShell,
    /// Active member rows.
    pub members: Vec<MemberRow>,
    /// Inactive memberships available for reactivation.
    pub inactive_members: Vec<MemberRow>,
    /// Globally active participants not currently active in the group.
    pub available_participants: Vec<MemberRow>,
    /// Spending rows.
    pub spendings: Vec<SpendingRow>,
    /// Cursor link for older spending rows.
    pub older_spendings: Option<String>,
    /// Cursor link for newer spending rows.
    pub newer_spendings: Option<String>,
    /// Whether an empty cursor page should offer the newest page.
    pub show_newest_spendings: bool,
    /// Whether mutations are blocked.
    pub archived: bool,
    /// Inline error.
    pub error: Option<String>,
    /// Participant name draft.
    pub create_name: String,
    /// Participant color draft.
    pub create_color: String,
    /// Expense form state.
    pub expense: ExpenseFormView,
}

/// Renderable shared expense form state.
pub struct ExpenseFormView {
    /// Form action.
    pub action: String,
    /// Heading.
    pub heading: String,
    /// Submit label.
    pub submit_label: String,
    /// Description.
    pub description: String,
    /// Total.
    pub total: String,
    /// Currency.
    pub currency: String,
    /// Currency options.
    pub currencies: Vec<SelectOption>,
    /// Category.
    pub spending_type: String,
    /// Category options.
    pub categories: Vec<SelectOption>,
    /// Date.
    pub spent_date: String,
    /// Payer mode.
    pub payer_mode: String,
    /// Split mode.
    pub split_mode: String,
    /// Selected single payer.
    pub single_payer_id: i64,
    /// Member payer rows.
    pub payer_rows: Vec<MemberRow>,
    /// Equal recipients.
    pub share_rows: Vec<MemberRow>,
    /// Exact owed rows.
    pub exact_rows: Vec<MemberRow>,
    /// Error message.
    pub error: Option<String>,
}

/// Select option.
pub struct SelectOption {
    /// Value.
    pub value: String,
    /// Display label.
    pub label: String,
    /// Selected state.
    pub selected: bool,
}

/// Renderable active member.
#[derive(Clone)]
pub struct MemberRow {
    /// Participant identifier.
    pub id: i64,
    /// Display name.
    pub name: String,
    /// Accent color.
    pub color: String,
    /// Active membership.
    pub active: bool,
    /// Archived identity.
    pub archived: bool,
    /// Selected in the current form.
    pub selected: bool,
    /// Draft amount.
    pub amount: String,
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

/// Read-only spending detail.
#[derive(Template)]
#[template(path = "spending_detail.html")]
pub struct SpendingDetailTemplate {
    /// Group ID.
    pub group_id: i64,
    /// Spending ID.
    pub spending_id: i64,
    /// Group archived status.
    pub archived: bool,
    /// Description.
    pub description: String,
    /// Total.
    pub total: String,
    /// Currency.
    pub currency: String,
    /// Category.
    pub spending_type: String,
    /// Date.
    pub spent_date: String,
    /// Payers.
    pub payers: Vec<AllocationRow>,
    /// Shares.
    pub shares: Vec<AllocationRow>,
    /// CSRF.
    pub csrf: String,
    /// Shared authenticated shell protection values.
    pub shell: AuthenticatedShell,
}

/// Named allocation row.
pub struct AllocationRow {
    /// Participant name.
    pub participant: String,
    /// Amount.
    pub amount: String,
}

/// Confirmation for deleting a spending or group.
#[derive(Template)]
#[template(path = "confirm.html")]
pub struct ConfirmTemplate {
    /// Heading.
    pub heading: String,
    /// Message.
    pub message: String,
    /// POST action.
    pub action: String,
    /// Cancel link.
    pub cancel: String,
    /// CSRF.
    pub csrf: String,
    /// Shared authenticated shell protection values.
    pub shell: AuthenticatedShell,
}
