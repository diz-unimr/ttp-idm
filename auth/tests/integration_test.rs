use auth::oidc::{Claims, Oidc};
use axum::routing::get;
use axum::{middleware, Router};
use axum_test::TestServer;
use chrono::Utc;
use http::StatusCode;
use httpmock::Method::GET;
use httpmock::MockServer;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde_json::{json, Value};
use std::sync::Arc;

#[tokio::test]
async fn missing_authentication() {
    let (server, _) = setup_test_server().await;

    // send request
    let response = server.get("/").await;

    // unauthorized
    response.assert_status(StatusCode::UNAUTHORIZED);
    response.assert_text("Bearer token missing");
}

#[tokio::test]
async fn invalid_token() {
    let (server, _) = setup_test_server().await;

    // send request
    let response = server
        .get("/")
        .authorization_bearer("invalid access token")
        .await;

    // unauthorized
    response.assert_status(StatusCode::UNAUTHORIZED);
    response.assert_text("InvalidToken");
}

#[tokio::test]
async fn success() {
    let jwks = json!({
        "keys":[{
            "alg": "RS256",
            "e": "AQAB",
            "ext": true,
            "key_ops": ["verify"],
            "kty": "RSA",
            "n": "47MkTf-mQtyuT1PR0irLGgY2V5UAeDYBoDnjF5VF-pz_L8a3ECIItotN4Mf1mjpC-6NNRl6zgIQ3KbSG-S6MkpFKXL4r-2-ipcHKekZGLDlNWpgv-tYJGPPPINktCXoz6Cqxx9K4P3NvySLjRQODOGYpd8IusYYKyn8PM87rPulSMRutwbZszJh5Hfs9XF9G76EvXc6sSgs3dbtNN_5aMyazl1db3RPoysTfoLD7bvD3kmnUdWUid8cIy_cNrJujlsC_oM1CVFNiTLosYv-hJS6XGdHi7eMc0DAdMc0hfJVm9BKBs84tI-CV193i6bNJF5RZ33eDP18prDShbSOATQ",
            "kid": "v3rzXUDjZ4HSxxLLTI29ejhHBzv2SMQUSbk3nUug3qA=",
            "use": "sig"
        }]
    });

    let test_key = r#"-----BEGIN PRIVATE KEY-----
        MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDjsyRN/6ZC3K5P
        U9HSKssaBjZXlQB4NgGgOeMXlUX6nP8vxrcQIgi2i03gx/WaOkL7o01GXrOAhDcp
        tIb5LoySkUpcviv7b6Klwcp6RkYsOU1amC/61gkY888g2S0JejPoKrHH0rg/c2/J
        IuNFA4M4Zil3wi6xhgrKfw8zzus+6VIxG63BtmzMmHkd+z1cX0bvoS9dzqxKCzd1
        u003/lozJrOXV1vdE+jKxN+gsPtu8PeSadR1ZSJ3xwjL9w2sm6OWwL+gzUJUU2JM
        uixi/6ElLpcZ0eLt4xzQMB0xzSF8lWb0EoGzzi0j4JXX3eLps0kXlFnfd4M/Xyms
        NKFtI4BNAgMBAAECggEAAK1DPSm2kgBOuSGj2Z7fvhLcQOP9sMQuuqeXyzXAYLK5
        KqhjWhyGL7S35VeMaiO+MaB8orbpWODYh4ebCkz4uGU1XJOcIduYSitw5KHACtjP
        KA1hWlRRVprZUZATscsd/sfegc+Lu9ryyOneCKuuZvgdK3DCUfjDoD9C1k6V8gWF
        SQjY+gwt+k2eC3aQd5vD1rhdHFfdyg4Xwq0Rsy6Y11Oxs+uNAstgMzjH2N3Ah059
        tUKL1eQshRaZ7WnnvUi1+xkDIUbEmolzWjhoF7NR1McvNGHevXgHu/pLl0gR063O
        i09PkXeXYMLP25VIQqReHqcW/JdkFTwSvaNr6ALNAQKBgQD4gDtpHCVDUx3uGV7s
        vVRxv9xZ4tCVepbLKex5JUO8Shx7W4KlZ39iK2kPS/UvAwtO9w9Aq2ckmVkRrI+K
        7aNOW0YUH2YHO5HRFBjYrCeI/UHYGV9bPUqvxp2nRyhSBWJC6K+rPXoJXiUjvIL5
        phi5fci6lJgz4KUjTbQP9BO8oQKBgQDqkjZ2dcnmmyC46Hm2X6g1RZSx23uMqcL9
        a9WxkyCQ3llLI5Rou4LsV6Ffxs3w4NUUsiYl4A7MX9jrAzz+SrtFMC+52EW+5r8V
        mJ5JlflNHrSFv23mpywiYdhqYeQCySjcIztPZQjo9dgR6owBBL/u+46X05NA4Wmh
        S11r2FRYLQKBgGH4vHOOQyqt5Ejw+7m+U0Kdb9SIVc/5CuaCWtbQWEottdj0lSd9
        DH25u6vqOHoWayjwwrSuXvXQ94q+S8FsO0wzNAfO8Ty8wZp6n+kcxmF95625Ix0n
        pwBx/8nphf4AXWMftdJ/ZFO5KE9UjRa741eOPctBtlgNo02t3uXDRtzBAoGBAJ6f
        rVDCKnxVXvVr0BKx8S/FE96KS6w9iGyTJXjlw1nz4nJbZxrD4q8sOyZnbBB+Gdna
        9s0aDSfLkQars+1KYAVTppKIW/HSXFmgUTn1vxaVswHXB9y4I7JEdHLMK8JugcEL
        2inAaxwOU8UZ1P9DVP+pAS5Olv+C70lxi4VITxEpAoGBAJQIRvD7UBezr4K+M5kO
        7xPYNSk/LXXXaa3GDnRGQPPhJyEIcSsFPrQVdDIEQfbdzKpd/bfk2k6603qTSHgR
        exFfeOgb+NPUocNzKygFOqeWgF+ugefeAa6nw8b1tEdVxOmS7B9pkZ9oIhy2+xs7
        gmxwdyUdvwcZMDRwnuiMGNfC
        -----END PRIVATE KEY-----"#;

    let (server, mock) = setup_test_server_with_jwks(Some(jwks.clone())).await;
    let token = create_jwt(
        test_key,
        mock.base_url(),
        "v3rzXUDjZ4HSxxLLTI29ejhHBzv2SMQUSbk3nUug3qA=".into(),
    );

    // send request
    let response = server.get("/").authorization_bearer(token).await;

    // authorized response
    response.assert_status(StatusCode::OK);
    response.assert_text("Hello, World!");
}

async fn setup_test_server() -> (TestServer, MockServer) {
    setup_test_server_with_jwks(None).await
}

async fn setup_test_server_with_jwks(jwks: Option<Value>) -> (TestServer, MockServer) {
    let idp = MockServer::start();
    // discovery endpoint mock
    let discovery_mock = idp.mock(|when, then| {
        when.method(GET).path("/.well-known/openid-configuration");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "issuer": idp.base_url(),
                "authorization_endpoint": format!("{}/auth", idp.base_url()),
                "token_endpoint": format!("{}/token", idp.base_url()),
                "introspection_endpoint": format!("{}/introspect", idp.base_url()),
                "userinfo_endpoint": format!("{}/userinfo", idp.base_url()),
                "jwks_uri": format!("{}/certs", idp.base_url()),
            }));
    });

    if let Some(keys) = jwks {
        idp.mock(|when, then| {
            when.method(GET).path("/certs");
            then.status(200)
                .header("content-type", "application/json")
                .body(keys.to_string());
        });
    }

    // setup server with auth middleware
    let oidc = Arc::new(Oidc::new("test".into(), idp.base_url()).await.unwrap());
    // assert oidc discovery
    discovery_mock.assert();

    let router = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(middleware::from_fn_with_state(
            oidc,
            auth::oidc::auth_middleware,
        ));
    (TestServer::new(router).unwrap(), idp)
}

fn create_jwt(key: &str, iss: String, kid: String) -> String {
    let now = Utc::now();
    let expiration = now
        .checked_add_signed(chrono::Duration::seconds(6000))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: "test".into(),
        iat: now.timestamp() as usize,
        exp: expiration as usize,
        iss,
    };
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid);
    let res = encode(
        &header,
        &claims,
        &EncodingKey::from_rsa_pem(key.as_bytes()).unwrap(),
    );

    res.unwrap()
}
