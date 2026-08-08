use async_trait::async_trait;
use debtor_application::{ApplicationError, GroupReader, ParticipantRepository};
use debtor_domain::model::{Color, EntityId, GroupMember, Name, Participant};

use super::decoding::{DbGroupMember, DbParticipant, decoded_bool, participant};
use super::{
    SqliteLedgerStore, changed, group_mutable, group_write_failure,
    group_write_failure_in_transaction, participant_mutable, participant_write_failure, storage,
};

async fn participant_by_id(
    pool: &sqlx::SqlitePool,
    id: EntityId,
) -> Result<Participant, ApplicationError> {
    sqlx::query_as!(
        DbParticipant,
        "SELECT id, name, color, is_archived FROM participants WHERE id = ?",
        id
    )
    .fetch_optional(pool)
    .await
    .map_err(storage)?
    .ok_or(ApplicationError::NotFound)
    .and_then(participant)
}

#[async_trait]
impl ParticipantRepository for SqliteLedgerStore {
    async fn list_participants(
        &self,
        archived: bool,
    ) -> Result<Vec<Participant>, ApplicationError> {
        let archived = i64::from(archived);
        sqlx::query_as!(DbParticipant, "SELECT id, name, color, is_archived FROM participants WHERE is_archived = ? ORDER BY name, id", archived)
            .fetch_all(&self.pool).await.map_err(storage)?.into_iter().map(participant).collect()
    }

    async fn create_participant(
        &self,
        name: Name,
        color: Color,
    ) -> Result<Participant, ApplicationError> {
        let _write_guard = self.write_guard().await?;
        let id = sqlx::query!(
            "INSERT INTO participants (name, color) VALUES (?, ?)",
            name.as_str(),
            color.as_str()
        )
        .execute(&self.pool)
        .await
        .map_err(storage)?
        .last_insert_rowid();
        participant_by_id(&self.pool, id).await
    }

    async fn participant(&self, id: EntityId) -> Result<Participant, ApplicationError> {
        participant_by_id(&self.pool, id).await
    }

    async fn create_group_participant(
        &self,
        group_id: EntityId,
        name: Name,
        color: Color,
    ) -> Result<Participant, ApplicationError> {
        let _write_guard = self.write_guard().await?;
        let mut tx = self.pool.begin().await.map_err(storage)?;
        let id = sqlx::query!(
            "INSERT INTO participants (name, color) VALUES (?, ?)",
            name.as_str(),
            color.as_str()
        )
        .execute(&mut *tx)
        .await
        .map_err(storage)?
        .last_insert_rowid();
        let membership = sqlx::query!("INSERT INTO group_members (group_id, participant_id) SELECT ?, ? WHERE EXISTS (SELECT 1 FROM groups WHERE id = ? AND is_archived = 0)", group_id, id, group_id)
            .execute(&mut *tx).await.map_err(storage)?;
        if membership.rows_affected() == 0 {
            return Err(group_write_failure_in_transaction(
                &mut tx,
                group_id,
                ApplicationError::Storage(debtor_application::StorageReason::Unexpected),
            )
            .await);
        }
        tx.commit().await.map_err(storage)?;
        participant_by_id(&self.pool, id).await
    }

    async fn update_participant(
        &self,
        id: EntityId,
        name: Name,
        color: Color,
    ) -> Result<Participant, ApplicationError> {
        let _write_guard = self.write_guard().await?;
        let result = sqlx::query!("UPDATE participants SET name = ?, color = ?, updated_at = datetime('now') WHERE id = ? AND is_archived = 0", name.as_str(), color.as_str(), id)
            .execute(&self.pool).await.map_err(storage)?;
        if result.rows_affected() == 0 {
            return Err(participant_write_failure(
                &self.pool,
                id,
                ApplicationError::Storage(debtor_application::StorageReason::Unexpected),
            )
            .await);
        }
        participant_by_id(&self.pool, id).await
    }

    async fn set_participant_archived(
        &self,
        id: EntityId,
        archived: bool,
    ) -> Result<(), ApplicationError> {
        let _write_guard = self.write_guard().await?;
        changed(sqlx::query!("UPDATE participants SET is_archived = ?, updated_at = datetime('now') WHERE id = ?", i64::from(archived), id)
            .execute(&self.pool).await.map_err(storage)?)
    }

    async fn group_members(
        &self,
        group_id: EntityId,
    ) -> Result<Vec<(Participant, GroupMember)>, ApplicationError> {
        self.group(group_id).await?;
        sqlx::query_as!(DbGroupMember, "SELECT p.id AS \"id!: i64\", p.name, p.color, p.is_archived, gm.is_active AS \"is_active!: i64\" FROM group_members gm JOIN participants p ON p.id = gm.participant_id WHERE gm.group_id = ? ORDER BY p.name, p.id", group_id)
            .fetch_all(&self.pool).await.map_err(storage)?.into_iter().map(|row| {
                let participant_id = row.id;
                Ok((participant(DbParticipant { id: participant_id, name: row.name, color: row.color, is_archived: row.is_archived })?, GroupMember { group_id, participant_id, is_active: decoded_bool(row.is_active)? }))
            }).collect()
    }

    async fn add_member(
        &self,
        group_id: EntityId,
        participant_id: EntityId,
    ) -> Result<(), ApplicationError> {
        let _write_guard = self.write_guard().await?;
        let result = sqlx::query!("INSERT INTO group_members (group_id, participant_id) SELECT ?, ? WHERE EXISTS (SELECT 1 FROM groups WHERE id = ? AND is_archived = 0) AND EXISTS (SELECT 1 FROM participants WHERE id = ? AND is_archived = 0) ON CONFLICT(group_id, participant_id) DO UPDATE SET is_active = 1", group_id, participant_id, group_id, participant_id)
            .execute(&self.pool).await.map_err(storage)?;
        if result.rows_affected() == 0 {
            group_mutable(&self.pool, group_id).await?;
            participant_mutable(&self.pool, participant_id).await?;
            return Err(ApplicationError::Storage(
                debtor_application::StorageReason::Unexpected,
            ));
        }
        Ok(())
    }

    async fn set_member_active(
        &self,
        group_id: EntityId,
        participant_id: EntityId,
        active: bool,
    ) -> Result<(), ApplicationError> {
        let _write_guard = self.write_guard().await?;
        let result = sqlx::query!("UPDATE group_members SET is_active = ? WHERE group_id = ? AND participant_id = ? AND EXISTS (SELECT 1 FROM groups WHERE id = ? AND is_archived = 0)", i64::from(active), group_id, participant_id, group_id)
            .execute(&self.pool).await.map_err(storage)?;
        if result.rows_affected() == 0 {
            return Err(
                group_write_failure(&self.pool, group_id, ApplicationError::NotFound).await,
            );
        }
        Ok(())
    }
}
