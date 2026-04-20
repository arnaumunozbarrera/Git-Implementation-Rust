use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::http::{header::AUTHORIZATION, HeaderMap, Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::api::api::AppState;
use crate::api::clients::supabase::SupabaseClient;

#[derive(Clone)]
pub struct AuthConfig {
    issuer: String,
    jwks_url: String,
    audience: Option<String>,
    keys: Arc<RwLock<HashMap<String, DecodingKey>>>,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub email: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct JwtClaims {
    sub: String,
    iss: String,
    exp: usize,
    #[serde(default)]
    aud: Option<JwtAudience>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    username: Option<String>,
    #[serde(default)]
    preferred_username: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
enum JwtAudience {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Deserialize)]
struct JwksDocument {
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize)]
struct Jwk {
    kid: String,
    n: String,
    e: String,
    #[serde(default)]
    alg: Option<String>,
}

impl AuthConfig {
    pub async fn from_env() -> Result<Option<Self>, String> {
        let issuer = match env::var("CLERK_JWT_ISSUER") {
            Ok(value) if !value.trim().is_empty() => value.trim().to_string(),
            _ => return Ok(None),
        };

        let jwks_url = env::var("CLERK_JWKS_URL")
            .map_err(|_| "[ERROR] Missing CLERK_JWKS_URL".to_string())?
            .trim()
            .to_string();
        let audience = env::var("CLERK_JWT_AUDIENCE")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        let keys = load_jwks(&jwks_url).await?;
        Ok(Some(Self {
            issuer,
            jwks_url,
            audience,
            keys: Arc::new(RwLock::new(keys)),
        }))
    }

    pub async fn authenticate(&self, headers: &HeaderMap) -> Result<AuthenticatedUser, String> {
        let token = extract_bearer_token(headers)?;
        let header = decode_header(&token)
            .map_err(|error| format!("[ERROR] Invalid JWT header: {}", error))?;
        let kid = header.kid.ok_or_else(|| "[ERROR] JWT missing kid".to_string())?;

        let mut decoding_key = {
            let keys = self.keys.read().await;
            keys.get(&kid).cloned()
        };

        if decoding_key.is_none() {
            let refreshed = load_jwks(&self.jwks_url).await?;
            let mut keys = self.keys.write().await;
            *keys = refreshed;
            decoding_key = keys.get(&kid).cloned();
        }

        let decoding_key = decoding_key.ok_or_else(|| "[ERROR] Unknown JWT key id".to_string())?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[self.issuer.as_str()]);
        if let Some(audience) = &self.audience {
            validation.set_audience(&[audience.as_str()]);
        }

        let data = decode::<JwtClaims>(&token, &decoding_key, &validation)
            .map_err(|error| format!("[ERROR] Invalid JWT: {}", error))?;

        if data.claims.iss != self.issuer {
            return Err("[ERROR] JWT issuer mismatch".to_string());
        }

        if let Some(expected_audience) = &self.audience {
            if !audience_matches(data.claims.aud.as_ref(), expected_audience) {
                return Err("[ERROR] JWT audience mismatch".to_string());
            }
        }

        let username = data
            .claims
            .username
            .clone()
            .or(data.claims.preferred_username.clone());

        Ok(AuthenticatedUser {
            user_id: data.claims.sub,
            email: data.claims.email,
            username,
        })
    }
}

pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let Some(auth) = state.auth.as_ref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "[ERROR] Auth not configured".to_string(),
        ));
    };

    let user = auth
        .authenticate(request.headers())
        .await
        .map_err(|message| (StatusCode::UNAUTHORIZED, message))?;

    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

pub async fn ensure_user_exists(
    client: Option<&SupabaseClient>,
    user: &AuthenticatedUser,
) -> Result<(), String> {
    let Some(client) = client else {
        return Ok(());
    };

    sqlx::query(
        "INSERT INTO users (id, username, email) VALUES ($1, $2, $3)
         ON CONFLICT (id) DO UPDATE SET
           username = COALESCE(EXCLUDED.username, users.username),
           email = COALESCE(EXCLUDED.email, users.email)",
    )
    .bind(&user.user_id)
    .bind(user.username.as_deref())
    .bind(user.email.as_deref())
    .execute(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to upsert authenticated user '{}': {}", user.user_id, error))?;

    Ok(())
}

fn extract_bearer_token(headers: &HeaderMap) -> Result<String, String> {
    let value = headers
        .get(AUTHORIZATION)
        .ok_or_else(|| "[ERROR] Missing Authorization header".to_string())?
        .to_str()
        .map_err(|_| "[ERROR] Invalid Authorization header".to_string())?;

    value
        .strip_prefix("Bearer ")
        .map(str::trim)
        .map(str::to_string)
        .filter(|token| !token.is_empty())
        .ok_or_else(|| "[ERROR] Authorization header must use Bearer token".to_string())
}

fn audience_matches(audience: Option<&JwtAudience>, expected: &str) -> bool {
    match audience {
        Some(JwtAudience::Single(value)) => value == expected,
        Some(JwtAudience::Multiple(values)) => values.iter().any(|value| value == expected),
        None => false,
    }
}

async fn load_jwks(jwks_url: &str) -> Result<HashMap<String, DecodingKey>, String> {
    let response = reqwest::get(jwks_url)
        .await
        .map_err(|error| format!("[ERROR] Failed to download JWKS: {}", error))?;
    let jwks = response
        .json::<JwksDocument>()
        .await
        .map_err(|error| format!("[ERROR] Failed to parse JWKS: {}", error))?;

    let mut keys = HashMap::new();
    for key in jwks.keys {
        if let Some(alg) = &key.alg {
            if alg != "RS256" {
                continue;
            }
        }

        let decoding_key = DecodingKey::from_rsa_components(&key.n, &key.e)
            .map_err(|error| format!("[ERROR] Failed to build decoding key: {}", error))?;
        keys.insert(key.kid, decoding_key);
    }

    if keys.is_empty() {
        return Err("[ERROR] JWKS did not contain any usable RSA keys".to_string());
    }

    Ok(keys)
}
