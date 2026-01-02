use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct OidcConfig {
    pub client_id: String,
    pub client_secret: String,
    pub issuer_url: String,
    pub redirect_uri: String,
    pub scopes: String,
}

impl OidcConfig {
    pub fn from_env() -> Self {
        Self {
            client_id: std::env::var("OIDC_CLIENT_ID")
                .unwrap_or_else(|_| "default-client-id".to_string()),
            client_secret: std::env::var("OIDC_CLIENT_SECRET")
                .unwrap_or_else(|_| "default-client-secret".to_string()),
            issuer_url: std::env::var("OIDC_ISSUER_URL")
                .unwrap_or_else(|_| "https://accounts.google.com".to_string()),
            redirect_uri: std::env::var("OIDC_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:3000/auth/oidc/callback".to_string()),
            scopes: std::env::var("OIDC_SCOPES")
                .unwrap_or_else(|_| "openid email profile".to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OidcDiscovery {
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub issuer: String,
    pub jwks_uri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub id_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub sub: String,          // OIDC subject identifier
    pub email: String,        // User's email address
    pub name: Option<String>, // Display name
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
}

pub struct OidcClient {
    config: OidcConfig,
    client: Client,
    discovery: Option<OidcDiscovery>,
}

impl OidcClient {
    pub fn new(config: OidcConfig) -> Self {
        Self {
            config,
            client: Client::new(),
            discovery: None,
        }
    }

    pub async fn discover(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let discovery_url = format!(
            "{}/.well-known/openid-configuration",
            self.config.issuer_url
        );

        log::debug!("Fetching OIDC discovery from: {}", discovery_url);

        let response = self.client.get(&discovery_url).send().await?;
        let json_body = response.text().await?;

        log::debug!("OIDC discovery response: {}", json_body);

        let discovery: OidcDiscovery = serde_json::from_str(&json_body)?;
        self.discovery = Some(discovery);
        Ok(())
    }

    pub fn generate_pkce() -> (String, String) {
        let mut random_bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut random_bytes);
        let code_verifier = URL_SAFE_NO_PAD.encode(random_bytes);

        let mut hasher = Sha256::new();
        hasher.update(code_verifier.as_bytes());
        let code_challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

        (code_verifier, code_challenge)
    }

    pub fn build_auth_url(
        &self,
        state: &str,
        code_challenge: &str,
        _redirect_url: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let discovery = self
            .discovery
            .as_ref()
            .ok_or("OIDC discovery not performed yet")?;

        let final_redirect_uri = &self.config.redirect_uri;

        let params = [
            ("response_type", "code"),
            ("client_id", &self.config.client_id),
            ("redirect_uri", final_redirect_uri),
            ("scope", &self.config.scopes),
            ("state", state),
            ("code_challenge", code_challenge),
            ("code_challenge_method", "S256"),
        ];

        let mut url = url::Url::parse(&discovery.authorization_endpoint)?;
        {
            let mut query_pairs = url.query_pairs_mut();
            for (key, value) in &params {
                query_pairs.append_pair(key, value);
            }
        }

        Ok(url.to_string())
    }

    pub async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<TokenResponse, Box<dyn std::error::Error>> {
        let discovery = self
            .discovery
            .as_ref()
            .ok_or("OIDC discovery not performed yet")?;

        let mut params = HashMap::new();
        params.insert("grant_type", "authorization_code");
        params.insert("client_id", &self.config.client_id);
        params.insert("client_secret", &self.config.client_secret);
        params.insert("code", code);
        params.insert("redirect_uri", &self.config.redirect_uri);
        params.insert("code_verifier", code_verifier);

        log::debug!("Token exchange request to: {}", discovery.token_endpoint);

        let response = self
            .client
            .post(&discovery.token_endpoint)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            log::error!("Token exchange failed: {}", error_text);
            return Err(format!("Token exchange failed: {}", error_text).into());
        }

        let token_response: TokenResponse = response.json().await?;
        Ok(token_response)
    }

    pub async fn get_user_info(
        &self,
        access_token: &str,
    ) -> Result<UserInfo, Box<dyn std::error::Error>> {
        let discovery = self
            .discovery
            .as_ref()
            .ok_or("OIDC discovery not performed yet")?;

        log::debug!("Fetching user info from: {}", discovery.userinfo_endpoint);

        let response = self
            .client
            .get(&discovery.userinfo_endpoint)
            .bearer_auth(access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            log::error!("User info request failed: {}", error_text);
            return Err(format!("User info request failed: {}", error_text).into());
        }

        let user_info: UserInfo = response.json().await?;
        Ok(user_info)
    }
}

pub type OidcClientArc = std::sync::Arc<tokio::sync::Mutex<OidcClient>>;

// Helper functions for session management
pub fn store_user_in_session(
    session: &actix_session::Session,
    user_info: &UserInfo,
) -> Result<(), actix_web::Error> {
    session.insert("oidc_user", user_info).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!(
            "Failed to store user in session: {}",
            e
        ))
    })?;
    Ok(())
}

pub fn get_user_from_session(session: &actix_session::Session) -> Option<UserInfo> {
    match session.get::<UserInfo>("oidc_user") {
        Ok(user) => user,
        Err(e) => {
            log::warn!("Failed to retrieve user from session: {}", e);
            None
        }
    }
}

pub fn clear_user_from_session(session: &actix_session::Session) {
    session.remove("oidc_user");
}

// Extract tenant information from JWT id_token
pub fn extract_tenant_claims_from_jwt(
    id_token: &str,
    tenant_claim_key: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // For now, we'll do a simple JWT payload decode without verification
    // In production, you should verify the JWT signature

    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid JWT format".into());
    }

    // Decode the payload (second part)
    let payload = parts[1];

    // Add padding if needed for base64 decoding
    let padded_payload = match payload.len() % 4 {
        0 => payload.to_string(),
        2 => format!("{}==", payload),
        3 => format!("{}=", payload),
        _ => return Err("Invalid base64 padding".into()),
    };

    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(padded_payload)?;

    let claims: serde_json::Value = serde_json::from_slice(&decoded)?;

    // Extract tenant information from the configured claim
    let tenants = match claims.get(tenant_claim_key) {
        Some(tenant_value) => match tenant_value {
            serde_json::Value::String(single_tenant) => {
                vec![single_tenant.clone()]
            }
            serde_json::Value::Array(tenant_array) => tenant_array
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect(),
            _ => {
                log::warn!(
                    "Tenant claim '{}' has unexpected format: {:?}",
                    tenant_claim_key,
                    tenant_value
                );
                vec![]
            }
        },
        None => {
            log::info!("No tenant claim '{}' found in JWT", tenant_claim_key);
            vec![]
        }
    };

    log::debug!("Extracted tenants from JWT: {:?}", tenants);
    Ok(tenants)
}

// Store tenant information in session
pub fn store_tenant_access_in_session(
    session: &actix_session::Session,
    tenants: &[String],
) -> Result<(), actix_web::Error> {
    session.insert("allowed_tenants", tenants).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!(
            "Failed to store tenant access in session: {}",
            e
        ))
    })?;
    Ok(())
}
