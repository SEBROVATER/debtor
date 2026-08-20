use async_trait::async_trait;
use debtor_application::{ApplicationError, GroupDeleteInput, GroupReader, GroupRepository};
use debtor_domain::currency::Currency;
use debtor_domain::model::{EntityId, Group, Name};

use super::decoding::{DbGroup, group};
use super::{SqliteLedgerStore, group_write_failure_in_transaction, storage};

#[async_trait]
impl GroupReader for SqliteLedgerStore {
    async fn list_groups(&self, archived: bool) -> Result<Vec<Group>, ApplicationError> {
        let archived = i64::from(archived);
        sqlx::query_as!(DbGroup, "SELECT id, name, currency, is_archived FROM groups WHERE is_archived = ? ORDER BY name, id", archived)
            .fetch_all(&self.pool).await.map_err(storage)?.into_iter().map(group).collect()
    }

    async fn group(&self, id: EntityId) -> Result<Group, ApplicationError> {
        sqlx::query_as!(
            DbGroup,
            "SELECT id, name, currency, is_archived FROM groups WHERE id = ?",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(storage)?
        .ok_or(ApplicationError::NotFound)
        .and_then(group)
    }
}

#[async_trait]
impl GroupRepository for SqliteLedgerStore {
    async fn create_group(
        &self,
        name: Name,
        currency: Currency,
    ) -> Result<Group, ApplicationError> {
        let _write_guard = self.write_guard().await?;
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        let id = sqlx::query!(
            "INSERT INTO groups (name, currency) VALUES (?, ?)",
            name.as_str(),
            currency.code()
        )
        .execute(&mut *transaction)
        .await
        .map_err(storage)?
        .last_insert_rowid();
        let created = sqlx::query_as!(
            DbGroup,
            "SELECT id, name, currency, is_archived FROM groups WHERE id = ?",
            id
        )
        .fetch_one(&mut *transaction)
        .await
        .map_err(storage)
        .and_then(group)?;
        transaction.commit().await.map_err(storage)?;
        self.committed();
        Ok(created)
    }

    async fn update_group(
        &self,
        id: EntityId,
        name: Name,
        currency: Currency,
    ) -> Result<Group, ApplicationError> {
        let _write_guard = self.write_guard().await?;
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        let result = sqlx::query!("UPDATE groups SET name = ?, currency = ?, updated_at = datetime('now') WHERE id = ? AND is_archived = 0", name.as_str(), currency.code(), id)
            .execute(&mut *transaction).await.map_err(storage)?;
        if result.rows_affected() == 0 {
            return Err(group_write_failure_in_transaction(
                &mut transaction,
                id,
                ApplicationError::NotFound,
            )
            .await);
        }
        let updated = sqlx::query_as!(
            DbGroup,
            "SELECT id, name, currency, is_archived FROM groups WHERE id = ?",
            id
        )
        .fetch_one(&mut *transaction)
        .await
        .map_err(storage)
        .and_then(group)?;
        transaction.commit().await.map_err(storage)?;
        self.committed();
        Ok(updated)
    }

    async fn archive_group(&self, id: EntityId) -> Result<(), ApplicationError> {
        let _write_guard = self.write_guard().await?;
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        let result = sqlx::query!(
            "UPDATE groups SET is_archived = 1, updated_at = datetime('now') WHERE id = ? AND is_archived = 0",
            id
        )
        .execute(&mut *transaction)
        .await
        .map_err(storage)?;
        if result.rows_affected() == 0 {
            return Err(group_write_failure_in_transaction(
                &mut transaction,
                id,
                ApplicationError::Conflict,
            )
            .await);
        }
        transaction.commit().await.map_err(storage)?;
        self.committed();
        Ok(())
    }

    async fn restore_group(&self, id: EntityId) -> Result<(), ApplicationError> {
        let _write_guard = self.write_guard().await?;
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        let result = sqlx::query!(
            "UPDATE groups SET is_archived = 0, updated_at = datetime('now') WHERE id = ? AND is_archived = 1",
            id
        )
        .execute(&mut *transaction)
        .await
        .map_err(storage)?;
        if result.rows_affected() == 0 {
            return Err(group_write_failure_in_transaction(
                &mut transaction,
                id,
                ApplicationError::Conflict,
            )
            .await);
        }
        transaction.commit().await.map_err(storage)?;
        self.committed();
        Ok(())
    }

    async fn delete_empty_group(&self, input: GroupDeleteInput) -> Result<(), ApplicationError> {
        let _write_guard = self.write_guard().await?;
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        let group = sqlx::query_as!(
            DbGroup,
            "SELECT id, name, currency, is_archived FROM groups WHERE id = ? AND is_archived = 0 AND NOT EXISTS (SELECT 1 FROM spendings WHERE group_id = ?)",
            input.group_id,
            input.group_id
        )
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?;
        if group.is_none() {
            return Err(group_write_failure_in_transaction(
                &mut transaction,
                input.group_id,
                ApplicationError::Conflict,
            )
            .await);
        }
        let mut participant_ids = sqlx::query_scalar!(
            "SELECT id FROM participants WHERE group_id = ? ORDER BY id",
            input.group_id
        )
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage)?;
        participant_ids.sort_unstable();
        let mut expected_ids = input.participant_ids;
        expected_ids.sort_unstable();
        if participant_ids != expected_ids {
            return Err(ApplicationError::Conflict);
        }
        let result = sqlx::query!(
            "DELETE FROM groups WHERE id = ? AND is_archived = 0 AND NOT EXISTS (SELECT 1 FROM spendings WHERE group_id = ?)",
            input.group_id,
            input.group_id
        )
        .execute(&mut *transaction)
        .await
        .map_err(storage)?;
        if result.rows_affected() == 0 {
            return Err(group_write_failure_in_transaction(
                &mut transaction,
                input.group_id,
                ApplicationError::Conflict,
            )
            .await);
        }
        transaction.commit().await.map_err(storage)?;
        self.committed();
        Ok(())
    }
}
