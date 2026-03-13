use chrono::NaiveDateTime;

use crate::app::state::AppState;
use crate::groups::group_service::{GroupError, GroupService, GroupUpdate};
use crate::groups::member_service::{MemberError, MemberService};
use crate::web::error::AppError;

#[derive(Debug, Clone)]
pub struct CreateGroupRequest {
    pub name: String,
    pub target_currency: String,
}

#[derive(Debug, Clone)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub target_currency: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateMemberRequest {
    pub display_name: String,
}

#[derive(Debug, Clone)]
pub struct RenameMemberRequest {
    pub display_name: String,
}

pub async fn handle_create_group(
    state: &AppState,
    request: CreateGroupRequest,
    now: NaiveDateTime,
) -> Result<crate::db::entities::groups::Model, AppError> {
    let service = GroupService::new(state.db.clone());
    service
        .create_group(&request.name, &request.target_currency, now)
        .await
        .map_err(map_group_error)
}

pub async fn handle_update_group(
    state: &AppState,
    group_id: &str,
    request: UpdateGroupRequest,
    now: NaiveDateTime,
) -> Result<crate::groups::group_service::GroupUpdateOutcome, AppError> {
    let service = GroupService::new(state.db.clone());
    service
        .update_group(
            group_id,
            GroupUpdate {
                name: request.name,
                target_currency: request.target_currency,
            },
            now,
        )
        .await
        .map_err(map_group_error)
}

pub async fn handle_delete_group(
    state: &AppState,
    group_id: &str,
) -> Result<bool, AppError> {
    let service = GroupService::new(state.db.clone());
    service.delete_group(group_id).await.map_err(map_group_error)
}

pub async fn handle_add_member(
    state: &AppState,
    group_id: &str,
    request: CreateMemberRequest,
    now: NaiveDateTime,
) -> Result<crate::db::entities::members::Model, AppError> {
    let service = MemberService::new(state.db.clone());
    service
        .add_member(group_id, &request.display_name, now)
        .await
        .map_err(map_member_error)
}

pub async fn handle_rename_member(
    state: &AppState,
    group_id: &str,
    member_id: &str,
    request: RenameMemberRequest,
    now: NaiveDateTime,
) -> Result<crate::db::entities::members::Model, AppError> {
    let service = MemberService::new(state.db.clone());
    service
        .rename_member(group_id, member_id, &request.display_name, now)
        .await
        .map_err(map_member_error)
}

pub async fn handle_remove_member(
    state: &AppState,
    group_id: &str,
    member_id: &str,
    now: NaiveDateTime,
) -> Result<crate::db::entities::members::Model, AppError> {
    let service = MemberService::new(state.db.clone());
    service
        .remove_member(group_id, member_id, now)
        .await
        .map_err(map_member_error)
}

pub async fn handle_list_members(
    state: &AppState,
    group_id: &str,
    include_inactive: bool,
) -> Result<Vec<crate::db::entities::members::Model>, AppError> {
    let service = MemberService::new(state.db.clone());
    service
        .list_members(group_id, include_inactive)
        .await
        .map_err(map_member_error)
}

fn map_group_error(error: GroupError) -> AppError {
    match error {
        GroupError::Validation(msg) => AppError::Validation(msg),
        GroupError::NotFound => AppError::NotFound,
        GroupError::Database(err) => AppError::Database(err),
    }
}

fn map_member_error(error: MemberError) -> AppError {
    match error {
        MemberError::Validation(msg) => AppError::Validation(msg),
        MemberError::NotFound | MemberError::GroupNotFound => AppError::NotFound,
        MemberError::Database(err) => AppError::Database(err),
    }
}
