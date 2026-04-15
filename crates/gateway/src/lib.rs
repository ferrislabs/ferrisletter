use std::{
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    Json, Router,
    body::Body,
    extract::{RawQuery, Request, State},
    http::{HeaderMap, HeaderName, StatusCode, Uri, header::WWW_AUTHENTICATE},
    response::{IntoResponse, Response},
    routing::any,
};
use bytes::Bytes;
use clap::Parser;
use jsonwebtoken::{
    Algorithm, DecodingKey, Validation, decode, decode_header,
    jwk::{Jwk, JwkSet},
};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, warn};

#[derive(Debug, Clone, Parser)]
#[command(name = "ferrisletter-gateway", about = "Ferrisletter MCP gateway")]
pub struct GatewayConfig {
    #[arg(
        long,
        env = "GATEWAY_LISTEN_ADDR",
        default_value = "0.0.0.0:9292",
        hide_env_values = true
    )]
    pub listen_addr: SocketAddr,

    #[arg(
        long,
        env = "OIDC_PROVIDER",
        default_value = "ferriskey",
        hide_env_values = true
    )]
    pub oidc_provider: OidcProvider,

    #[arg(long, env = "OIDC_JWKS_URL", hide_env_values = true)]
    pub oidc_jwks_url: String,

    #[arg(long, env = "OIDC_EXPECTED_ISSUER", hide_env_values = true)]
    pub oidc_expected_issuer: String,

    #[arg(long, env = "OIDC_AUTHORIZATION_ENDPOINT", hide_env_values = true)]
    pub oidc_authorization_endpoint: String,

    #[arg(long, env = "OIDC_TOKEN_ENDPOINT", hide_env_values = true)]
    pub oidc_token_endpoint: String,

    #[arg(
        long,
        env = "OIDC_SCOPES_SUPPORTED",
        value_delimiter = ',',
        default_value = "openid,profile,email",
        hide_env_values = true
    )]
    pub oidc_scopes_supported: Vec<String>,

    #[arg(long, env = "MCP_SERVER_URL", hide_env_values = true)]
    pub mcp_server_url: String,

    #[arg(
        long,
        env = "JWKS_CACHE_TTL_SECS",
        default_value_t = 300,
        hide_env_values = true
    )]
    pub jwks_cache_ttl_secs: u64,

    #[arg(
        long,
        env = "MCP_PROXY_TIMEOUT_SECS",
        default_value_t = 30,
        hide_env_values = true
    )]
    pub mcp_proxy_timeout_secs: u64,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OidcProvider {
    Ferriskey,
}

#[derive(Clone)]
pub struct AppState {
    validator: JwtValidator,
    mcp_server_url: String,
    upstream_client: HttpClient,
    required_scopes: Vec<String>,
    oidc_metadata: AuthorizationServerMetadata,
    oidc_authorization_endpoint: String,
    oidc_token_endpoint: String,
}

impl AppState {
    pub fn new(config: GatewayConfig) -> Result<Self, GatewayError> {
        let oidc_expected_issuer = config
            .oidc_expected_issuer
            .trim_end_matches('/')
            .to_string();
        let oidc_jwks_url = config.oidc_jwks_url.trim_end_matches('/').to_string();
        let oidc_authorization_endpoint = config
            .oidc_authorization_endpoint
            .trim_end_matches('/')
            .to_string();
        let oidc_token_endpoint = config.oidc_token_endpoint.trim_end_matches('/').to_string();
        let upstream_client = HttpClient::builder()
            .timeout(Duration::from_secs(config.mcp_proxy_timeout_secs))
            .build()
            .map_err(|err| GatewayError::internal(format!("failed to build HTTP client: {err}")))?;

        Ok(Self {
            validator: JwtValidator::new(
                config.oidc_provider,
                oidc_jwks_url.clone(),
                oidc_expected_issuer.clone(),
                Duration::from_secs(config.jwks_cache_ttl_secs),
            ),
            mcp_server_url: config.mcp_server_url.trim_end_matches('/').to_string(),
            upstream_client,
            required_scopes: config.oidc_scopes_supported.clone(),
            oidc_metadata: AuthorizationServerMetadata {
                issuer: oidc_expected_issuer,
                authorization_endpoint: oidc_authorization_endpoint.clone(),
                token_endpoint: oidc_token_endpoint.clone(),
                jwks_uri: oidc_jwks_url,
                response_types_supported: vec!["code".to_string()],
                grant_types_supported: vec!["authorization_code".to_string()],
                code_challenge_methods_supported: vec!["S256".to_string()],
                scopes_supported: config.oidc_scopes_supported,
                token_endpoint_auth_methods_supported: vec![
                    "client_secret_basic".to_string(),
                    "client_secret_post".to_string(),
                    "none".to_string(),
                ],
            },
            oidc_authorization_endpoint,
            oidc_token_endpoint,
        })
    }
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/mcp", any(mcp_proxy))
        .route(
            "/.well-known/oauth-protected-resource",
            any(protected_resource_metadata),
        )
        .route(
            "/.well-known/oauth-protected-resource/mcp",
            any(protected_resource_metadata),
        )
        .route(
            "/.well-known/oauth-authorization-server",
            any(oauth_authorization_server_metadata),
        )
        .route(
            "/.well-known/oauth-authorization-server/mcp",
            any(oauth_authorization_server_metadata),
        )
        .route(
            "/.well-known/openid-configuration",
            any(openid_configuration),
        )
        .route(
            "/.well-known/openid-configuration/mcp",
            any(openid_configuration),
        )
        .route("/authorize", any(compat_authorize))
        .route("/token", any(compat_token))
        .with_state(state)
}

async fn mcp_proxy(
    State(state): State<AppState>,
    request: Request,
) -> Result<Response, GatewayError> {
    let (parts, body) = request.into_parts();
    let token = extract_bearer_token(&parts.headers)
        .ok_or_else(|| GatewayError::unauthorized_mcp(&state, &parts.headers))?
        .to_string();

    state.validator.validate(&token).await?;

    let body = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|err| GatewayError::bad_request(format!("failed to read request body: {err}")))?;

    send_upstream(
        &state.mcp_server_url,
        &state.upstream_client,
        &parts.method,
        &parts.uri,
        &parts.headers,
        body,
    )
    .await
}

async fn send_upstream(
    upstream_base: &str,
    client: &HttpClient,
    method: &axum::http::Method,
    uri: &Uri,
    headers: &HeaderMap,
    body: Bytes,
) -> Result<Response, GatewayError> {
    let upstream_url = build_upstream_url(upstream_base, uri);
    debug!(%upstream_url, "forwarding request to MCP server");

    let mut req = client
        .request(method.clone(), &upstream_url)
        .header("Accept", "application/json, text/event-stream")
        .body(body);

    for (name, value) in filtered_request_headers(headers) {
        req = req.header(name, value);
    }

    let upstream_response = req.send().await.map_err(|err| {
        warn!(%upstream_url, error = %err, "failed to reach MCP server");
        GatewayError::bad_gateway(format!("failed to reach MCP server: {err}"))
    })?;

    build_proxy_response(upstream_response)
}

fn build_proxy_response(upstream_response: reqwest::Response) -> Result<Response, GatewayError> {
    let status = upstream_response.status();
    let headers = upstream_response.headers().clone();
    let stream = upstream_response.bytes_stream();

    let mut response = Response::builder().status(status);
    for (name, value) in filtered_response_headers(&headers) {
        response = response.header(name, value);
    }

    response
        .body(Body::from_stream(stream))
        .map_err(|err| GatewayError::internal(format!("failed to build proxy response: {err}")))
}

async fn protected_resource_metadata(
    State(state): State<AppState>,
    request: Request,
) -> Result<Json<ProtectedResourceMetadata>, GatewayError> {
    let resource = request_base_url(&request)
        .map(|base| format!("{base}/mcp"))
        .unwrap_or_else(|| "/mcp".to_string());

    Ok(Json(ProtectedResourceMetadata {
        resource,
        authorization_servers: vec![state.oidc_metadata.issuer.clone()],
        bearer_methods_supported: vec!["header".to_string()],
        scopes_supported: state.required_scopes.clone(),
    }))
}

async fn oauth_authorization_server_metadata(
    State(state): State<AppState>,
) -> Result<Json<AuthorizationServerMetadata>, GatewayError> {
    Ok(Json(state.oidc_metadata.clone()))
}

async fn openid_configuration(
    State(state): State<AppState>,
) -> Result<Json<AuthorizationServerMetadata>, GatewayError> {
    Ok(Json(state.oidc_metadata.clone()))
}

async fn compat_authorize(
    State(state): State<AppState>,
    RawQuery(query): RawQuery,
) -> Result<Response, GatewayError> {
    let location = match query {
        Some(query) if !query.is_empty() => {
            format!("{}?{query}", state.oidc_authorization_endpoint)
        }
        _ => state.oidc_authorization_endpoint.clone(),
    };

    Response::builder()
        .status(StatusCode::FOUND)
        .header(axum::http::header::LOCATION, location)
        .body(Body::empty())
        .map_err(|err| GatewayError::internal(format!("failed to build authorize redirect: {err}")))
}

async fn compat_token(
    State(state): State<AppState>,
    request: Request,
) -> Result<Response, GatewayError> {
    let (parts, body) = request.into_parts();
    let body = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|err| {
            GatewayError::bad_request(format!("failed to read token request body: {err}"))
        })?;

    let mut upstream_request = state
        .upstream_client
        .request(parts.method.clone(), state.oidc_token_endpoint.clone())
        .body(body);

    for (name, value) in filtered_request_headers(&parts.headers) {
        upstream_request = upstream_request.header(name, value);
    }

    let upstream_response = upstream_request.send().await.map_err(|err| {
        GatewayError::bad_gateway(format!("failed to reach OIDC token endpoint: {err}"))
    })?;

    build_proxy_response(upstream_response)
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    let header = headers.get(axum::http::header::AUTHORIZATION)?;
    let value = header.to_str().ok()?;
    let token = value.strip_prefix("Bearer ")?;
    if token.is_empty() { None } else { Some(token) }
}

fn build_upstream_url(base: &str, uri: &Uri) -> String {
    let path_and_query = uri
        .path_and_query()
        .map(|value| value.as_str())
        .unwrap_or("/");
    let query = uri.query().map(|value| format!("?{value}"));

    if base.ends_with(uri.path()) {
        format!("{}{}", base, query.as_deref().unwrap_or_default())
    } else {
        format!("{base}{path_and_query}")
    }
}

fn request_base_url(request: &Request) -> Option<String> {
    let headers = request.headers();
    let host = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get(axum::http::header::HOST))?
        .to_str()
        .ok()?;
    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("http");

    Some(format!("{proto}://{host}"))
}

fn filtered_request_headers(headers: &HeaderMap) -> Vec<(HeaderName, String)> {
    headers
        .iter()
        .filter_map(|(name, value)| {
            if is_hop_by_hop_header(name) || name == axum::http::header::AUTHORIZATION {
                return None;
            }
            value
                .to_str()
                .ok()
                .map(|value| (name.clone(), value.to_string()))
        })
        .collect()
}

fn filtered_response_headers(headers: &HeaderMap) -> Vec<(HeaderName, String)> {
    headers
        .iter()
        .filter_map(|(name, value)| {
            if is_hop_by_hop_header(name) {
                return None;
            }
            value
                .to_str()
                .ok()
                .map(|value| (name.clone(), value.to_string()))
        })
        .collect()
}

fn is_hop_by_hop_header(name: &HeaderName) -> bool {
    matches!(
        name.as_str().to_ascii_lowercase().as_str(),
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
            | "host"
            | "content-length"
    )
}

#[derive(Debug, Clone, Serialize)]
struct ProtectedResourceMetadata {
    resource: String,
    authorization_servers: Vec<String>,
    bearer_methods_supported: Vec<String>,
    scopes_supported: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct AuthorizationServerMetadata {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    jwks_uri: String,
    response_types_supported: Vec<String>,
    grant_types_supported: Vec<String>,
    code_challenge_methods_supported: Vec<String>,
    scopes_supported: Vec<String>,
    token_endpoint_auth_methods_supported: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtClaims {
    pub exp: u64,
    pub iss: String,
    pub sub: String,
}

#[derive(Clone)]
pub struct JwtValidator {
    provider: OidcProvider,
    client: HttpClient,
    jwks_url: String,
    expected_issuer: String,
    cache_ttl: Duration,
    cache: Arc<RwLock<Option<CachedJwks>>>,
}

#[derive(Clone)]
struct CachedJwks {
    fetched_at: Instant,
    jwks: JwkSet,
}

impl JwtValidator {
    pub fn new(
        provider: OidcProvider,
        jwks_url: String,
        expected_issuer: String,
        cache_ttl: Duration,
    ) -> Self {
        Self {
            provider,
            client: HttpClient::new(),
            jwks_url,
            expected_issuer,
            cache_ttl,
            cache: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn validate(&self, token: &str) -> Result<JwtClaims, GatewayError> {
        let header = decode_header(token).map_err(|err| {
            GatewayError::unauthorized(format!(
                "invalid {} JWT header: {err}",
                oidc_provider_label(self.provider)
            ))
        })?;

        if header.alg != Algorithm::RS256 {
            return Err(GatewayError::unauthorized(format!(
                "unsupported {} JWT alg {:?}",
                oidc_provider_label(self.provider),
                header.alg,
            )));
        }

        let jwk = match self.find_jwk(header.kid.as_deref(), false).await? {
            Some(jwk) => jwk,
            None => self
                .find_jwk(header.kid.as_deref(), true)
                .await?
                .ok_or_else(|| GatewayError::unauthorized("no matching JWK found"))?,
        };

        let decoding_key = DecodingKey::from_jwk(&jwk).map_err(|err| {
            GatewayError::unauthorized(format!(
                "invalid {} JWK: {err}",
                oidc_provider_label(self.provider)
            ))
        })?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_aud = false;
        validation.set_issuer(&[self.expected_issuer.as_str()]);
        validation.set_required_spec_claims(&["exp", "iss", "sub"]);

        decode::<JwtClaims>(token, &decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|err| {
                GatewayError::unauthorized(format!(
                    "invalid {} JWT: {err}",
                    oidc_provider_label(self.provider)
                ))
            })
    }

    async fn find_jwk(
        &self,
        kid: Option<&str>,
        force_refresh: bool,
    ) -> Result<Option<Jwk>, GatewayError> {
        let jwks = self.get_jwks(force_refresh).await?;

        if let Some(kid) = kid {
            Ok(jwks
                .keys
                .into_iter()
                .find(|jwk| jwk.common.key_id.as_deref() == Some(kid)))
        } else if jwks.keys.len() == 1 {
            Ok(jwks.keys.into_iter().next())
        } else {
            Ok(None)
        }
    }

    async fn get_jwks(&self, force_refresh: bool) -> Result<JwkSet, GatewayError> {
        {
            let cache = self.cache.read().await;
            if !force_refresh {
                if let Some(cache) = cache.as_ref() {
                    if cache.fetched_at.elapsed() < self.cache_ttl {
                        return Ok(cache.jwks.clone());
                    }
                }
            }
        }

        let response = self
            .client
            .get(&self.jwks_url)
            .send()
            .await
            .map_err(|err| {
                GatewayError::service_unavailable(format!(
                    "failed to fetch {} JWKS: {err}",
                    oidc_provider_label(self.provider)
                ))
            })?;

        if !response.status().is_success() {
            return Err(GatewayError::service_unavailable(format!(
                "{} JWKS endpoint returned {}",
                oidc_provider_label(self.provider),
                response.status()
            )));
        }

        let jwks: JwkSet = response.json().await.map_err(|err| {
            GatewayError::service_unavailable(format!(
                "failed to parse {} JWKS: {err}",
                oidc_provider_label(self.provider)
            ))
        })?;

        let mut cache = self.cache.write().await;
        *cache = Some(CachedJwks {
            fetched_at: Instant::now(),
            jwks: jwks.clone(),
        });

        Ok(jwks)
    }
}

fn oidc_provider_label(provider: OidcProvider) -> &'static str {
    match provider {
        OidcProvider::Ferriskey => "Ferriskey",
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct GatewayError {
    status: StatusCode,
    message: String,
    headers: Vec<(HeaderName, String)>,
}

impl GatewayError {
    fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
            headers: Vec::new(),
        }
    }

    fn unauthorized_mcp(state: &AppState, headers: &HeaderMap) -> Self {
        let scope = state.required_scopes.join(" ");
        let base = headers
            .get("x-forwarded-host")
            .or_else(|| headers.get(axum::http::header::HOST))
            .and_then(|value| value.to_str().ok())
            .map(|host| {
                let proto = headers
                    .get("x-forwarded-proto")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or("http");
                format!("{proto}://{host}")
            })
            .unwrap_or_default();
        let resource_metadata = if base.is_empty() {
            "/.well-known/oauth-protected-resource/mcp".to_string()
        } else {
            format!("{base}/.well-known/oauth-protected-resource/mcp")
        };
        let header_value =
            format!("Bearer resource_metadata=\"{resource_metadata}\", scope=\"{scope}\"");

        Self {
            status: StatusCode::UNAUTHORIZED,
            message: "missing or invalid bearer token".to_string(),
            headers: vec![(WWW_AUTHENTICATE, header_value)],
        }
    }

    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
            headers: Vec::new(),
        }
    }

    fn bad_gateway(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            message: message.into(),
            headers: Vec::new(),
        }
    }

    fn service_unavailable(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            message: message.into(),
            headers: Vec::new(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
            headers: Vec::new(),
        }
    }
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let mut response = Response::builder().status(self.status);
        for (name, value) in self.headers {
            response = response.header(name, value);
        }

        let body = serde_json::json!({
            "error": self.message,
        });

        response
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap_or_default()))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::build_upstream_url;

    #[test]
    fn upstream_url_accepts_server_base_url() {
        let uri = "/mcp?session=abc".parse().unwrap();

        assert_eq!(
            build_upstream_url("http://127.0.0.1:8080", &uri),
            "http://127.0.0.1:8080/mcp?session=abc"
        );
    }

    #[test]
    fn upstream_url_accepts_mcp_endpoint_url() {
        let uri = "/mcp?session=abc".parse().unwrap();

        assert_eq!(
            build_upstream_url("http://127.0.0.1:8080/mcp", &uri),
            "http://127.0.0.1:8080/mcp?session=abc"
        );
    }
}
