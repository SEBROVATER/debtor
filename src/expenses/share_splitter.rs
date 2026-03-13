use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShareMode {
    Equal,
    Percent,
    Amount,
}

#[derive(Debug, Clone)]
pub struct ShareInput {
    pub member_id: String,
    pub mode: ShareMode,
    pub value: Option<Decimal>,
}

impl ShareInput {
    pub fn equal(member_id: &str) -> Self {
        Self {
            member_id: member_id.to_string(),
            mode: ShareMode::Equal,
            value: None,
        }
    }

    pub fn percent(member_id: &str, value: Decimal) -> Self {
        Self {
            member_id: member_id.to_string(),
            mode: ShareMode::Percent,
            value: Some(value),
        }
    }

    pub fn amount(member_id: &str, value: Decimal) -> Self {
        Self {
            member_id: member_id.to_string(),
            mode: ShareMode::Amount,
            value: Some(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShareAllocation {
    pub member_id: String,
    pub mode: ShareMode,
    pub share_value: Decimal,
    pub computed_amount: Decimal,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ShareSplitError {
    #[error("no shares provided")]
    EmptyShares,
    #[error("invalid share value")]
    InvalidShareValue,
    #[error("percent total exceeds 100")]
    InvalidPercentTotal,
    #[error("amount shares exceed total")]
    AmountExceedsTotal,
    #[error("remaining amount cannot be split")]
    RemainingAmountMismatch,
}

pub fn normalize_shares(
    total_amount: Decimal,
    shares: Vec<ShareInput>,
) -> Result<Vec<ShareAllocation>, ShareSplitError> {
    if shares.is_empty() {
        return Err(ShareSplitError::EmptyShares);
    }

    let mut amount_total = Decimal::ZERO;
    let mut percent_total = Decimal::ZERO;
    let mut equal_count = 0usize;

    for share in &shares {
        match share.mode {
            ShareMode::Equal => equal_count += 1,
            ShareMode::Percent => {
                let value = share.value.ok_or(ShareSplitError::InvalidShareValue)?;
                if value <= Decimal::ZERO {
                    return Err(ShareSplitError::InvalidShareValue);
                }
                percent_total += value;
            }
            ShareMode::Amount => {
                let value = share.value.ok_or(ShareSplitError::InvalidShareValue)?;
                if value <= Decimal::ZERO {
                    return Err(ShareSplitError::InvalidShareValue);
                }
                amount_total += value;
            }
        }
    }

    if percent_total > Decimal::from(100) {
        return Err(ShareSplitError::InvalidPercentTotal);
    }

    if amount_total > total_amount {
        return Err(ShareSplitError::AmountExceedsTotal);
    }

    let percent_amount = (total_amount * percent_total) / Decimal::from(100);
    let remaining = total_amount - amount_total - percent_amount;
    if remaining < Decimal::ZERO {
        return Err(ShareSplitError::AmountExceedsTotal);
    }

    if equal_count == 0 && remaining != Decimal::ZERO {
        return Err(ShareSplitError::RemainingAmountMismatch);
    }

    let equal_share = if equal_count > 0 {
        remaining / Decimal::from(equal_count as i64)
    } else {
        Decimal::ZERO
    };

    let mut allocations = Vec::with_capacity(shares.len());
    for share in shares {
        let allocation = match share.mode {
            ShareMode::Equal => ShareAllocation {
                member_id: share.member_id,
                mode: ShareMode::Equal,
                share_value: Decimal::ZERO,
                computed_amount: equal_share,
            },
            ShareMode::Percent => {
                let value = share.value.expect("validated percent");
                let computed = (total_amount * value) / Decimal::from(100);
                ShareAllocation {
                    member_id: share.member_id,
                    mode: ShareMode::Percent,
                    share_value: value,
                    computed_amount: computed,
                }
            }
            ShareMode::Amount => {
                let value = share.value.expect("validated amount");
                ShareAllocation {
                    member_id: share.member_id,
                    mode: ShareMode::Amount,
                    share_value: value,
                    computed_amount: value,
                }
            }
        };
        allocations.push(allocation);
    }

    Ok(allocations)
}
