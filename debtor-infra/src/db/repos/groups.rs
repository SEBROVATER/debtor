use async_trait::async_trait;
use debtor_application::{ApplicationError, GroupReader, GroupRepository};
use debtor_domain::currency::Currency;
use debtor_domain::model::{EntityId, Group, Name};

use super::decoding::{DbGroup, group};
use super::{SqliteLedgerStore, changed, storage};

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
        let id = sqlx::query!(
            "INSERT INTO groups (name, currency) VALUES (?, ?)",
            name.as_str(),
            currency.code()
        )
        .execute(&self.pool)
        .await
        .map_err(storage)?
        .last_insert_rowid();
        self.group(id).await
    }

    async fn update_group(
        &self,
        id: EntityId,
        name: Name,
        currency: Currency,
    ) -> Result<Group, ApplicationError> {
        let _write_guard = self.write_guard().await?;
        changed(sqlx::query!("UPDATE groups SET name = ?, currency = ?, updated_at = datetime('now') WHERE id = ? AND is_archived = 0", name.as_str(), currency.code(), id)
            .execute(&self.pool).await.map_err(storage)?)?;
        self.group(id).await
    }

    async fn set_group_archived(
        &self,
        id: EntityId,
        archived: bool,
    ) -> Result<(), ApplicationError> {
        let _write_guard = self.write_guard().await?;
        changed(
            sqlx::query!(
                "UPDATE groups SET is_archived = ?, updated_at = datetime('now') WHERE id = ?",
                i64::from(archived),
                id
            )
            .execute(&self.pool)
            .await
            .map_err(storage)?,
        )
    }

    async fn delete_empty_group(&self, id: EntityId) -> Result<(), ApplicationError> {
        let _write_guard = self.write_guard().await?;
        let result = sqlx::query!("DELETE FROM groups WHERE id = ? AND is_archived = 0 AND NOT EXISTS (SELECT 1 FROM spendings WHERE group_id = ?)", id, id)
            .execute(&self.pool).await.map_err(storage)?;
        if result.rows_affected() == 0 {
            if self.group(id).await.is_err() {
                return Err(ApplicationError::NotFound);
            }
            return Err(ApplicationError::Conflict);
        }
        Ok(())
    }
}
