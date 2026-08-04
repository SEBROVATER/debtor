//! Core entities and validated values.

use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use thiserror::Error;

use crate::currency::Currency;

/// Stable identifier used by persisted domain entities.
pub type EntityId = i64;

/// Domain validation failure.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// A text field is empty after trimming.
    #[error("{field} must not be empty")]
    Empty {
        /// Field that failed validation.
        field: &'static str,
    },
    /// A text field exceeds its product limit.
    #[error("{field} must be at most {limit} characters")]
    TooLong {
        /// Field that exceeded its limit.
        field: &'static str,
        /// Maximum allowed character count.
        limit: usize,
    },
    /// A color does not use RGB hexadecimal notation.
    #[error("color must use #RRGGBB notation")]
    InvalidColor,
    /// A date predates supported data.
    #[error("spending date must be on or after 2025-01-01")]
    DateTooEarly,
    /// An amount is not positive.
    #[error("{field} must be positive")]
    NonPositive {
        /// Field that must be positive.
        field: &'static str,
    },
    /// An amount exceeds currency precision.
    #[error("amount has too many decimal places for {currency}")]
    InvalidPrecision {
        /// Currency that defines allowed precision.
        currency: Currency,
    },
    /// An amount exceeds the product limit.
    #[error("amount exceeds the maximum supported value")]
    AmountTooLarge,
    /// An allocation list is empty.
    #[error("at least one {field} is required")]
    EmptyAllocations {
        /// Allocation type that is empty.
        field: &'static str,
    },
    /// An allocation repeats a participant.
    #[error("participant {participant_id} appears more than once")]
    DuplicateParticipant {
        /// Repeated participant identifier.
        participant_id: EntityId,
    },
    /// An equal split would create one or more zero-valued shares.
    #[error("equal split total must contain at least one minor unit per recipient")]
    InsufficientMinorUnits {
        /// Number of selected recipients.
        recipients: usize,
    },
    /// An allocation sum differs from the total.
    #[error("{field} allocations must equal the spending total")]
    AllocationTotalMismatch {
        /// Allocation type whose sum differs.
        field: &'static str,
    },
    /// A decimal aggregation exceeded the representable range.
    #[error("decimal aggregation overflowed")]
    ArithmeticOverflow,
}

/// A trimmed display name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name(String);

impl Name {
    /// Creates a valid name.
    ///
    /// # Errors
    ///
    /// Returns a validation error for an empty or overlong value.
    pub fn new(value: impl AsRef<str>) -> Result<Self, ValidationError> {
        let value = value.as_ref().trim();
        if value.is_empty() {
            return Err(ValidationError::Empty { field: "name" });
        }
        if value.chars().count() > 100 {
            return Err(ValidationError::TooLong {
                field: "name",
                limit: 100,
            });
        }
        Ok(Self(value.to_owned()))
    }

    /// Returns the normalized text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A normalized participant color.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Color(String);

impl Color {
    /// Creates a valid `#RRGGBB` color.
    ///
    /// # Errors
    ///
    /// Returns a validation error for malformed color input.
    pub fn new(value: impl AsRef<str>) -> Result<Self, ValidationError> {
        let value = value.as_ref().trim();
        if value.len() != 7
            || !value.starts_with('#')
            || !value.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
        {
            return Err(ValidationError::InvalidColor);
        }
        Ok(Self(value.to_ascii_uppercase()))
    }

    /// Returns the normalized CSS color.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A trimmed spending description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Description(String);

impl Description {
    /// Creates a valid description.
    ///
    /// # Errors
    ///
    /// Returns a validation error for an empty or overlong value.
    pub fn new(value: impl AsRef<str>) -> Result<Self, ValidationError> {
        let value = value.as_ref().trim();
        if value.is_empty() {
            return Err(ValidationError::Empty {
                field: "description",
            });
        }
        if value.chars().count() > 200 {
            return Err(ValidationError::TooLong {
                field: "description",
                limit: 200,
            });
        }
        Ok(Self(value.to_owned()))
    }

    /// Returns the normalized text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Fixed spending categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpendingType {
    /// Food and dining.
    Food,
    /// Transport.
    Transport,
    /// Housing.
    Housing,
    /// Fun and entertainment.
    Fun,
    /// Shopping.
    Shopping,
    /// Bills and utilities.
    Bills,
    /// Health.
    Health,
    /// Other spending.
    Other,
}

impl SpendingType {
    /// All categories.
    pub const ALL: [Self; 8] = [
        Self::Food,
        Self::Transport,
        Self::Housing,
        Self::Fun,
        Self::Shopping,
        Self::Bills,
        Self::Health,
        Self::Other,
    ];

    /// Stable database value.
    pub const fn code(self) -> &'static str {
        match self {
            Self::Food => "food",
            Self::Transport => "transport",
            Self::Housing => "housing",
            Self::Fun => "fun",
            Self::Shopping => "shopping",
            Self::Bills => "bills",
            Self::Health => "health",
            Self::Other => "other",
        }
    }
}

impl fmt::Display for SpendingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

/// Parsing failure for a spending type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseSpendingTypeError;

impl fmt::Display for ParseSpendingTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unknown spending type")
    }
}
impl std::error::Error for ParseSpendingTypeError {}

impl FromStr for SpendingType {
    type Err = ParseSpendingTypeError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "food" => Ok(Self::Food),
            "transport" => Ok(Self::Transport),
            "housing" => Ok(Self::Housing),
            "fun" => Ok(Self::Fun),
            "shopping" => Ok(Self::Shopping),
            "bills" => Ok(Self::Bills),
            "health" => Ok(Self::Health),
            "other" => Ok(Self::Other),
            _ => Err(ParseSpendingTypeError),
        }
    }
}

/// A persisted group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    /// Database identifier.
    pub id: EntityId,
    /// Display name.
    pub name: Name,
    /// Target display currency.
    pub currency: Currency,
    /// Whether the group is read-only.
    pub is_archived: bool,
}

/// A persisted participant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Participant {
    /// Database identifier.
    pub id: EntityId,
    /// Display name.
    pub name: Name,
    /// Accent color.
    pub color: Color,
    /// Whether unavailable for new allocations.
    pub is_archived: bool,
}

/// A group membership.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupMember {
    /// Group identifier.
    pub group_id: EntityId,
    /// Participant identifier.
    pub participant_id: EntityId,
    /// Whether eligible for new allocations.
    pub is_active: bool,
}

/// A positive payer or share allocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Allocation {
    /// Participant identifier.
    pub participant_id: EntityId,
    /// Positive source-currency amount.
    pub amount: Decimal,
}

/// A complete spending aggregate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spending {
    /// Database identifier.
    pub id: EntityId,
    /// Owning group.
    pub group_id: EntityId,
    /// Description.
    pub description: Description,
    /// Total source amount.
    pub total: Decimal,
    /// Source currency.
    pub currency: Currency,
    /// Category.
    pub spending_type: SpendingType,
    /// Spending date.
    pub spent_date: NaiveDate,
    /// Positive payer allocations.
    pub payers: Vec<Allocation>,
    /// Positive share allocations.
    pub shares: Vec<Allocation>,
}

impl Spending {
    /// Validates the aggregate before persistence.
    ///
    /// # Errors
    ///
    /// Returns the first invalid aggregate invariant.
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_amount(self.total, self.currency, "total")?;
        let earliest = NaiveDate::from_ymd_opt(2025, 1, 1).ok_or(ValidationError::DateTooEarly)?;
        if self.spent_date < earliest {
            return Err(ValidationError::DateTooEarly);
        }
        validate_allocations(&self.payers, self.total, self.currency, "payer")?;
        validate_allocations(&self.shares, self.total, self.currency, "share")
    }

    /// Returns paid-minus-owed participant nets in source currency.
    ///
    /// # Errors
    ///
    /// Returns an arithmetic overflow error when allocation aggregation exceeds
    /// the Decimal representation.
    pub fn source_nets(&self) -> Result<BTreeMap<EntityId, Decimal>, ValidationError> {
        let mut result: BTreeMap<EntityId, Decimal> = BTreeMap::new();
        for value in &self.payers {
            let net = result.entry(value.participant_id).or_default();
            *net = net
                .checked_add(value.amount)
                .ok_or(ValidationError::ArithmeticOverflow)?;
        }
        for value in &self.shares {
            let net = result.entry(value.participant_id).or_default();
            *net = net
                .checked_sub(value.amount)
                .ok_or(ValidationError::ArithmeticOverflow)?;
        }
        Ok(result)
    }
}

/// Validates a positive source-currency amount.
///
/// # Errors
///
/// Returns a validation error for non-positive, over-precise, or oversized amounts.
pub fn validate_amount(
    amount: Decimal,
    currency: Currency,
    field: &'static str,
) -> Result<(), ValidationError> {
    if amount <= Decimal::ZERO {
        return Err(ValidationError::NonPositive { field });
    }
    if !currency.accepts_precision(amount) {
        return Err(ValidationError::InvalidPrecision { currency });
    }
    if amount > Decimal::new(999_999_999_999, 0) {
        return Err(ValidationError::AmountTooLarge);
    }
    Ok(())
}

fn validate_allocations(
    values: &[Allocation],
    total: Decimal,
    currency: Currency,
    field: &'static str,
) -> Result<(), ValidationError> {
    if values.is_empty() {
        return Err(ValidationError::EmptyAllocations { field });
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut sum = Decimal::ZERO;
    for value in values {
        if !seen.insert(value.participant_id) {
            return Err(ValidationError::DuplicateParticipant {
                participant_id: value.participant_id,
            });
        }
        validate_amount(value.amount, currency, field)?;
        sum = sum
            .checked_add(value.amount)
            .ok_or(ValidationError::ArithmeticOverflow)?;
    }
    if sum != total {
        return Err(ValidationError::AllocationTotalMismatch { field });
    }
    Ok(())
}
