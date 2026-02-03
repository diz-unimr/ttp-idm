use crate::config::AppConfig;
use crate::server::ApiBuild;
use anyhow::anyhow;
use log::{error, info};
use shadow_rs::shadow;

mod api;
mod config;
mod error;
mod model;
mod server;
mod ttp;

shadow!(build);

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
    match server::serve(
        config,
        ApiBuild {
            version: build::TAG.to_string(),
            mode: build::BUILD_RUST_CHANNEL.to_string(),
            time: build::BUILD_TIME.to_string(),
        },
    )
    .await
    {
        Ok(_) => info!("Server stopped"),
        Err(e) => {
            error!("Server stopped: {e}");
            return Err(e);
        }
    }

    Ok(())
}
