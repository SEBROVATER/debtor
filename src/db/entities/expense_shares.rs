use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "expense_shares")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub expense_id: String,
    pub member_id: String,
    pub share_mode: String,
    pub share_value: Decimal,
    pub computed_amount: Decimal,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::expenses::Entity",
        from = "Column::ExpenseId",
        to = "super::expenses::Column::Id",
        on_delete = "Cascade"
    )]
    Expense,
    #[sea_orm(
        belongs_to = "super::members::Entity",
        from = "Column::MemberId",
        to = "super::members::Column::Id",
        on_delete = "Restrict"
    )]
    Member,
}

impl Related<super::expenses::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Expense.def()
    }
}

impl Related<super::members::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Member.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
