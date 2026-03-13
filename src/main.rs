use debtor::app::config::AppConfig;
use debtor::app::state::AppState;
use debtor::db::bootstrap::{bootstrap_admin_user, initialize_database};
use debtor::exchange_rates::frankfurter_client::FrankfurterClient;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env()?;
    let db = initialize_database(&config.database_url).await?;
    let _ = bootstrap_admin_user(&db, &config).await?;
    let provider = Arc::new(FrankfurterClient::with_base_url(&config.exchange_base_url));
    let _state = AppState::new(config, db, Some(provider));

    println!("debtor bootstrap initialized");
    Ok(())
}
