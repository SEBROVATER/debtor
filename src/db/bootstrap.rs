use chrono::Utc;
use debtor_migration::Migrator;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use sea_orm_migration::MigratorTrait;

use crate::app::config::AppConfig;
use crate::db::connection::connect_sqlite;
use crate::db::entities::admin_users;

pub async fn initialize_database(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let conn = connect_sqlite(database_url).await?;
    Migrator::up(&conn, None).await?;
    Ok(conn)
}

pub async fn bootstrap_admin_user(
    conn: &DatabaseConnection,
    config: &AppConfig,
) -> Result<bool, DbErr> {
    let existing = admin_users::Entity::find()
        .filter(admin_users::Column::Username.eq(config.admin_username.clone()))
        .one(conn)
        .await?;

    if existing.is_some() {
        return Ok(false);
    }

    let Some(password_hash) = config.admin_password_hash.clone() else {
        return Ok(false);
    };

    let now = Utc::now().naive_utc();
    let _ = admin_users::ActiveModel {
        id: Set(1),
        username: Set(config.admin_username.clone()),
        password_hash: Set(password_hash),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(conn)
    .await?;

    Ok(true)
}
