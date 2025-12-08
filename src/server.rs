use crate::api;
use crate::config::AppConfig;
use crate::model;
use crate::ttp::client::TtpClient;
use axum::routing::get;
use axum::Router;
use log::info;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub(crate) struct ApiContext {
    pub(crate) client: TtpClient,
}

/// API metadata
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "TTP ID Management Web API", body = str),
    ),
    tag = "metadata"
)]
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

    // TTP client
    let client = TtpClient::new(&config.ttp).await?;
    client.test_connection().await?;
    client.setup_domains().await?;

    // context
    let state = Arc::new(ApiContext { client });

    let router = build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Listening on {}", listener.local_addr()?);
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .map_err(|e| e.into())
}

fn build_router(state: Arc<ApiContext>) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/", get(root))
        .merge(api::router())
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

#[derive(OpenApi)]
#[openapi(
    paths(
        root,
        api::create,
    ),
    components(schemas(
        model::IdResponse,
        model::Idat,
        model::PromptResponse,
        model::Link,
    )),
    tags((name = "pseudonymization"))
)]
struct ApiDoc;

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use std::sync::Arc;

    #[tokio::test]
    async fn root_test() {
        let config = AppConfig {
            log_level: "".to_string(),
            ttp: Default::default(),
        };
        let state = Arc::new(ApiContext {
            client: TtpClient::new(&config.ttp).await.unwrap(),
        });

        // test server
        let router = build_router(state);
        let server = TestServer::new(router).unwrap();

        // send request
        let response = server.get("/").await;

        // assert
        response.assert_status_ok();
        response.assert_text("TTP ID Management API");
    }
}
