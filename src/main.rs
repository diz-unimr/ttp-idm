use crate::config::AppConfig;

mod config;
mod server;
mod api;
mod client;
mod error;
mod model;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // config
    let config = AppConfig::new().expect("Failed to load config");

    // run
    server::serve(config).await?;

    Ok(())
}
