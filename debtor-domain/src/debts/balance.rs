//! Balance calculation across spendings.

use std::collections::BTreeMap;

use rust_decimal::Decimal;

use crate::model::{EntityId, Spending};

/// Adds source-currency nets from a spending after multiplying by its rate.
pub fn add_converted_spending(
    balances: &mut BTreeMap<EntityId, Decimal>,
    spending: &Spending,
    rate: Decimal,
) {
    for (participant_id, net) in spending.source_nets() {
        *balances.entry(participant_id).or_default() += net * rate;
    }
}
