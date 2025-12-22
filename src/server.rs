use crate::api;
use crate::config::AppConfig;
use crate::model;
use crate::ttp::client::TtpClient;
use auth::oidc::Oidc as OidcAuth;
use axum::routing::get;
use axum::{middleware, Router};
use log::info;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;
use utoipa::openapi::security::{ClientCredentials, Flow, OAuth2, Scopes, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::{Config, SwaggerUi};

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

    // api state
    let state = Arc::new(ApiContext { client });
    // auth state
    let auth_state = match config
        .auth
        .and_then(|auth| auth.oidc)
        .map(|o| OidcAuth::new(o.client_id, o.issuer_url))
    {
        None => None,
        Some(res) => Some(Arc::new(res.await?)),
    };

    let router = build_router(state, auth_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Listening on {}", listener.local_addr()?);
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .map_err(|e| e.into())
}

fn build_router(api_state: Arc<ApiContext>, auth_state: Option<Arc<OidcAuth>>) -> Router {
    api_route(auth_state)
        .route("/", get(root))
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi())
                .config(Config::default().try_it_out_enabled(false)),
        )
        .with_state(api_state)
        .layer(TraceLayer::new_for_http())
}

fn api_route(auth_state: Option<Arc<OidcAuth>>) -> Router<Arc<ApiContext>> {
    if let Some(auth) = auth_state {
        api::router().layer(middleware::from_fn_with_state(
            auth,
            auth::oidc::auth_middleware,
        ))
    } else {
        api::router()
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        root,
        api::create,
        api::read,
    ),
    components(schemas(
        model::IdRequest,
        model::IdResponse,
        model::Idat,
        model::PromptResponse,
        model::Link,
    )),
    modifiers(&SecurityAddon),
    tags((name = "Pseudonym management"))
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "oauth",
                SecurityScheme::OAuth2(OAuth2::new([Flow::ClientCredentials(
                    ClientCredentials::new("https://localhost/token", Scopes::new()),
                )])),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use std::sync::Arc;

    #[tokio::test]
    async fn root_test() {
        let config = AppConfig::default();
        {
            let state = Arc::new(ApiContext {
                client: TtpClient::new(&config.ttp).await.unwrap(),
            });

            // test server
            let router = build_router(state, None);
            let server = TestServer::new(router).unwrap();

            // send request
            let response = server.get("/").await;

            // assert
            response.assert_status_ok();
            response.assert_text("TTP ID Management API");
        }
    }
}
