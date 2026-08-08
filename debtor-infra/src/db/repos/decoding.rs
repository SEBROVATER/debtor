use std::str::FromStr;

use debtor_application::{ApplicationError, SpendingSummary, StorageReason};
use debtor_domain::currency::Currency;
use debtor_domain::model::{
    Allocation, Color, Description, EntityId, Group, Name, Participant, SpendingType,
    validate_amount,
};
use debtor_domain::money::parse_decimal;

pub(super) struct DbGroup {
    pub(super) id: i64,
    pub(super) name: String,
    pub(super) currency: String,
    pub(super) is_archived: i64,
}
pub(super) struct DbParticipant {
    pub(super) id: i64,
    pub(super) name: String,
    pub(super) color: String,
    pub(super) is_archived: i64,
}
pub(super) struct DbGroupMember {
    pub(super) id: i64,
    pub(super) name: String,
    pub(super) color: String,
    pub(super) is_archived: i64,
    pub(super) is_active: i64,
}
pub(super) struct DbSpending {
    pub(super) description: String,
    pub(super) total_amount: String,
    pub(super) currency: String,
    pub(super) spending_type: String,
    pub(super) spent_date: String,
}
pub(super) struct DbSpendingSummary {
    pub(super) id: i64,
    pub(super) description: String,
    pub(super) total_amount: String,
    pub(super) currency: String,
    pub(super) spending_type: String,
    pub(super) spent_date: String,
}
pub(super) struct DbSnapshotSpending {
    pub(super) id: i64,
    pub(super) description: String,
    pub(super) total_amount: String,
    pub(super) currency: String,
    pub(super) spending_type: String,
    pub(super) spent_date: String,
}
pub(super) struct DbAllocation {
    pub(super) participant_id: i64,
    pub(super) amount: String,
}
pub(super) struct DbSpendingAllocation {
    pub(super) spending_id: i64,
    pub(super) participant_id: i64,
    pub(super) amount: String,
}

pub(super) fn group(row: DbGroup) -> Result<Group, ApplicationError> {
    Ok(Group {
        id: row.id,
        name: Name::new(row.name).map_err(|_| invalid())?,
        currency: Currency::from_str(&row.currency).map_err(|_| invalid())?,
        is_archived: decoded_bool(row.is_archived)?,
    })
}

pub(super) fn participant(row: DbParticipant) -> Result<Participant, ApplicationError> {
    Ok(Participant {
        id: row.id,
        name: Name::new(row.name).map_err(|_| invalid())?,
        color: Color::new(row.color).map_err(|_| invalid())?,
        is_archived: decoded_bool(row.is_archived)?,
    })
}

pub(super) fn decoded_bool(value: i64) -> Result<bool, ApplicationError> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(invalid()),
    }
}

pub(super) fn canonical_decimal(value: &str) -> Result<rust_decimal::Decimal, ApplicationError> {
    parse_decimal(value).map_err(|_| invalid())
}

pub(super) fn spending_summary(
    group_id: EntityId,
    row: DbSpendingSummary,
) -> Result<SpendingSummary, ApplicationError> {
    let currency = Currency::from_str(&row.currency).map_err(|_| invalid())?;
    let total = canonical_decimal(&row.total_amount)?;
    validate_amount(total, currency, "total").map_err(|_| invalid())?;
    SpendingType::from_str(&row.spending_type).map_err(|_| invalid())?;
    let spent_date =
        chrono::NaiveDate::parse_from_str(&row.spent_date, "%Y-%m-%d").map_err(|_| invalid())?;
    let earliest = chrono::NaiveDate::from_ymd_opt(2025, 1, 1).ok_or_else(invalid)?;
    if spent_date < earliest {
        return Err(invalid());
    }
    Ok(SpendingSummary {
        id: row.id,
        group_id,
        description: Description::new(row.description).map_err(|_| invalid())?,
        total,
        currency,
        spent_date,
    })
}

pub(super) fn allocation(row: DbAllocation) -> Result<Allocation, ApplicationError> {
    Ok(Allocation {
        participant_id: row.participant_id,
        amount: canonical_decimal(&row.amount)?,
    })
}

pub(super) fn invalid() -> ApplicationError {
    ApplicationError::Storage(StorageReason::InvalidData)
}
