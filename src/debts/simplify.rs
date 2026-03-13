use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebtTransfer {
    pub from_member_id: String,
    pub to_member_id: String,
    pub amount: Decimal,
}

pub fn simplify_balances(balances: &[(String, Decimal)]) -> Vec<DebtTransfer> {
    let mut debtors: Vec<(String, Decimal)> = balances
        .iter()
        .filter(|(_, balance)| *balance < Decimal::ZERO)
        .map(|(id, balance)| (id.clone(), balance.abs()))
        .collect();
    let mut creditors: Vec<(String, Decimal)> = balances
        .iter()
        .filter(|(_, balance)| *balance > Decimal::ZERO)
        .map(|(id, balance)| (id.clone(), *balance))
        .collect();

    debtors.sort_by(|a, b| a.0.cmp(&b.0));
    creditors.sort_by(|a, b| a.0.cmp(&b.0));

    let mut transfers = Vec::new();
    let mut di = 0usize;
    let mut ci = 0usize;

    while di < debtors.len() && ci < creditors.len() {
        let (debtor_id, debtor_amount) = debtors[di].clone();
        let (creditor_id, creditor_amount) = creditors[ci].clone();

        if debtor_amount == Decimal::ZERO {
            di += 1;
            continue;
        }
        if creditor_amount == Decimal::ZERO {
            ci += 1;
            continue;
        }

        let payment = if debtor_amount <= creditor_amount {
            debtor_amount
        } else {
            creditor_amount
        };

        transfers.push(DebtTransfer {
            from_member_id: debtor_id.clone(),
            to_member_id: creditor_id.clone(),
            amount: payment,
        });

        debtors[di].1 -= payment;
        creditors[ci].1 -= payment;
    }

    transfers
}

pub fn minimal_transfers(balances: &[(String, Decimal)]) -> Vec<DebtTransfer> {
    let mut entries: Vec<(String, Decimal)> = balances
        .iter()
        .filter(|(_, balance)| *balance != Decimal::ZERO)
        .cloned()
        .collect();

    if entries.is_empty() {
        return Vec::new();
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let n = entries.len();
    let size = 1usize << n;
    let mut sum = vec![Decimal::ZERO; size];

    for mask in 1..size {
        let bit = mask & (!mask + 1);
        let idx = bit.trailing_zeros() as usize;
        let prev = mask ^ bit;
        sum[mask] = sum[prev] + entries[idx].1;
    }

    let mut dp = vec![i32::MIN; size];
    let mut choice = vec![0usize; size];
    let _ = best_partition(size - 1, &sum, &mut dp, &mut choice);

    let mut transfers = Vec::new();
    build_transfers(&entries, &sum, &choice, size - 1, &mut transfers);

    transfers.sort_by(|a, b| {
        let from_cmp = a.from_member_id.cmp(&b.from_member_id);
        if from_cmp == std::cmp::Ordering::Equal {
            a.to_member_id.cmp(&b.to_member_id)
        } else {
            from_cmp
        }
    });

    transfers
}

fn best_partition(mask: usize, sum: &[Decimal], memo: &mut [i32], choice: &mut [usize]) -> i32 {
    if mask == 0 {
        return 0;
    }
    if memo[mask] != i32::MIN {
        return memo[mask];
    }
    if sum[mask] != Decimal::ZERO {
        memo[mask] = 0;
        choice[mask] = 0;
        return 0;
    }

    let first = mask & (!mask + 1);
    let mut sub = mask;
    let mut best = 1;
    let mut best_sub = mask;

    while sub > 0 {
        if (sub & first) != 0 && sum[sub] == Decimal::ZERO {
            let remaining = mask ^ sub;
            let score = 1 + best_partition(remaining, sum, memo, choice);
            if score > best {
                best = score;
                best_sub = sub;
            }
        }
        sub = (sub - 1) & mask;
    }

    memo[mask] = best;
    choice[mask] = best_sub;
    best
}

fn build_transfers(
    entries: &[(String, Decimal)],
    sum: &[Decimal],
    choice: &[usize],
    mask: usize,
    out: &mut Vec<DebtTransfer>,
) {
    if mask == 0 {
        return;
    }

    let chosen = choice[mask];
    if chosen == 0 || chosen == mask || sum[mask] == Decimal::ZERO {
        let subset = mask;
        let mut subset_entries = Vec::new();
        for idx in 0..entries.len() {
            if (subset & (1usize << idx)) != 0 {
                subset_entries.push(entries[idx].clone());
            }
        }
        out.extend(simplify_balances(&subset_entries));
        return;
    }

    build_transfers(entries, sum, choice, chosen, out);
    build_transfers(entries, sum, choice, mask ^ chosen, out);
}
