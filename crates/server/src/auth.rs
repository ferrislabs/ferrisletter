//! Pluggable authentication — generic IAM integration for the platform.
//!
//! The [`AuthProvider`] trait defines the interface. Two implementations:
//!
//! - [`NoAuthProvider`] — always returns anonymous; default for dev/stdio.
//! - [`OidcAuthProvider`] — validates JWT bearer tokens via JWKS from any
//!   OIDC-compliant identity provider (Keycloak, Auth0, Cloud-IAM, etc.).
//!
//! External crates (e.g. Lattice) can provide custom implementations.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// An authenticated (or anonymous) user identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    /// Unique user identifier (e.g. UUID from the IdP, or `"anonymous"`).
    pub user_id: String,
    /// External identifier from the IdP (e.g. Keycloak `sub` claim).
    pub external_id: Option<String>,
    /// Email address, if known.
    pub email: Option<String>,
    /// Display name, if known.
    pub name: Option<String>,
}

impl AuthUser {
    /// Construct an anonymous user (for stdio / no-auth mode).
    pub fn anonymous() -> Self {
        Self {
            user_id: "anonymous".to_string(),
            external_id: None,
            email: None,
            name: None,
        }
    }

    pub fn is_anonymous(&self) -> bool {
        self.user_id == "anonymous"
    }
}

/// Async trait for authentication providers.
///
/// Follows the same pattern as [`crate::FavoriteStore`] and [`crate::UserStore`]:
/// concrete trait with RPIT, plus a type-erased [`BoxedAuthProvider`] wrapper.
pub trait AuthProvider: Send + Sync {
    /// Validate a bearer token and return the authenticated user.
    ///
    /// Returns `Ok(None)` if the token is invalid or expired (caller should
    /// return 401). Returns `Ok(Some(user))` on success.
    fn authenticate(
        &self,
        token: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<Option<AuthUser>>> + Send;

    /// OIDC authorization server URLs that this resource delegates to.
    ///
    /// Used in the `/.well-known/oauth-protected-resource` response.
    fn authorization_servers(&self) -> Vec<String>;

    /// OAuth scopes supported by this resource.
    fn scopes_supported(&self) -> Vec<String>;
}

// ── NoAuthProvider ───────────────────────────────────────────────────────

/// Always returns an anonymous user. Default for dev/stdio mode.
pub struct NoAuthProvider;

impl AuthProvider for NoAuthProvider {
    async fn authenticate(&self, _token: &str) -> anyhow::Result<Option<AuthUser>> {
        Ok(Some(AuthUser::anonymous()))
    }

    fn authorization_servers(&self) -> Vec<String> {
        Vec::new()
    }

    fn scopes_supported(&self) -> Vec<String> {
        Vec::new()
    }
}

// ── OidcAuthProvider ─────────────────────────────────────────────────────

/// Validates JWT bearer tokens against an OIDC provider's JWKS endpoint.
///
/// On construction, fetches `<issuer_url>/.well-known/openid-configuration`
/// to discover the `jwks_uri`, then fetches the JWKS key set.
pub struct OidcAuthProvider {
    /// OIDC issuer URL (e.g. `https://ciam.cloud-iam.com/realms/lattice`).
    issuer_url: String,
    /// Expected audience claim (e.g. `https://mcp.lattice.dev`).
    audience: String,
    /// Cached JWKS decoding keys.
    jwks: Arc<tokio::sync::RwLock<jsonwebtoken::jwk::JwkSet>>,
    /// HTTP client for key refresh.
    http: reqwest::Client,
}

/// OIDC discovery document (subset of fields we need).
#[derive(Deserialize)]
struct OidcDiscovery {
    jwks_uri: String,
    #[serde(default)]
    #[allow(dead_code)]
    authorization_endpoint: String,
    issuer: String,
}

impl OidcAuthProvider {
    /// Create a provider by performing OIDC discovery against the issuer.
    ///
    /// Fetches `<issuer_url>/.well-known/openid-configuration` and then the
    /// JWKS key set. Fails if discovery or JWKS fetch fails.
    pub async fn discover(issuer_url: &str, audience: &str) -> anyhow::Result<Self> {
        let http = reqwest::Client::new();
        let discovery_url = format!(
            "{}/.well-known/openid-configuration",
            issuer_url.trim_end_matches('/')
        );

        tracing::info!(%discovery_url, "fetching OIDC discovery document");
        let discovery: OidcDiscovery = http
            .get(&discovery_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        tracing::info!(jwks_uri = %discovery.jwks_uri, issuer = %discovery.issuer, "OIDC discovery complete");

        let jwks: jsonwebtoken::jwk::JwkSet = http
            .get(&discovery.jwks_uri)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        tracing::info!(keys = jwks.keys.len(), "JWKS loaded");

        Ok(Self {
            issuer_url: discovery.issuer,
            audience: audience.to_string(),
            jwks: Arc::new(tokio::sync::RwLock::new(jwks)),
            http,
        })
    }

    /// Try to find a decoding key from the JWKS for the given key ID.
    fn find_decoding_key(
        jwks: &jsonwebtoken::jwk::JwkSet,
        kid: &str,
    ) -> Option<jsonwebtoken::DecodingKey> {
        jwks.find(kid)
            .and_then(|jwk| jsonwebtoken::DecodingKey::from_jwk(jwk).ok())
    }

    /// Refresh the JWKS from the issuer (e.g. when a `kid` is not found).
    async fn refresh_jwks(&self) -> anyhow::Result<()> {
        let discovery_url = format!(
            "{}/.well-known/openid-configuration",
            self.issuer_url.trim_end_matches('/')
        );
        let discovery: OidcDiscovery = self
            .http
            .get(&discovery_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let new_jwks: jsonwebtoken::jwk::JwkSet = self
            .http
            .get(&discovery.jwks_uri)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        tracing::info!(keys = new_jwks.keys.len(), "JWKS refreshed");
        *self.jwks.write().await = new_jwks;
        Ok(())
    }
}

/// Standard JWT claims we extract.
#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    preferred_username: Option<String>,
}

impl AuthProvider for OidcAuthProvider {
    async fn authenticate(&self, token: &str) -> anyhow::Result<Option<AuthUser>> {
        // Decode the header to get the `kid`.
        let header = match jsonwebtoken::decode_header(token) {
            Ok(h) => h,
            Err(e) => {
                tracing::debug!("invalid JWT header: {e}");
                return Ok(None);
            }
        };

        let kid = match header.kid.as_deref() {
            Some(k) => k,
            None => {
                tracing::debug!("JWT missing kid claim");
                return Ok(None);
            }
        };

        // Try to find the key; if not found, refresh JWKS once and retry.
        let jwks = self.jwks.read().await;
        let decoding_key = match Self::find_decoding_key(&jwks, kid) {
            Some(key) => key,
            None => {
                drop(jwks);
                tracing::debug!(kid, "kid not in JWKS, refreshing");
                if let Err(e) = self.refresh_jwks().await {
                    tracing::warn!("JWKS refresh failed: {e}");
                    return Ok(None);
                }
                let jwks = self.jwks.read().await;
                match Self::find_decoding_key(&jwks, kid) {
                    Some(key) => key,
                    None => {
                        tracing::debug!(kid, "kid still not found after JWKS refresh");
                        return Ok(None);
                    }
                }
            }
        };

        let mut validation = jsonwebtoken::Validation::new(header.alg);
        validation.set_audience(&[&self.audience]);
        validation.set_issuer(&[&self.issuer_url]);

        match jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation) {
            Ok(token_data) => {
                let claims = token_data.claims;
                let display_name = claims.name.or(claims.preferred_username);
                Ok(Some(AuthUser {
                    user_id: claims.sub.clone(),
                    external_id: Some(claims.sub),
                    email: claims.email,
                    name: display_name,
                }))
            }
            Err(e) => {
                tracing::debug!("JWT validation failed: {e}");
                Ok(None)
            }
        }
    }

    fn authorization_servers(&self) -> Vec<String> {
        vec![self.issuer_url.clone()]
    }

    fn scopes_supported(&self) -> Vec<String> {
        vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
        ]
    }
}

// ── Type-erased wrapper ──────────────────────────────────────────────────

/// Internal object-safe trait with boxed futures (same pattern as
/// `ErasedFavoriteStore` / `ErasedUserStore`).
trait ErasedAuthProvider: Send + Sync {
    fn authenticate<'a>(
        &'a self,
        token: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<Option<AuthUser>>> + Send + 'a>,
    >;

    fn authorization_servers(&self) -> Vec<String>;
    fn scopes_supported(&self) -> Vec<String>;
}

impl<T: AuthProvider> ErasedAuthProvider for T {
    fn authenticate<'a>(
        &'a self,
        token: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<Option<AuthUser>>> + Send + 'a>,
    > {
        Box::pin(AuthProvider::authenticate(self, token))
    }

    fn authorization_servers(&self) -> Vec<String> {
        AuthProvider::authorization_servers(self)
    }

    fn scopes_supported(&self) -> Vec<String> {
        AuthProvider::scopes_supported(self)
    }
}

/// Type-erased auth provider that can be shared across async tasks.
pub struct BoxedAuthProvider(Box<dyn ErasedAuthProvider>);

impl BoxedAuthProvider {
    pub fn new<T: AuthProvider + 'static>(provider: T) -> Self {
        Self(Box::new(provider))
    }

    pub async fn authenticate(&self, token: &str) -> anyhow::Result<Option<AuthUser>> {
        self.0.authenticate(token).await
    }

    pub fn authorization_servers(&self) -> Vec<String> {
        self.0.authorization_servers()
    }

    pub fn scopes_supported(&self) -> Vec<String> {
        self.0.scopes_supported()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn no_auth_provider_returns_anonymous() {
        let provider = NoAuthProvider;
        let user = AuthProvider::authenticate(&provider, "any-token")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(user.user_id, "anonymous");
        assert!(user.is_anonymous());
    }

    #[test]
    fn no_auth_provider_has_no_servers() {
        let provider = NoAuthProvider;
        assert!(AuthProvider::authorization_servers(&provider).is_empty());
        assert!(AuthProvider::scopes_supported(&provider).is_empty());
    }

    #[tokio::test]
    async fn boxed_auth_provider_delegates() {
        let boxed = BoxedAuthProvider::new(NoAuthProvider);
        let user = boxed.authenticate("anything").await.unwrap().unwrap();
        assert!(user.is_anonymous());
        assert!(boxed.authorization_servers().is_empty());
    }

    #[test]
    fn auth_user_anonymous_constructor() {
        let user = AuthUser::anonymous();
        assert_eq!(user.user_id, "anonymous");
        assert!(user.is_anonymous());
        assert!(user.external_id.is_none());
        assert!(user.email.is_none());
        assert!(user.name.is_none());
    }

    #[test]
    fn auth_user_non_anonymous() {
        let user = AuthUser {
            user_id: "abc-123".to_string(),
            external_id: Some("ext-456".to_string()),
            email: Some("user@example.com".to_string()),
            name: Some("Test User".to_string()),
        };
        assert!(!user.is_anonymous());
    }
}
