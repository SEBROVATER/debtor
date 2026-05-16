use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ExpenseShareSummary {
    pub payer_member_id: String,
    pub shares: Vec<MemberShare>,
}

#[derive(Debug, Clone)]
pub struct MemberShare {
    pub member_id: String,
    pub amount: Decimal,
}

pub fn compute_balances(expenses: &[ExpenseShareSummary]) -> HashMap<String, Decimal> {
    let mut balances: HashMap<String, Decimal> = HashMap::new();

    for expense in expenses {
        let total: Decimal = expense.shares.iter().map(|s| s.amount).sum();
        let payer_entry = balances.entry(expense.payer_member_id.clone()).or_default();
        *payer_entry += total;

        for share in &expense.shares {
            let entry = balances.entry(share.member_id.clone()).or_default();
            *entry -= share.amount;
        }
    }

    balances
}
