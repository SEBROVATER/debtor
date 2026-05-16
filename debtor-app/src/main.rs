#![warn(missing_docs)]

//! debtor — Composition root.
//!
//! This binary crate wires together all workspace crates:
//! - Creates the `SQLx` connection pool
//! - Instantiates concrete infrastructure adapters
//! - Builds the Axum router with middleware
//! - Starts the HTTP server

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debtor_app=debug,tower_http=debug".into()),
        )
        .init();

    tracing::info!("debtor starting...");

    Ok(())
}
