//! Integration tests for `SQLite` ledger persistence rules.

#![allow(clippy::expect_used)]

use chrono::NaiveDate;
use debtor_application::{
    ApplicationError, GroupDeleteInput, GroupReader, GroupRepository, ParticipantReader,
    ParticipantRepository, SpendingCursor, SpendingPageDirection, SpendingReader,
    SpendingRepository, StorageReason,
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
        .create_group_participant(
            group.id,
            Name::new("Ari").expect("valid name"),
            Color::new("#112233").expect("valid color"),
        )
        .await
        .expect("create participant");
    (store, group.id, participant.id)
}

#[sqlx::test(migrations = "../migrations")]
async fn group_settings_update_is_transactional_and_preserves_group_state(pool: SqlitePool) {
    let (store, group_id, _) = active_group_and_participant(&pool).await;

    let updated = store
        .update_group(
            group_id,
            Name::new("Renamed trip").expect("valid name"),
            Currency::Eur,
        )
        .await
        .expect("update settings");

    assert_eq!(updated.name.as_str(), "Renamed trip");
    assert_eq!(updated.currency, Currency::Eur);
    assert!(!updated.is_archived);
    let loaded = store.group(group_id).await.expect("load updated group");
    assert_eq!(loaded.name.as_str(), "Renamed trip");
    assert_eq!(loaded.currency, Currency::Eur);
}

#[sqlx::test(migrations = "../migrations")]
async fn group_lifecycle_is_state_scoped_and_history_free_delete_is_atomic(pool: SqlitePool) {
    let (store, group_id, participant_id) = active_group_and_participant(&pool).await;

    store.archive_group(group_id).await.expect("archive group");
    assert!(
        store
            .group(group_id)
            .await
            .expect("archived group")
            .is_archived
    );
    store.restore_group(group_id).await.expect("restore group");
    assert!(
        !store
            .group(group_id)
            .await
            .expect("restored group")
            .is_archived
    );

    let mismatch = store
        .delete_empty_group(GroupDeleteInput {
            group_id,
            participant_ids: Vec::new(),
        })
        .await
        .expect_err("mismatched confirmation set");
    assert!(matches!(mismatch, ApplicationError::Conflict));
    assert!(store.group(group_id).await.is_ok());

    let (history_store, history_group_id, history_participant_id) =
        active_group_and_participant(&pool).await;
    history_store
        .add_member(history_group_id, history_participant_id)
        .await
        .expect("history member");
    history_store
        .create_spending(spending(history_group_id, history_participant_id))
        .await
        .expect("history spending");
    let history_delete = history_store
        .delete_empty_group(GroupDeleteInput {
            group_id: history_group_id,
            participant_ids: vec![history_participant_id],
        })
        .await
        .expect_err("referenced group delete");
    assert!(matches!(history_delete, ApplicationError::Conflict));
    assert!(history_store.group(history_group_id).await.is_ok());

    let race_runtime = SqliteLedgerRuntime::new(pool.clone());
    let race_store = race_runtime.store();
    let race_group = race_store
        .create_group(Name::new("Race").expect("race name"), Currency::Usd)
        .await
        .expect("race group");
    let first = race_runtime.store();
    let second = race_runtime.store();
    let race_group_id = race_group.id;
    let (first_result, second_result) = tokio::join!(
        first.archive_group(race_group_id),
        second.archive_group(race_group_id)
    );
    assert_eq!(
        [first_result.is_ok(), second_result.is_ok()]
            .into_iter()
            .filter(|ok| *ok)
            .count(),
        1
    );
    assert!(
        race_store
            .group(race_group_id)
            .await
            .expect("race group")
            .is_archived
    );

    store
        .delete_empty_group(GroupDeleteInput {
            group_id,
            participant_ids: vec![participant_id],
        })
        .await
        .expect("delete history-free group");
    assert!(matches!(
        store.group(group_id).await,
        Err(ApplicationError::NotFound)
    ));
    let participant_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM participants WHERE id = ?")
            .bind(participant_id)
            .fetch_one(&pool)
            .await
            .expect("participant count");
    assert_eq!(participant_count, 0);
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
    assert_eq!(back.older.expect("older cursor").id, 2);
    assert!(back.newer.is_none());

    let empty = store
        .spending_page(
            group_id,
            Some(SpendingCursor {
                direction: SpendingPageDirection::Older,
                spent_date: first.items[24].spent_date,
                id: 1,
            }),
        )
        .await
        .expect("empty page");
    assert!(empty.items.is_empty());
    assert!(empty.older.is_none());
    assert!(empty.newer.is_none());
}

#[sqlx::test(migrations = "../migrations")]
async fn spending_history_projection_resolves_current_identity_and_complete_shares(
    pool: SqlitePool,
) {
    let (store, group_id, participant_id) = active_group_and_participant(&pool).await;
    store
        .add_member(group_id, participant_id)
        .await
        .expect("add member");
    let target = store
        .create_spending(spending(group_id, participant_id))
        .await
        .expect("create spending");
    store
        .update_group_participant(
            group_id,
            participant_id,
            Name::new("Renamed Ari").expect("name"),
            Color::new("#112233").expect("color"),
        )
        .await
        .expect("rename participant");
    sqlx::query("UPDATE participants SET is_archived = 1 WHERE id = ?")
        .bind(participant_id)
        .execute(&pool)
        .await
        .expect("archive participant");

    let page = store
        .spending_history_page(group_id, None)
        .await
        .expect("history projection");
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].spending.id, target.id);
    assert_eq!(page.items[0].payer.name.as_str(), "Renamed Ari");
    assert!(page.items[0].payer.is_archived);
    assert_eq!(page.items[0].shares.len(), 1);
    assert_eq!(page.items[0].shares[0].1.amount, Decimal::new(1_000, 2));

    let detail = store
        .spending_detail(group_id, target.id)
        .await
        .expect("complete detail");
    assert_eq!(detail.group.id, group_id);
    assert_eq!(detail.payers[0].0.name.as_str(), "Renamed Ari");
    assert!(detail.payers[0].0.is_archived);
    assert_eq!(detail.shares[0].1.amount, Decimal::new(1_000, 2));
}

#[sqlx::test(migrations = "../migrations")]
async fn spending_detail_does_not_materialize_unrelated_history(pool: SqlitePool) {
    let (store, group_id, participant_id) = active_group_and_participant(&pool).await;
    store
        .add_member(group_id, participant_id)
        .await
        .expect("add member");
    let target = store
        .create_spending(spending(group_id, participant_id))
        .await
        .expect("target spending");
    sqlx::query("INSERT INTO spendings (group_id, description, total_amount, currency, spending_type, spent_date) VALUES (?, 'Unrelated', 'not-a-decimal', 'USD', 'food', '2026-01-01')")
        .bind(group_id)
        .execute(&pool)
        .await
        .expect("insert unrelated malformed history");

    assert_eq!(
        store
            .spending(group_id, target.id)
            .await
            .expect("direct detail")
            .id,
        target.id
    );
}

#[sqlx::test(migrations = "../migrations")]
async fn archived_group_rejects_member_add_without_changing_membership(pool: SqlitePool) {
    let (store, group_id, participant_id) = active_group_and_participant(&pool).await;
    store.archive_group(group_id).await.expect("archive group");

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
    assert_eq!(count, 1);
}

#[sqlx::test(migrations = "../migrations")]
async fn archived_group_rolls_back_create_and_join(pool: SqlitePool) {
    let (store, group_id, _) = active_group_and_participant(&pool).await;
    store.archive_group(group_id).await.expect("archive group");

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
    store.archive_group(group_id).await.expect("archive group");

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
    store.archive_group(group_id).await.expect("archive group");

    assert!(matches!(
        store.delete_spending(group_id, created.id).await,
        Err(ApplicationError::Conflict)
    ));
    assert!(store.spending(group_id, created.id).await.is_ok());
}

#[sqlx::test(migrations = "../migrations")]
async fn archived_mutation_races_consistently_return_conflict(pool: SqlitePool) {
    let (store, group_id, participant_id) = active_group_and_participant(&pool).await;
    store
        .add_member(group_id, participant_id)
        .await
        .expect("add member");
    let spending = store
        .create_spending(spending(group_id, participant_id))
        .await
        .expect("create spending");
    store.archive_group(group_id).await.expect("archive group");

    let mut updated = spending;
    updated.description = Description::new("Updated dinner").expect("description");
    for result in [
        store
            .update_group(
                group_id,
                Name::new("Renamed trip").expect("name"),
                Currency::Eur,
            )
            .await
            .map(|_| ()),
        store
            .set_member_active(group_id, participant_id, false)
            .await,
        store.update_spending(updated).await.map(|_| ()),
    ] {
        assert!(matches!(result, Err(ApplicationError::Conflict)));
    }
}

#[sqlx::test(migrations = "../migrations")]
async fn archived_participant_rejects_direct_update(pool: SqlitePool) {
    let (store, group_id, participant_id) = active_group_and_participant(&pool).await;
    store
        .set_participant_archived(participant_id, true)
        .await
        .expect("archive participant");

    assert!(matches!(
        store
            .update_group_participant(
                group_id,
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
async fn participant_update_is_group_scoped_and_preserves_identity(pool: SqlitePool) {
    let (store, group_id, participant_id) = active_group_and_participant(&pool).await;
    let other_group = store
        .create_group(Name::new("Other").expect("valid name"), Currency::Usd)
        .await
        .expect("create other group");

    let updated = store
        .update_group_participant(
            group_id,
            participant_id,
            Name::new("  Updated  ").expect("valid name"),
            Color::new("#aabbcc").expect("valid color"),
        )
        .await
        .expect("update participant");
    assert_eq!(updated.id, participant_id);
    assert_eq!(updated.name.as_str(), "Updated");
    assert_eq!(updated.color.as_str(), "#AABBCC");

    assert!(matches!(
        store
            .update_group_participant(
                other_group.id,
                participant_id,
                Name::new("No disclosure").expect("valid name"),
                Color::new("#ddeeff").expect("valid color"),
            )
            .await,
        Err(ApplicationError::NotFound)
    ));
    let persisted = store
        .participant(participant_id)
        .await
        .expect("participant");
    assert_eq!(persisted.id, participant_id);
    assert_eq!(persisted.name.as_str(), "Updated");
    assert_eq!(persisted.color.as_str(), "#AABBCC");
}

#[sqlx::test(migrations = "../migrations")]
async fn participant_update_rechecks_group_and_membership_lifecycle(pool: SqlitePool) {
    let (store, group_id, participant_id) = active_group_and_participant(&pool).await;
    store.archive_group(group_id).await.expect("archive group");
    assert!(matches!(
        store
            .update_group_participant(
                group_id,
                participant_id,
                Name::new("Archived").expect("valid name"),
                Color::new("#445566").expect("valid color"),
            )
            .await,
        Err(ApplicationError::NotFound | ApplicationError::Conflict)
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

    let (store, group_id, participant_id) = active_group_and_participant(&pool).await;
    store
        .set_member_active(group_id, participant_id, false)
        .await
        .expect("deactivate membership");
    assert!(matches!(
        store
            .update_group_participant(
                group_id,
                participant_id,
                Name::new("Inactive").expect("valid name"),
                Color::new("#778899").expect("valid color"),
            )
            .await,
        Err(ApplicationError::NotFound)
    ));
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

#[sqlx::test(migrations = "../migrations")]
async fn corrupted_persisted_non_monetary_values_are_rejected(pool: SqlitePool) {
    let (store, group_id, participant_id) = active_group_and_participant(&pool).await;
    store
        .add_member(group_id, participant_id)
        .await
        .expect("add member");
    let created = store
        .create_spending(spending(group_id, participant_id))
        .await
        .expect("create spending");

    let mut connection = pool.acquire().await.expect("test connection");
    sqlx::query("PRAGMA ignore_check_constraints = ON")
        .execute(&mut *connection)
        .await
        .expect("allow deliberate corruption");

    sqlx::query("UPDATE groups SET is_archived = 2 WHERE id = ?")
        .bind(group_id)
        .execute(&mut *connection)
        .await
        .expect("corrupt group boolean");
    assert!(matches!(
        debtor_application::GroupReader::group(&store, group_id).await,
        Err(ApplicationError::Storage(StorageReason::InvalidData))
    ));
    sqlx::query("UPDATE groups SET is_archived = 0 WHERE id = ?")
        .bind(group_id)
        .execute(&mut *connection)
        .await
        .expect("restore group boolean");

    sqlx::query("UPDATE participants SET color = 'invalid' WHERE id = ?")
        .bind(participant_id)
        .execute(&mut *connection)
        .await
        .expect("corrupt participant color");
    assert!(matches!(
        store.participant(participant_id).await,
        Err(ApplicationError::Storage(StorageReason::InvalidData))
    ));
    sqlx::query("UPDATE participants SET color = '#112233' WHERE id = ?")
        .bind(participant_id)
        .execute(&mut *connection)
        .await
        .expect("restore participant color");

    sqlx::query("UPDATE group_members SET is_active = 2 WHERE group_id = ? AND participant_id = ?")
        .bind(group_id)
        .bind(participant_id)
        .execute(&mut *connection)
        .await
        .expect("corrupt membership boolean");
    assert!(matches!(
        store.group_members(group_id).await,
        Err(ApplicationError::Storage(StorageReason::InvalidData))
    ));
    sqlx::query("UPDATE group_members SET is_active = 1 WHERE group_id = ? AND participant_id = ?")
        .bind(group_id)
        .bind(participant_id)
        .execute(&mut *connection)
        .await
        .expect("restore membership boolean");

    for (query, value, restore_query, original) in [
        (
            "UPDATE spendings SET description = ? WHERE id = ?",
            "",
            "UPDATE spendings SET description = ? WHERE id = ?",
            "Dinner",
        ),
        (
            "UPDATE spendings SET currency = ? WHERE id = ?",
            "XXX",
            "UPDATE spendings SET currency = ? WHERE id = ?",
            "USD",
        ),
        (
            "UPDATE spendings SET spending_type = ? WHERE id = ?",
            "invalid",
            "UPDATE spendings SET spending_type = ? WHERE id = ?",
            "food",
        ),
        (
            "UPDATE spendings SET spent_date = ? WHERE id = ?",
            "not-a-date",
            "UPDATE spendings SET spent_date = ? WHERE id = ?",
            "2026-01-01",
        ),
    ] {
        sqlx::query(query)
            .bind(value)
            .bind(created.id)
            .execute(&mut *connection)
            .await
            .expect("corrupt spending field");
        assert!(matches!(
            store.spending(group_id, created.id).await,
            Err(ApplicationError::Storage(StorageReason::InvalidData))
        ));
        sqlx::query(restore_query)
            .bind(original)
            .bind(created.id)
            .execute(&mut *connection)
            .await
            .expect("restore spending field");
    }
}
