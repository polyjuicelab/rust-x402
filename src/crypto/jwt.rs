//! JWT utilities for authentication

use crate::{Result, X402Error};
use jsonwebtoken::{Algorithm, Header};

/// JWT claims for Coinbase API authentication
#[derive(Debug, serde::Serialize)]
struct Claims {
    iss: String,
    sub: String,
    aud: String,
    iat: u64,
    exp: u64,
    uri: String,
}

/// JWT options for authentication
#[derive(Debug, Clone)]
pub struct JwtOptions {
    pub key_id: String,
    pub key_secret: String,
    pub request_method: String,
    pub request_host: String,
    pub request_path: String,
}

impl JwtOptions {
    /// Create new JWT options
    pub fn new(
        key_id: impl Into<String>,
        key_secret: impl Into<String>,
        request_method: impl Into<String>,
        request_host: impl Into<String>,
        request_path: impl Into<String>,
    ) -> Self {
        Self {
            key_id: key_id.into(),
            key_secret: key_secret.into(),
            request_method: request_method.into(),
            request_host: request_host.into(),
            request_path: request_path.into(),
        }
    }
}

/// Generate JWT token for Coinbase API authentication
pub fn generate_jwt(options: JwtOptions) -> Result<String> {
    // Remove https:// if present
    let request_host = options.request_host.trim_start_matches("https://");

    let now = chrono::Utc::now().timestamp() as u64;
    let exp = now + 300; // 5 minutes

    let claims = Claims {
        iss: options.key_id.clone(),
        sub: options.key_id,
        aud: request_host.to_string(),
        iat: now,
        exp,
        uri: options.request_path,
    };

    let header = Header::new(Algorithm::HS256);
    let key = jsonwebtoken::EncodingKey::from_secret(options.key_secret.as_bytes());
    let token = jsonwebtoken::encode(&header, &claims, &key)
        .map_err(|e| X402Error::config(format!("JWT encoding failed: {}", e)))?;

    Ok(token)
}

/// Create an authorization header for Coinbase API requests
pub fn create_auth_header(
    api_key_id: &str,
    api_key_secret: &str,
    request_host: &str,
    request_path: &str,
) -> Result<String> {
    let options = JwtOptions::new(
        api_key_id,
        api_key_secret,
        "POST", // Default to POST method
        request_host,
        request_path,
    );

    let token = generate_jwt(options)?;
    Ok(format!("Bearer {}", token))
}

/// Create auth header with custom method
pub fn create_auth_header_with_method(
    api_key_id: &str,
    api_key_secret: &str,
    request_method: &str,
    request_host: &str,
    request_path: &str,
) -> Result<String> {
    let options = JwtOptions::new(
        api_key_id,
        api_key_secret,
        request_method,
        request_host,
        request_path,
    );

    let token = generate_jwt(options)?;
    Ok(format!("Bearer {}", token))
}
