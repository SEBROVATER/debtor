use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "auth_state")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub failed_attempt_count: i32,
    pub lockout_until: Option<DateTime>,
    pub last_failed_at: Option<DateTime>,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
