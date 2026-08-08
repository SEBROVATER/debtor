//! Integration tests for `SQLite` ledger persistence rules.

#![allow(clippy::expect_used)]

use chrono::NaiveDate;
use debtor_application::{
    ApplicationError, GroupRepository, ParticipantRepository, SpendingCursor,
    SpendingPageDirection, SpendingReader, SpendingRepository, StorageReason,
};
use debtor_domain::currency::Currency;
use debtor_domain::model::{Allocation, Color, Description, Name, Spending, SpendingType};
use debtor_infra::db::repos::{SqliteLedgerRuntime, SqliteLedgerStore};
use rust_decimal::Decimal;
use sqlx::SqlitePool;

async fn active_group_and_participant(pool: &SqlitePool) -> (SqliteLedgerStore, i64, i64) {
    let store = SqliteLedgerRuntime::new(pool.clone()).store();
    let group = store
        .create_group(Name::new("Trip").expect("valid name"), Currency::Usd)
        .await
        .expect("create group");
    let participant = store
        .create_participant(
            Name::new("Ari").expect("valid name"),
            Color::new("#112233").expect("valid color"),
        )
        .await
        .expect("create participant");
    (store, group.id, participant.id)
}

fn spending(group_id: i64, participant_id: i64) -> Spending {
    Spending {
        id: 0,
        group_id,
        description: Description::new("Dinner").expect("valid description"),
        total: Decimal::new(1_000, 2),
        currency: Currency::Usd,
        spending_type: SpendingType::Food,
        spent_date: NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date"),
        payers: vec![Allocation {
            participant_id,
            amount: Decimal::new(1_000, 2),
        }],
        shares: vec![Allocation {
            participant_id,
            amount: Decimal::new(1_000, 2),
        }],
    }
}

#[sqlx::test(migrations = "../migrations")]
async fn spending_history_is_bounded_and_keyset_stable(pool: SqlitePool) {
    let (store, group_id, participant_id) = active_group_and_participant(&pool).await;
    store
        .add_member(group_id, participant_id)
        .await
        .expect("add member");
    for index in 0..26 {
        let mut row = spending(group_id, participant_id);
        row.description = Description::new(format!("Dinner {index}")).expect("description");
        store.create_spending(row).await.expect("create spending");
    }

    let first = store
        .spending_page(group_id, None)
        .await
        .expect("first page");
    assert_eq!(first.items.len(), 25);
    assert!(first.older.is_some());
    assert!(first.newer.is_none());
    assert_eq!(first.items[0].id, 26);
    assert_eq!(first.items[24].id, 2);

    let second = store
        .spending_page(group_id, first.older)
        .await
        .expect("second page");
    assert_eq!(second.items.len(), 1);
    assert_eq!(second.items[0].id, 1);
    assert!(second.older.is_none());
    assert!(second.newer.is_some());

    let back = store
        .spending_page(
            group_id,
            Some(SpendingCursor {
                direction: SpendingPageDirection::Newer,
                spent_date: second.items[0].spent_date,
                id: second.items[0].id,
            }),
        )
        .await
        .expect("newer page");
    assert_eq!(back.items.len(), 25);
    assert_eq!(back.items[0].id, 26);
    assert_eq!(back.items[24].id, 2);
}

#[sqlx::test(migrations = "../migrations")]
async fn archived_group_rejects_member_add_without_creating_membership(pool: SqlitePool) {
    let (store, group_id, participant_id) = active_group_and_participant(&pool).await;
    store
        .set_group_archived(group_id, true)
        .await
        .expect("archive group");

    assert!(matches!(
        store.add_member(group_id, participant_id).await,
        Err(ApplicationError::Conflict)
    ));
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM group_members WHERE group_id = ? AND participant_id = ?",
    )
    .bind(group_id)
    .bind(participant_id)
    .fetch_one(&pool)
    .await
    .expect("count memberships");
    assert_eq!(count, 0);
}

#[sqlx::test(migrations = "../migrations")]
async fn archived_group_rolls_back_create_and_join(pool: SqlitePool) {
    let (store, group_id, _) = active_group_and_participant(&pool).await;
    store
        .set_group_archived(group_id, true)
        .await
        .expect("archive group");

    assert!(matches!(
        store
            .create_group_participant(
                group_id,
                Name::new("Bea").expect("valid name"),
                Color::new("#445566").expect("valid color"),
            )
            .await,
        Err(ApplicationError::Conflict)
    ));
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM participants WHERE name = 'Bea'")
        .fetch_one(&pool)
        .await
        .expect("count participants");
    assert_eq!(count, 0);
}

#[sqlx::test(migrations = "../migrations")]
async fn archived_group_rejects_spending_create_without_aggregate_rows(pool: SqlitePool) {
    let (store, group_id, participant_id) = active_group_and_participant(&pool).await;
    store
        .add_member(group_id, participant_id)
        .await
        .expect("add member");
    store
        .set_group_archived(group_id, true)
        .await
        .expect("archive group");

    assert!(matches!(
        store
            .create_spending(spending(group_id, participant_id))
            .await,
        Err(ApplicationError::Conflict)
    ));
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spendings WHERE group_id = ?")
        .bind(group_id)
        .fetch_one(&pool)
        .await
        .expect("count spendings");
    assert_eq!(count, 0);
}

#[sqlx::test(migrations = "../migrations")]
async fn archived_group_rejects_spending_delete_and_preserves_history(pool: SqlitePool) {
    let (store, group_id, participant_id) = active_group_and_participant(&pool).await;
    store
        .add_member(group_id, participant_id)
        .await
        .expect("add member");
    let created = store
        .create_spending(spending(group_id, participant_id))
        .await
        .expect("create spending");
    store
        .set_group_archived(group_id, true)
        .await
        .expect("archive group");

    assert!(matches!(
        store.delete_spending(group_id, created.id).await,
        Err(ApplicationError::Conflict)
    ));
    assert!(store.spending(group_id, created.id).await.is_ok());
}

#[sqlx::test(migrations = "../migrations")]
async fn archived_participant_rejects_direct_update(pool: SqlitePool) {
    let (store, _, participant_id) = active_group_and_participant(&pool).await;
    store
        .set_participant_archived(participant_id, true)
        .await
        .expect("archive participant");

    assert!(matches!(
        store
            .update_participant(
                participant_id,
                Name::new("Renamed").expect("valid name"),
                Color::new("#445566").expect("valid color"),
            )
            .await,
        Err(ApplicationError::Conflict)
    ));
    assert_eq!(
        store
            .participant(participant_id)
            .await
            .expect("load participant")
            .name
            .as_str(),
        "Ari"
    );
}

#[sqlx::test(migrations = "../migrations")]
async fn corrupted_persisted_money_is_rejected(pool: SqlitePool) {
    let (store, group_id, participant_id) = active_group_and_participant(&pool).await;
    store
        .add_member(group_id, participant_id)
        .await
        .expect("add member");
    let created = store
        .create_spending(spending(group_id, participant_id))
        .await
        .expect("create spending");

    sqlx::query("UPDATE spendings SET total_amount = '10.00' WHERE id = ?")
        .bind(created.id)
        .execute(&pool)
        .await
        .expect("corrupt spending amount");

    assert!(matches!(
        store.spending(group_id, created.id).await,
        Err(ApplicationError::Storage(StorageReason::InvalidData))
    ));
}
