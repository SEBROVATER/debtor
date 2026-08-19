//! Askama template types.

use askama::Template;

/// Shared protection values for an authenticated page shell.
#[derive(Clone)]
pub struct AuthenticatedShell {
    /// Current synchronizer token.
    pub csrf: String,
    /// Single-use token shared by mutually exclusive unsafe forms on this page.
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
    /// Allow-listed native recovery destination.
    pub recovery_path: &'a str,
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
    /// Inline validation error.
    pub error: Option<String>,
    /// Lifecycle completion announcement.
    pub notice: Option<String>,
    /// Group row to focus after restore.
    pub focus_group: Option<i64>,
}

/// Renderable group row.
pub struct GroupRow {
    /// ID.
    pub id: i64,
    /// Name.
    pub name: String,
    /// Currency.
    pub currency: String,
    /// Number of active, non-archived participants in the Group.
    pub active_participants: usize,
    /// Whether this row receives the post-restore focus.
    pub focused: bool,
}

/// Current-month Source Currency summary projection.
pub struct SourceSummaryView {
    /// Current UTC month label.
    pub month: String,
    /// Explicit UTC context label.
    pub context: String,
    /// Source Currency blocks.
    pub currencies: Vec<SourceCurrencyRow>,
    /// Whether the month contains no Spendings.
    pub empty: bool,
    /// Whether the source calculation was unavailable.
    pub unavailable: bool,
    /// Scoped status announcement.
    pub status: String,
}

/// One Source Currency block in the Summary projection.
pub struct SourceCurrencyRow {
    /// ISO Source Currency code.
    pub currency: String,
    /// Currency symbol.
    pub symbol: String,
    /// Exact formatted Group total.
    pub total: String,
    /// Per-Payer totals.
    pub payers: Vec<SourcePayerRow>,
}

/// One current Participant Payer row in a Source Currency block.
pub struct SourcePayerRow {
    /// Participant identifier, retained for stable markup.
    pub id: i64,
    /// Current Participant name.
    pub name: String,
    /// Stored marker color.
    pub color: String,
    /// Whether the identity is archived.
    pub archived: bool,
    /// Exact formatted paid total.
    pub total: String,
}

/// Current-month Group Currency conversion projection.
#[derive(Clone, Copy)]
pub enum ConvertedSummaryState {
    /// Values are complete and use only non-future contexts.
    Ready,
    /// Values use at least one eligible stale quote and no future context.
    Stale,
    /// Values include at least one future-dated provisional context.
    Provisional,
    /// Values use stale evidence for a future Spending.
    ProvisionalStale,
    /// Values are being refreshed by an enhanced request.
    Updating,
    /// No converted values are available.
    Unavailable,
}

impl ConvertedSummaryState {
    /// Whether the owning result is currently busy.
    pub fn is_updating(self) -> bool {
        matches!(self, Self::Updating)
    }

    /// Whether the result has no usable converted values.
    pub fn is_unavailable(self) -> bool {
        matches!(self, Self::Unavailable)
    }

    /// Whether the values are provisional.
    pub fn is_provisional(self) -> bool {
        matches!(self, Self::Provisional | Self::ProvisionalStale)
    }

    /// Whether the values use stale rate evidence.
    pub fn is_stale(self) -> bool {
        matches!(self, Self::Stale | Self::ProvisionalStale)
    }
}

/// Display-ready converted Summary state.
pub struct ConvertedSummaryView {
    /// Group Currency code.
    pub currency: String,
    /// Group Currency symbol.
    pub symbol: String,
    /// Whether the month has no converted Payer rows.
    pub empty: bool,
    /// Current conversion state.
    pub state: ConvertedSummaryState,
    /// Exact formatted Group Currency total.
    pub total: String,
    /// Converted Payer rows.
    pub payers: Vec<ConvertedPayerRow>,
    /// Deterministic rate evidence.
    pub rates: Vec<ConvertedRateRow>,
    /// Scoped status announcement.
    pub status: String,
}

/// HTMX/native-refreshable converted Summary fragment.
#[derive(Template)]
#[template(path = "converted_summary.html")]
pub struct ConvertedSummaryTemplate {
    /// Group identifier used by the refresh target.
    pub group_id: i64,
    /// Display-ready converted state.
    pub converted_summary: ConvertedSummaryView,
}

/// One converted Payer row.
pub struct ConvertedPayerRow {
    /// Participant identifier.
    pub id: i64,
    /// Current Participant name.
    pub name: String,
    /// Stored marker color.
    pub color: String,
    /// Whether the identity is archived.
    pub archived: bool,
    /// Formatted converted total.
    pub total: String,
}

/// One disclosed rate context.
pub struct ConvertedRateRow {
    /// Source Currency code.
    pub base: String,
    /// Group Currency code.
    pub quote: String,
    /// Original requested date.
    pub requested_date: String,
    /// Effective fetch date.
    pub fetch_date: String,
    /// Provider effective date.
    pub effective_date: String,
    /// Exact rate.
    pub rate: String,
    /// Whether stale evidence was used.
    pub stale: bool,
    /// Whether the context is provisional.
    pub provisional: bool,
}

/// Debt view page.
#[derive(Template)]
#[template(path = "debts.html")]
pub struct DebtsTemplate {
    /// Group identifier.
    pub group_id: i64,
    /// Whether mutations are blocked for this Group.
    pub archived: bool,
    /// Currency.
    pub currency: String,
    /// Whether the snapshot contained any Spending.
    pub has_spendings: bool,
    /// Complete Participant balance rows.
    pub balances: Vec<BalanceRow>,
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

/// Renderable Participant balance.
pub struct BalanceRow {
    /// Current Participant name.
    pub participant: String,
    /// Whether the identity is archived.
    pub archived: bool,
    /// Participant marker color.
    pub color: String,
    /// Exact display amount including symbol and ISO code.
    pub amount: String,
    /// Explicit human-readable balance direction.
    pub direction: String,
}

/// Renderable exchange-rate disclosure row.
pub struct RateRow {
    /// Base currency.
    pub base: String,
    /// Target currency.
    pub quote: String,
    /// Requested date.
    pub requested_date: String,
    /// Cache/provider fetch date used for the requested context.
    pub fetch_date: String,
    /// Provider effective date.
    pub effective_date: String,
    /// Exact rate.
    pub rate: String,
    /// Stale marker.
    pub stale: bool,
    /// Provisional marker.
    pub provisional: bool,
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use askama::Template;

    use super::{AuthenticatedShell, BalanceRow, DebtsTemplate, RateRow, TransferRow};

    #[test]
    fn debts_template_discloses_current_mode_and_selection() {
        let template = DebtsTemplate {
            group_id: 1,
            archived: false,
            currency: "USD".to_owned(),
            has_spendings: true,
            balances: vec![BalanceRow {
                participant: "Ada".to_owned(),
                archived: false,
                color: "#123456".to_owned(),
                amount: "$1.00 USD".to_owned(),
                direction: "is owed".to_owned(),
            }],
            transfers: vec![TransferRow {
                from: "Bob".to_owned(),
                to: "Ada".to_owned(),
                amount: "$1.00 USD".to_owned(),
            }],
            mode: "current".to_owned(),
            warning: None,
            calculated_at: "2026-08-19T06:00:00+00:00".to_owned(),
            rates: vec![RateRow {
                base: "EUR".to_owned(),
                quote: "USD".to_owned(),
                requested_date: "2026-08-19".to_owned(),
                fetch_date: "2026-08-19".to_owned(),
                effective_date: "2026-08-19".to_owned(),
                rate: "1.10".to_owned(),
                stale: false,
                provisional: false,
            }],
            shell: AuthenticatedShell {
                csrf: "csrf".to_owned(),
                submission_token: "submission".to_owned(),
            },
        };

        let rendered = template.render().expect("current debts template");

        assert!(rendered.contains("Current calculation"));
        assert!(rendered.contains("Current rates are selected for this result."));
        assert!(rendered.contains("value=\"current\" aria-controls=\"debts-results\" checked"));
        assert!(rendered.contains("hx-push-url=\"true\""));
        assert!(rendered.contains("hx-trigger=\"change\""));
        assert!(rendered.contains("aria-busy=\"false\""));
        assert!(rendered.contains("class=\"debt-updating-placeholder\" role=\"status\""));
        assert!(!rendered.contains("hx-on::"));
        assert!(!rendered.contains(" hx-on"));
        assert!(rendered.contains("<h2 id=\"debts-results-heading\" tabindex=\"-1\" autofocus>"));
        assert!(
            !rendered
                .contains("id=\"debts-heading\" class=\"group-heading\" tabindex=\"-1\" autofocus")
        );
        assert!(!rendered.contains("Historical calculation</h2>"));

        let css = include_str!("../../static/css/app.css");
        assert!(
            css.contains(
                ".debt-results.htmx-request .debt-updating-placeholder { display: block; }"
            )
        );
        assert!(
            css.contains(".debt-results.htmx-request .debt-financial-content { display: none; }")
        );
    }
}

/*
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

*/
/// Group spending page.
#[derive(Template)]
#[template(path = "group.html")]
pub struct GroupTemplate {
    /// Group name.
    pub name: String,
    /// Group identifier.
    pub group_id: i64,
    /// Current contextual shell destination.
    pub section: String,
    /// Target currency.
    pub currency: String,
    /// Group settings name draft.
    pub settings_name: String,
    /// Group settings currency draft.
    pub settings_currency: String,
    /// Supported Group Currency options.
    pub settings_currencies: Vec<SelectOption>,
    /// Group settings validation error.
    pub settings_error: Option<String>,
    /// Committed settings announcement.
    pub settings_notice: Option<String>,
    /// Submitted settings field that failed validation.
    pub settings_invalid_field: Option<String>,
    /// CSRF token.
    pub csrf: String,
    /// Shared authenticated shell protection values.
    pub shell: AuthenticatedShell,
    /// Active member rows.
    pub members: Vec<MemberRow>,
    /// Inactive memberships available for reactivation.
    pub inactive_members: Vec<MemberRow>,
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
    /// Whether the loaded history proves this Group is empty.
    pub can_delete: bool,
    /// Inline error.
    pub error: Option<String>,
    /// Participant field that failed validation.
    pub participant_invalid_field: Option<String>,
    /// Participant row to focus after a committed add.
    pub focus_participant: Option<i64>,
    /// Participant row to announce after a committed edit.
    pub participant_notice: Option<i64>,
    /// Participant name draft.
    pub create_name: String,
    /// Participant color draft.
    pub create_color: String,
    /// Expense form state.
    pub expense: ExpenseFormView,
    /// Current-month Source Currency financial result.
    pub source_summary: SourceSummaryView,
    /// Current-month Group Currency financial result.
    pub converted_summary: ConvertedSummaryView,
}

/// Focused full-page Spending create/preview form.
#[derive(Template)]
#[template(path = "spending_form.html")]
pub struct SpendingFormTemplate {
    /// Group name.
    pub group_name: String,
    /// Group identifier.
    pub group_id: i64,
    /// Shared authenticated shell protection values.
    pub shell: AuthenticatedShell,
    /// Form projection.
    pub expense: ExpenseFormView,
    /// Form action for create preview/approval or existing Spending update.
    pub action: String,
    /// Allow-listed native return destination for Cancel.
    pub cancel_path: String,
    /// Whether the current page is a reviewed non-editable preview.
    pub reviewed: bool,
    /// Whether this form edits an existing Spending.
    pub editing: bool,
    /// Existing Spending identifier when editing.
    pub spending_id: i64,
    /// Form-level error or status text.
    pub status: Option<String>,
    /// Whether the heading should receive forward focus.
    pub focus_heading: bool,
}

/// Renderable Group-scoped Participant edit row for enhanced requests.
#[derive(Template)]
#[template(path = "participant_edit_row.html")]
pub struct ParticipantEditRowTemplate {
    /// Owning Group identifier.
    pub group_id: i64,
    /// CSRF token.
    pub csrf: String,
    /// Shared authenticated submission token.
    pub submission_token: String,
    /// Participant edit row projection.
    pub member: MemberRow,
    /// Whether the row itself should receive focus.
    pub focus_row: bool,
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
    /// Whether the selected Payer is eligible for this edit role.
    pub payer_allowed: bool,
    /// Member payer rows.
    pub payer_rows: Vec<MemberRow>,
    /// Equal recipients.
    pub share_rows: Vec<MemberRow>,
    /// Exact allocation difference or closure state.
    pub allocation_status: String,
    /// Error message.
    pub error: Option<String>,
    /// Dynamic submitted fields whose participant row is not available in the current projection.
    pub unmapped_fields: Vec<(String, String)>,
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
#[allow(clippy::struct_excessive_bools)]
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
    /// Whether this identity may be selected as Payer in this form.
    pub payer_allowed: bool,
    /// Whether this identity may be selected as a Share Participant in this form.
    pub share_allowed: bool,
    /// Selected in the current form.
    pub selected: bool,
    /// Exact allocation validation message for this row.
    pub allocation_error: Option<String>,
    /// Draft amount.
    pub amount: String,
    /// Derived exact Share amount for a Proportional preview.
    pub derived_amount: String,
    /// Whether this row currently renders its edit form.
    pub editing: bool,
    /// Edit name draft.
    pub edit_name: String,
    /// Edit color draft.
    pub edit_color: String,
    /// Edit validation error.
    pub edit_error: Option<String>,
    /// Edit field with invalid guidance.
    pub edit_invalid_field: Option<String>,
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
    /// Group name.
    pub group_name: String,
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
    /// Currency symbol.
    pub currency_symbol: String,
    /// Currency.
    pub currency: String,
    /// Category.
    pub spending_type: String,
    /// Date.
    pub spent_date: String,
    /// Payers.
    pub payers: Vec<TransactionAllocationRow>,
    /// Shares.
    pub shares: Vec<TransactionAllocationRow>,
    /// CSRF.
    pub csrf: String,
    /// Shared authenticated shell protection values.
    pub shell: AuthenticatedShell,
}

/// Transactions history page.
#[derive(Template)]
#[template(path = "transactions.html")]
#[allow(clippy::struct_excessive_bools)]
pub struct TransactionsTemplate {
    /// Group name.
    pub group_name: String,
    /// Group identifier.
    pub group_id: i64,
    /// Group Currency.
    pub currency: String,
    /// Active section.
    pub section: String,
    /// Whether this Group is archived.
    pub archived: bool,
    /// Shared authenticated shell protection values.
    pub shell: AuthenticatedShell,
    /// Bounded transaction rows.
    pub spendings: Vec<TransactionRow>,
    /// Cursor link for older rows.
    pub older_spendings: Option<String>,
    /// Cursor link for newer rows.
    pub newer_spendings: Option<String>,
    /// Whether the requested page should offer a newest link.
    pub show_newest_spendings: bool,
    /// Whether the history is empty.
    pub empty: bool,
    /// Page context announcement.
    pub page_status: String,
    /// Whether the Transactions heading receives forward focus.
    pub focus_heading: bool,
}

/// Renderable transaction row and expanded detail.
pub struct TransactionRow {
    /// Spending identifier.
    pub id: i64,
    /// Description.
    pub description: String,
    /// Exact source total.
    pub total: String,
    /// Currency symbol.
    pub currency_symbol: String,
    /// Source Currency.
    pub currency: String,
    /// ISO spending date.
    pub spent_date: String,
    /// Category.
    pub spending_type: String,
    /// Current Payer.
    pub payer: TransactionParticipant,
    /// Historical Payer amount.
    pub payer_amount: String,
    /// Current Share identities and exact amounts.
    pub shares: Vec<TransactionAllocationRow>,
    /// Whether this row should be expanded/focused.
    pub focused: bool,
    /// Whether this row's Delete control receives return focus.
    pub delete_focused: bool,
    /// Canonical Delete confirmation route for this row.
    pub delete_path: String,
}

/// Current Participant identity shown in history.
pub struct TransactionParticipant {
    /// Participant identifier.
    pub id: i64,
    /// Current name.
    pub name: String,
    /// Stored marker color.
    pub color: String,
    /// Whether the identity is archived.
    pub archived: bool,
}

/// Historical allocation with current Participant identity.
pub struct TransactionAllocationRow {
    /// Current Participant identity.
    pub participant: TransactionParticipant,
    /// Exact historical amount.
    pub amount: String,
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
    /// Optional disclosed Participant names for Group deletion.
    pub details: Vec<String>,
    /// Whether the confirmation is irreversible.
    pub destructive: bool,
    /// Complete Spending facts shown on a Spending delete confirmation.
    pub facts: Vec<ConfirmFact>,
    /// Stable focus target for the confirmation page.
    pub focus_id: String,
}

/// One labeled fact on a destructive confirmation page.
pub struct ConfirmFact {
    /// Fact label.
    pub label: String,
    /// Already validated display value.
    pub value: String,
}
