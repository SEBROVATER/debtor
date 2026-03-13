use sea_orm_migration::prelude::*;

use debtor_migration::Migrator;

#[tokio::main]
async fn main() {
    cli::run_cli(Migrator).await;
}
