use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AdminUsers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AdminUsers::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AdminUsers::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(AdminUsers::PasswordHash).string().not_null())
                    .col(ColumnDef::new(AdminUsers::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(AdminUsers::UpdatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AuthState::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuthState::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AuthState::FailedAttemptCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(AuthState::LockoutUntil).date_time())
                    .col(ColumnDef::new(AuthState::LastFailedAt).date_time())
                    .col(ColumnDef::new(AuthState::UpdatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Sessions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Sessions::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Sessions::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(Sessions::TokenHash)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Sessions::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Sessions::LastSeenAt).date_time().not_null())
                    .col(ColumnDef::new(Sessions::ExpiresAt).date_time().not_null())
                    .col(ColumnDef::new(Sessions::RevokedAt).date_time())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sessions_user")
                            .from(Sessions::Table, Sessions::UserId)
                            .to(AdminUsers::Table, AdminUsers::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Groups::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Groups::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Groups::Name).string().not_null())
                    .col(
                        ColumnDef::new(Groups::TargetCurrency)
                            .string_len(3)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Groups::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Groups::UpdatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Members::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Members::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Members::GroupId).string().not_null())
                    .col(ColumnDef::new(Members::DisplayName).string().not_null())
                    .col(
                        ColumnDef::new(Members::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(ColumnDef::new(Members::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Members::UpdatedAt).date_time().not_null())
                    .col(ColumnDef::new(Members::RemovedAt).date_time())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_members_group")
                            .from(Members::Table, Members::GroupId)
                            .to(Groups::Table, Groups::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Expenses::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Expenses::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Expenses::GroupId).string().not_null())
                    .col(ColumnDef::new(Expenses::PayerMemberId).string().not_null())
                    .col(
                        ColumnDef::new(Expenses::Amount)
                            .decimal_len(16, 6)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Expenses::Currency).string_len(3).not_null())
                    .col(ColumnDef::new(Expenses::Note).string())
                    .col(ColumnDef::new(Expenses::ExpenseDate).date().not_null())
                    .col(ColumnDef::new(Expenses::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Expenses::UpdatedAt).date_time().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_expenses_group")
                            .from(Expenses::Table, Expenses::GroupId)
                            .to(Groups::Table, Groups::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_expenses_payer")
                            .from(Expenses::Table, Expenses::PayerMemberId)
                            .to(Members::Table, Members::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ExpenseShares::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ExpenseShares::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ExpenseShares::ExpenseId).string().not_null())
                    .col(ColumnDef::new(ExpenseShares::MemberId).string().not_null())
                    .col(ColumnDef::new(ExpenseShares::ShareMode).string().not_null())
                    .col(
                        ColumnDef::new(ExpenseShares::ShareValue)
                            .decimal_len(16, 6)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExpenseShares::ComputedAmount)
                            .decimal_len(16, 6)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExpenseShares::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExpenseShares::UpdatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_expense_shares_expense")
                            .from(ExpenseShares::Table, ExpenseShares::ExpenseId)
                            .to(Expenses::Table, Expenses::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_expense_shares_member")
                            .from(ExpenseShares::Table, ExpenseShares::MemberId)
                            .to(Members::Table, Members::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ExchangeRates::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ExchangeRates::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ExchangeRates::FromCurrency)
                            .string_len(3)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExchangeRates::ToCurrency)
                            .string_len(3)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExchangeRates::Rate)
                            .decimal_len(16, 6)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExchangeRates::FetchedAt)
                            .date_time()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ExchangeRates::RateDate).date().not_null())
                    .col(ColumnDef::new(ExchangeRates::Provider).string().not_null())
                    .index(
                        Index::create()
                            .name("idx_exchange_rates_unique")
                            .table(ExchangeRates::Table)
                            .col(ExchangeRates::FromCurrency)
                            .col(ExchangeRates::ToCurrency)
                            .col(ExchangeRates::RateDate)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ExchangeRates::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ExpenseShares::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Expenses::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Members::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Groups::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Sessions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AuthState::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AdminUsers::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum AdminUsers {
    Table,
    Id,
    Username,
    PasswordHash,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum AuthState {
    Table,
    Id,
    FailedAttemptCount,
    LockoutUntil,
    LastFailedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Sessions {
    Table,
    Id,
    UserId,
    TokenHash,
    CreatedAt,
    LastSeenAt,
    ExpiresAt,
    RevokedAt,
}

#[derive(DeriveIden)]
enum Groups {
    Table,
    Id,
    Name,
    TargetCurrency,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Members {
    Table,
    Id,
    GroupId,
    DisplayName,
    IsActive,
    CreatedAt,
    UpdatedAt,
    RemovedAt,
}

#[derive(DeriveIden)]
enum Expenses {
    Table,
    Id,
    GroupId,
    PayerMemberId,
    Amount,
    Currency,
    Note,
    ExpenseDate,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ExpenseShares {
    Table,
    Id,
    ExpenseId,
    MemberId,
    ShareMode,
    ShareValue,
    ComputedAmount,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ExchangeRates {
    Table,
    Id,
    FromCurrency,
    ToCurrency,
    Rate,
    FetchedAt,
    RateDate,
    Provider,
}
