use chrono::NaiveDateTime;
use thiserror::Error;
use uuid::Uuid;

use crate::db::entities::members;
use crate::groups::group_repo::GroupRepo;
use crate::groups::member_repo::MemberRepo;
use sea_orm::DbErr;

#[derive(Debug, Error)]
pub enum MemberError {
    #[error("member not found")]
    NotFound,
    #[error("group not found")]
    GroupNotFound,
    #[error("validation error: {0}")]
    Validation(String),
    #[error(transparent)]
    Database(#[from] DbErr),
}

#[derive(Clone)]
pub struct MemberService {
    member_repo: MemberRepo,
    group_repo: GroupRepo,
}

impl MemberService {
    pub fn new(conn: sea_orm::DatabaseConnection) -> Self {
        Self {
            member_repo: MemberRepo::new(conn.clone()),
            group_repo: GroupRepo::new(conn),
        }
    }

    pub async fn add_member(
        &self,
        group_id: &str,
        display_name: &str,
        now: NaiveDateTime,
    ) -> Result<members::Model, MemberError> {
        self.ensure_group_exists(group_id).await?;
        let display_name = normalize_name(display_name)?;
        self.ensure_unique_active_name(group_id, &display_name)
            .await?;

        let id = Uuid::new_v4().to_string();
        let member = self
            .member_repo
            .create(id, group_id.to_string(), display_name, now)
            .await?;
        Ok(member)
    }

    pub async fn rename_member(
        &self,
        group_id: &str,
        member_id: &str,
        display_name: &str,
        now: NaiveDateTime,
    ) -> Result<members::Model, MemberError> {
        self.ensure_group_exists(group_id).await?;
        let display_name = normalize_name(display_name)?;
        self.ensure_unique_active_name(group_id, &display_name)
            .await?;

        let existing = self.member_repo.find(member_id).await?;
        let Some(existing) = existing else {
            return Err(MemberError::NotFound);
        };
        if existing.group_id != group_id {
            return Err(MemberError::NotFound);
        }

        let updated = self
            .member_repo
            .update_name(member_id, display_name, now)
            .await?;
        updated.ok_or(MemberError::NotFound)
    }

    pub async fn remove_member(
        &self,
        group_id: &str,
        member_id: &str,
        now: NaiveDateTime,
    ) -> Result<members::Model, MemberError> {
        self.ensure_group_exists(group_id).await?;
        let existing = self.member_repo.find(member_id).await?;
        let Some(existing) = existing else {
            return Err(MemberError::NotFound);
        };
        if existing.group_id != group_id {
            return Err(MemberError::NotFound);
        }

        let updated = self.member_repo.soft_delete(member_id, now).await?;
        updated.ok_or(MemberError::NotFound)
    }

    pub async fn list_members(
        &self,
        group_id: &str,
        include_inactive: bool,
    ) -> Result<Vec<members::Model>, MemberError> {
        self.ensure_group_exists(group_id).await?;
        Ok(self.member_repo.list(group_id, include_inactive).await?)
    }

    async fn ensure_group_exists(&self, group_id: &str) -> Result<(), MemberError> {
        let exists = self.group_repo.find(group_id).await?;
        if exists.is_none() {
            return Err(MemberError::GroupNotFound);
        }
        Ok(())
    }

    async fn ensure_unique_active_name(
        &self,
        group_id: &str,
        display_name: &str,
    ) -> Result<(), MemberError> {
        let active = self.member_repo.list(group_id, false).await?;
        let lower = display_name.to_ascii_lowercase();
        if active
            .iter()
            .any(|member| member.display_name.to_ascii_lowercase() == lower)
        {
            return Err(MemberError::Validation(
                "member name must be unique".to_string(),
            ));
        }
        Ok(())
    }
}

fn normalize_name(input: &str) -> Result<String, MemberError> {
    let trimmed = input.trim();
    if trimmed.is_empty() || trimmed.len() > 80 {
        return Err(MemberError::Validation(
            "display name must be 1..80 chars".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}
