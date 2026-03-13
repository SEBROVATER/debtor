use chrono::NaiveDateTime;
use thiserror::Error;
use uuid::Uuid;

use crate::db::entities::groups;
use crate::groups::group_repo::GroupRepo;
use sea_orm::DbErr;

#[derive(Debug, Error)]
pub enum GroupError {
    #[error("group not found")]
    NotFound,
    #[error("validation error: {0}")]
    Validation(String),
    #[error(transparent)]
    Database(#[from] DbErr),
}

#[derive(Debug, Clone)]
pub struct GroupUpdate {
    pub name: Option<String>,
    pub target_currency: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GroupUpdateOutcome {
    pub group: groups::Model,
    pub currency_changed: bool,
}

#[derive(Clone)]
pub struct GroupService {
    repo: GroupRepo,
}

impl GroupService {
    pub fn new(conn: sea_orm::DatabaseConnection) -> Self {
        Self {
            repo: GroupRepo::new(conn),
        }
    }

    pub async fn create_group(
        &self,
        name: &str,
        target_currency: &str,
        now: NaiveDateTime,
    ) -> Result<groups::Model, GroupError> {
        let name = normalize_name(name)?;
        let target_currency = normalize_currency(target_currency)?;
        let id = Uuid::new_v4().to_string();

        let group = self
            .repo
            .create(id, name, target_currency, now)
            .await?;
        Ok(group)
    }

    pub async fn list_groups(&self) -> Result<Vec<groups::Model>, GroupError> {
        Ok(self.repo.list().await?)
    }

    pub async fn update_group(
        &self,
        group_id: &str,
        update: GroupUpdate,
        now: NaiveDateTime,
    ) -> Result<GroupUpdateOutcome, GroupError> {
        let existing = self.repo.find(group_id).await?;
        let Some(existing) = existing else {
            return Err(GroupError::NotFound);
        };

        let mut currency_changed = false;
        let name = if let Some(name) = update.name {
            Some(normalize_name(&name)?)
        } else {
            None
        };

        let target_currency = if let Some(currency) = update.target_currency {
            let normalized = normalize_currency(&currency)?;
            if normalized != existing.target_currency {
                currency_changed = true;
            }
            Some(normalized)
        } else {
            None
        };

        let updated = self
            .repo
            .update(group_id, name, target_currency, now)
            .await?;

        let Some(updated) = updated else {
            return Err(GroupError::NotFound);
        };

        Ok(GroupUpdateOutcome {
            group: updated,
            currency_changed,
        })
    }

    pub async fn delete_group(&self, group_id: &str) -> Result<bool, GroupError> {
        Ok(self.repo.delete(group_id).await?)
    }
}

fn normalize_name(input: &str) -> Result<String, GroupError> {
    let trimmed = input.trim();
    if trimmed.is_empty() || trimmed.len() > 80 {
        return Err(GroupError::Validation("name must be 1..80 chars".to_string()));
    }
    Ok(trimmed.to_string())
}

fn normalize_currency(input: &str) -> Result<String, GroupError> {
    let trimmed = input.trim().to_ascii_uppercase();
    if trimmed.len() != 3 || !trimmed.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(GroupError::Validation(
            "currency must be ISO-4217 code".to_string(),
        ));
    }
    Ok(trimmed)
}
