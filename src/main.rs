use crate::config::AppConfig;
use anyhow::anyhow;
use log::{error, info};

mod api;
mod config;
mod error;
mod model;
mod server;
mod ttp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // config
    let config = match AppConfig::new() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to load config");
            return Err(anyhow!(e));
        }
    };

    // run
    match server::serve(config).await {
        Ok(_) => info!("Server stopped"),
        Err(e) => {
            error!("Server stopped: {e}");
            return Err(e);
        }
    }

    Ok(())
}
