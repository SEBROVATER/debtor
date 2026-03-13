use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sessions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub user_id: i32,
    pub token_hash: String,
    pub created_at: DateTime,
    pub last_seen_at: DateTime,
    pub expires_at: DateTime,
    pub revoked_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::admin_users::Entity",
        from = "Column::UserId",
        to = "super::admin_users::Column::Id",
        on_delete = "Cascade"
    )]
    AdminUser,
}

impl Related<super::admin_users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AdminUser.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
