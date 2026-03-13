use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "expenses")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub group_id: String,
    pub payer_member_id: String,
    pub amount: Decimal,
    pub currency: String,
    pub note: Option<String>,
    pub expense_date: Date,
    pub created_at: DateTime,
    pub updated_at: DateTime,
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
    #[sea_orm(
        belongs_to = "super::members::Entity",
        from = "Column::PayerMemberId",
        to = "super::members::Column::Id",
        on_delete = "Restrict"
    )]
    Payer,
    #[sea_orm(has_many = "super::expense_shares::Entity")]
    ExpenseShares,
}

impl Related<super::groups::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Group.def()
    }
}

impl Related<super::members::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Payer.def()
    }
}

impl Related<super::expense_shares::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ExpenseShares.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
