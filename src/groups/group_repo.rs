use chrono::NaiveDateTime;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct GroupRow {
    pub id: String,
    pub name: String,
    pub target_currency: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone)]
pub struct GroupRepo {
    pool: SqlitePool,
}

impl GroupRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        id: String,
        name: String,
        target_currency: String,
        now: NaiveDateTime,
    ) -> Result<GroupRow, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO groups (id, name, target_currency, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?)",
            id,
            name,
            target_currency,
            now,
            now
        )
        .execute(&self.pool)
        .await?;

        Ok(GroupRow {
            id,
            name,
            target_currency,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn list(&self) -> Result<Vec<GroupRow>, sqlx::Error> {
        sqlx::query_as!(GroupRow,
            r#"SELECT id, name, target_currency,
               created_at as "created_at: NaiveDateTime",
               updated_at as "updated_at: NaiveDateTime"
               FROM groups"#)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn find(&self, id: &str) -> Result<Option<GroupRow>, sqlx::Error> {
        sqlx::query_as!(
            GroupRow,
            r#"SELECT id, name, target_currency,
               created_at as "created_at: NaiveDateTime",
               updated_at as "updated_at: NaiveDateTime"
               FROM groups WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn update(
        &self,
        id: &str,
        name: Option<String>,
        target_currency: Option<String>,
        now: NaiveDateTime,
    ) -> Result<Option<GroupRow>, sqlx::Error> {
        let Some(existing) = self.find(id).await? else {
            return Ok(None);
        };

        let new_name = name.unwrap_or(existing.name);
        let new_currency = target_currency.unwrap_or(existing.target_currency);

        sqlx::query!(
            "UPDATE groups SET name = ?, target_currency = ?, updated_at = ? WHERE id = ?",
            new_name,
            new_currency,
            now,
            id
        )
        .execute(&self.pool)
        .await?;

        self.find(id).await
    }

    pub async fn delete(&self, id: &str) -> Result<bool, sqlx::Error> {
        Ok(
            sqlx::query!("DELETE FROM groups WHERE id = ?", id)
                .execute(&self.pool)
                .await?
                .rows_affected()
                > 0,
        )
    }
}
