use std::env;

use debtor::app::config::AppConfig;
use debtor::app::state::AppState;
use debtor::db::bootstrap::{bootstrap_admin_user, initialize_database};
use debtor::exchange_rates::frankfurter_client::FrankfurterClient;
use std::sync::Arc;

fn load_dotenv() {
    // Try loading .env; if not found, silently continue with system env
    let iter = match dotenvy::dotenv_iter() {
        Ok(iter) => iter,
        Err(dotenvy::Error::Io(ref err)) if err.kind() == std::io::ErrorKind::NotFound => {
            return; // No .env file — use system environment only
        }
        Err(err) => {
            eprintln!("FATAL: Failed to load .env file: {err}");
            std::process::exit(1);
        }
    };

    // System environment takes precedence: only set vars not already present
    for item in iter {
        let (key, value) = match item {
            Ok(pair) => pair,
            Err(err) => {
                eprintln!("FATAL: Failed to parse .env file: {err}");
                std::process::exit(1);
            }
        };
        if env::var(&key).is_err() {
            unsafe { env::set_var(&key, &value) };
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_dotenv();

    let config = AppConfig::from_env()?;
    let db = initialize_database(&config.database_url).await?;
    let _ = bootstrap_admin_user(&db, &config).await?;
    let provider = Arc::new(FrankfurterClient::with_base_url(&config.exchange_base_url));
    let _state = AppState::new(config, db, Some(provider));

    println!("debtor bootstrap initialized");
    Ok(())
}
