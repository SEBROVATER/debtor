use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "groups")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub name: String,
    pub target_currency: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::members::Entity")]
    Members,
    #[sea_orm(has_many = "super::expenses::Entity")]
    Expenses,
}

impl Related<super::members::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Members.def()
    }
}

impl Related<super::expenses::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Expenses.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
