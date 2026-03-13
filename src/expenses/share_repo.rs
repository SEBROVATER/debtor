use chrono::NaiveDateTime;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::db::entities::expense_shares;
use crate::expenses::share_splitter::{ShareAllocation, ShareMode};

#[derive(Clone)]
pub struct ShareRepo {
    conn: DatabaseConnection,
}

impl ShareRepo {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    pub async fn list_by_expense(
        &self,
        expense_id: &str,
    ) -> Result<Vec<expense_shares::Model>, DbErr> {
        expense_shares::Entity::find()
            .filter(expense_shares::Column::ExpenseId.eq(expense_id))
            .all(&self.conn)
            .await
    }

    pub async fn replace_shares(
        &self,
        expense_id: &str,
        shares: Vec<ShareAllocation>,
        now: NaiveDateTime,
    ) -> Result<Vec<expense_shares::Model>, DbErr> {
        expense_shares::Entity::delete_many()
            .filter(expense_shares::Column::ExpenseId.eq(expense_id))
            .exec(&self.conn)
            .await?;

        if shares.is_empty() {
            return Ok(Vec::new());
        }

        let models = shares
            .into_iter()
            .map(|share| expense_shares::ActiveModel {
                id: Set(Uuid::new_v4().to_string()),
                expense_id: Set(expense_id.to_string()),
                member_id: Set(share.member_id),
                share_mode: Set(mode_to_string(share.mode)),
                share_value: Set(share.share_value),
                computed_amount: Set(share.computed_amount),
                created_at: Set(now),
                updated_at: Set(now),
            })
            .collect::<Vec<_>>();

        expense_shares::Entity::insert_many(models)
            .exec(&self.conn)
            .await?;

        self.list_by_expense(expense_id).await
    }
}

fn mode_to_string(mode: ShareMode) -> String {
    match mode {
        ShareMode::Equal => "equal".to_string(),
        ShareMode::Percent => "percent".to_string(),
        ShareMode::Amount => "amount".to_string(),
    }
}
