use crate::api;
use crate::client::FhirClient;
use crate::config::{AppConfig, Auth};
use axum::routing::get;
use axum::Router;
use log::info;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub(crate) struct ApiContext {
    pub(crate) auth: Option<Auth>,
    pub(crate) client: FhirClient,
}

async fn root() -> &'static str {
    "TTP ID Management API"
}

pub(crate) async fn serve(config: AppConfig) -> anyhow::Result<()> {
    let filter = format!(
        "{}={level},tower_http={level}",
        env!("CARGO_CRATE_NAME"),
        level = config.log_level
    );
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.into()))
        .init();

    // FHIR REST client
    let client = FhirClient::new(&config.ttp).await?;
    client.test_connection().await?;

    // context
    let state = ApiContext {
        auth: config.auth.clone(),
        client,
    };

    let router = Router::new()
        .route("/", get(root))
        .merge(api::router())
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Listening on {}", listener.local_addr()?);
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .map_err(|e| e.into())
}
