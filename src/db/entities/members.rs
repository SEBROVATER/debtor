use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "members")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub group_id: String,
    pub display_name: String,
    pub is_active: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub removed_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::groups::Entity",
        from = "Column::GroupId",
        to = "super::groups::Column::Id",
        on_delete = "Cascade"
    )]
    Group,
    #[sea_orm(has_many = "super::expense_shares::Entity")]
    ExpenseShares,
    #[sea_orm(has_many = "super::expenses::Entity")]
    Expenses,
}

impl Related<super::groups::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Group.def()
    }
}

impl Related<super::expense_shares::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ExpenseShares.def()
    }
}

impl Related<super::expenses::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Expenses.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
