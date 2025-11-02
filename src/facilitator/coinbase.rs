//! Coinbase facilitator integration

use crate::crypto::jwt;
use crate::types::FacilitatorConfig;
use crate::{Result, X402Error};
use std::collections::HashMap;
use std::env;

/// Coinbase facilitator base URL
pub const COINBASE_FACILITATOR_BASE_URL: &str = "https://api.cdp.coinbase.com";
/// Coinbase facilitator v2 route
pub const COINBASE_FACILITATOR_V2_ROUTE: &str = "/platform/v2/x402";
/// SDK version
pub const SDK_VERSION: &str = "0.1.0";

/// Create authentication headers for Coinbase facilitator
pub fn create_auth_headers(
    api_key_id: &str,
    api_key_secret: &str,
) -> impl Fn() -> Result<HashMap<String, HashMap<String, String>>> + Send + Sync {
    let api_key_id = api_key_id.to_string();
    let api_key_secret = api_key_secret.to_string();

    move || {
        // Use provided credentials or fall back to environment variables
        let id = if api_key_id.is_empty() {
            env::var("CDP_API_KEY_ID").unwrap_or_default()
        } else {
            api_key_id.clone()
        };

        let secret = if api_key_secret.is_empty() {
            env::var("CDP_API_KEY_SECRET").unwrap_or_default()
        } else {
            api_key_secret.clone()
        };

        if id.is_empty() || secret.is_empty() {
            return Err(X402Error::config(
                "Missing credentials: CDP_API_KEY_ID and CDP_API_KEY_SECRET must be set",
            ));
        }

        let verify_token = jwt::create_auth_header_with_method(
            &id,
            &secret,
            "POST",
            COINBASE_FACILITATOR_BASE_URL,
            &format!("{}/verify", COINBASE_FACILITATOR_V2_ROUTE),
        )?;

        let settle_token = jwt::create_auth_header_with_method(
            &id,
            &secret,
            "POST",
            COINBASE_FACILITATOR_BASE_URL,
            &format!("{}/settle", COINBASE_FACILITATOR_V2_ROUTE),
        )?;

        let correlation_header = create_correlation_header();

        let mut headers = HashMap::new();

        let mut verify_headers = HashMap::new();
        verify_headers.insert("Authorization".to_string(), verify_token);
        verify_headers.insert(
            "Correlation-Context".to_string(),
            correlation_header.clone(),
        );
        headers.insert("verify".to_string(), verify_headers);

        let mut settle_headers = HashMap::new();
        settle_headers.insert("Authorization".to_string(), settle_token);
        settle_headers.insert("Correlation-Context".to_string(), correlation_header);
        headers.insert("settle".to_string(), settle_headers);

        Ok(headers)
    }
}

/// Create a facilitator config for Coinbase
pub fn create_facilitator_config(api_key_id: &str, api_key_secret: &str) -> FacilitatorConfig {
    FacilitatorConfig::new(format!(
        "{}{}",
        COINBASE_FACILITATOR_BASE_URL, COINBASE_FACILITATOR_V2_ROUTE
    ))
    .with_auth_headers(Box::new(create_auth_headers(api_key_id, api_key_secret)))
}

/// Create correlation header for requests
fn create_correlation_header() -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

    let data = [
        ("sdk_version", SDK_VERSION),
        ("sdk_language", "rust"),
        ("source", "x402"),
        ("source_version", crate::VERSION),
    ];

    let pairs: Vec<String> = data
        .iter()
        .map(|(key, value)| format!("{}={}", key, utf8_percent_encode(value, NON_ALPHANUMERIC)))
        .collect();

    pairs.join(",")
}

/// Create a default Coinbase facilitator config
pub fn default_coinbase_config() -> FacilitatorConfig {
    create_facilitator_config("", "")
}

/// Create a Coinbase facilitator config with explicit credentials
pub fn coinbase_config_with_credentials(
    api_key_id: impl Into<String>,
    api_key_secret: impl Into<String>,
) -> FacilitatorConfig {
    let id = api_key_id.into();
    let secret = api_key_secret.into();
    create_facilitator_config(&id, &secret)
}

/// Create a Coinbase facilitator config from environment variables
pub fn coinbase_config_from_env() -> FacilitatorConfig {
    let api_key_id = env::var("CDP_API_KEY_ID").unwrap_or_default();
    let api_key_secret = env::var("CDP_API_KEY_SECRET").unwrap_or_default();

    create_facilitator_config(&api_key_id, &api_key_secret)
}
