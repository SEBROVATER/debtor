use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "exchange_rates")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub from_currency: String,
    pub to_currency: String,
    pub rate: Decimal,
    pub fetched_at: DateTime,
    pub rate_date: Date,
    pub provider: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
