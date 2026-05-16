use chrono::NaiveDateTime;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct MemberRow {
    pub id: String,
    pub group_id: String,
    pub display_name: String,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub removed_at: Option<NaiveDateTime>,
}

#[derive(Clone)]
pub struct MemberRepo {
    pool: SqlitePool,
}

impl MemberRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        id: String,
        group_id: String,
        display_name: String,
        now: NaiveDateTime,
    ) -> Result<MemberRow, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO members (id, group_id, display_name, is_active, created_at, updated_at, removed_at)
             VALUES (?, ?, ?, 1, ?, ?, NULL)",
            id,
            group_id,
            display_name,
            now,
            now
        )
        .execute(&self.pool)
        .await?;

        Ok(MemberRow {
            id,
            group_id,
            display_name,
            is_active: true,
            created_at: now,
            updated_at: now,
            removed_at: None,
        })
    }

    pub async fn find(&self, member_id: &str) -> Result<Option<MemberRow>, sqlx::Error> {
        sqlx::query_as!(
            MemberRow,
            r#"SELECT id, group_id, display_name, is_active as "is_active: bool",
               created_at as "created_at: NaiveDateTime",
               updated_at as "updated_at: NaiveDateTime",
               removed_at as "removed_at: Option<NaiveDateTime>"
               FROM members WHERE id = ?"#,
            member_id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list(
        &self,
        group_id: &str,
        include_inactive: bool,
    ) -> Result<Vec<MemberRow>, sqlx::Error> {
        if include_inactive {
            sqlx::query_as!(
                MemberRow,
                r#"SELECT id, group_id, display_name, is_active as "is_active: bool",
                   created_at as "created_at: NaiveDateTime",
                   updated_at as "updated_at: NaiveDateTime",
                   removed_at as "removed_at: Option<NaiveDateTime>"
                   FROM members WHERE group_id = ?"#,
                group_id
            )
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as!(
                MemberRow,
                r#"SELECT id, group_id, display_name, is_active as "is_active: bool",
                   created_at as "created_at: NaiveDateTime",
                   updated_at as "updated_at: NaiveDateTime",
                   removed_at as "removed_at: Option<NaiveDateTime>"
                   FROM members WHERE group_id = ? AND is_active = 1"#,
                group_id
            )
            .fetch_all(&self.pool)
            .await
        }
    }

    pub async fn update_name(
        &self,
        member_id: &str,
        display_name: String,
        now: NaiveDateTime,
    ) -> Result<Option<MemberRow>, sqlx::Error> {
        let Some(_existing) = self.find(member_id).await? else {
            return Ok(None);
        };

        sqlx::query!(
            "UPDATE members SET display_name = ?, updated_at = ? WHERE id = ?",
            display_name,
            now,
            member_id
        )
        .execute(&self.pool)
        .await?;

        self.find(member_id).await
    }

    pub async fn soft_delete(
        &self,
        member_id: &str,
        now: NaiveDateTime,
    ) -> Result<Option<MemberRow>, sqlx::Error> {
        let Some(_existing) = self.find(member_id).await? else {
            return Ok(None);
        };

        sqlx::query!(
            "UPDATE members SET is_active = 0, removed_at = ?, updated_at = ? WHERE id = ?",
            now,
            now,
            member_id
        )
        .execute(&self.pool)
        .await?;

        self.find(member_id).await
    }
}
