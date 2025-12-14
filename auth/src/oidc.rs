use crate::oidc::AuthError::Client;
use async_oidc_jwt_validator::{OidcConfig, OidcValidator};
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use http::StatusCode;
use log::{debug, error};
use oauth2::basic::{BasicClient, BasicRequestTokenError};
use oauth2::url::ParseError;
use oauth2::{EndpointNotSet, EndpointSet};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    OAuth2(BasicRequestTokenError<<reqwest::Client as oauth2::AsyncHttpClient<'static>>::Error>),

    #[error(transparent)]
    JWT(jsonwebtoken::errors::Error),

    #[error(transparent)]
    ParseError(#[from] ParseError),

    #[error("OIDC client error: {0}")]
    Client(String),
}

pub type BasicClientSet =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
}

#[derive(Clone)]
pub struct Oidc {
    validator: OidcValidator,
}

impl Oidc {
    pub async fn new(client_id: String, issuer_url: String) -> Result<Oidc, AuthError> {
        let discovery: DiscoveryDocument = DiscoveryDocument::new(&issuer_url).await?;

        // jwt validation config
        let validation_config = OidcConfig::new(issuer_url, client_id, discovery.jwks_uri);
        let validator = OidcValidator::new(validation_config);

        Ok(Oidc { validator })
    }

    pub(crate) async fn authenticate(&self, token: &str) -> Result<(), AuthError> {
        match self.validator.validate::<Claims>(token).await {
            Ok(claims) => {
                debug!("Valid token for sub: {}", claims.sub);
                Ok(())
            }
            Err(e) => {
                error!("Bearer token validation failed: {}", e);
                Err(AuthError::JWT(e))
            }
        }
    }
}

pub async fn auth_middleware(
    State(state): State<Arc<Oidc>>,
    creds: Option<TypedHeader<Authorization<Bearer>>>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    if let Some(c) = creds {
        match state.authenticate(c.token()).await {
            Ok(_) => next.run(request).await,
            Err(e) => (StatusCode::UNAUTHORIZED, e.to_string()).into_response(),
        }
    } else {
        (StatusCode::UNAUTHORIZED, "Bearer token missing").into_response()
    }
}

#[derive(Deserialize)]
pub struct DiscoveryDocument {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub introspection_endpoint: String,
    pub userinfo_endpoint: String,
    pub jwks_uri: String,
}

impl DiscoveryDocument {
    pub async fn new(issuer_url: &str) -> Result<Self, AuthError> {
        discover(issuer_url).await
    }
}

async fn discover(issuer_url: &str) -> Result<DiscoveryDocument, AuthError> {
    let discovery_url = format!("{}/.well-known/openid-configuration", issuer_url);

    debug!("Fetching OpenID Connect discovery from: {}", discovery_url);

    let response = reqwest::get(&discovery_url).await?;

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    if !content_type.starts_with("application/json") {
        return Err(Client(format!(
            "Unexpected Content-Type: '{}', expected 'application/json'",
            content_type
        )));
    }

    if !response.status().is_success() {
        return Err(Client(format!(
            "OIDC discovery request failed with status: {}",
            response.status()
        )));
    }

    let discovery: DiscoveryDocument = response
        .json()
        .await
        .map_err(|e| AuthError::Client(format!("Failed to parse OIDC discovery response: {e}")))?;

    Ok(discovery)
}
