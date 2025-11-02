//! Facilitator configuration and response types

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Type alias for authentication headers function
pub type AuthHeadersFn =
    dyn Fn() -> crate::Result<HashMap<String, HashMap<String, String>>> + Send + Sync;

/// Type alias for authentication headers function wrapped in Arc
pub type AuthHeadersFnArc = Arc<AuthHeadersFn>;

/// Type alias for authentication headers function wrapped in Box
pub type AuthHeadersFnBox = Box<AuthHeadersFn>;

/// Facilitator configuration
#[derive(Clone)]
pub struct FacilitatorConfig {
    /// Base URL of the facilitator service
    pub url: String,
    /// Request timeout
    pub timeout: Option<Duration>,
    /// Function to create authentication headers
    pub create_auth_headers: Option<AuthHeadersFnArc>,
}

impl std::fmt::Debug for FacilitatorConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FacilitatorConfig")
            .field("url", &self.url)
            .field("timeout", &self.timeout)
            .field("create_auth_headers", &"<function>")
            .finish()
    }
}

impl FacilitatorConfig {
    /// Create a new facilitator config
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            timeout: None,
            create_auth_headers: None,
        }
    }

    /// Validate the facilitator configuration
    pub fn validate(&self) -> crate::Result<()> {
        if self.url.is_empty() {
            return Err(crate::X402Error::config("Facilitator URL cannot be empty"));
        }

        if !self.url.starts_with("http://") && !self.url.starts_with("https://") {
            return Err(crate::X402Error::config(
                "Facilitator URL must start with http:// or https://",
            ));
        }

        Ok(())
    }

    /// Set the request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the auth headers creator
    pub fn with_auth_headers(mut self, creator: AuthHeadersFnBox) -> Self {
        self.create_auth_headers = Some(Arc::from(creator));
        self
    }
}

impl Default for FacilitatorConfig {
    fn default() -> Self {
        Self::new("https://x402.org/facilitator")
    }
}

/// Payment verification response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResponse {
    /// Whether the payment is valid
    #[serde(rename = "isValid")]
    pub is_valid: bool,
    /// Reason for invalidity (if applicable)
    #[serde(rename = "invalidReason", skip_serializing_if = "Option::is_none")]
    pub invalid_reason: Option<String>,
    /// Payer's address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payer: Option<String>,
}

/// Payment settlement response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettleResponse {
    /// Whether the settlement was successful
    pub success: bool,
    /// Error reason if settlement failed
    #[serde(rename = "errorReason", skip_serializing_if = "Option::is_none")]
    pub error_reason: Option<String>,
    /// Transaction hash or identifier
    pub transaction: String,
    /// Network where the transaction was executed
    pub network: String,
    /// Payer address if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payer: Option<String>,
}

impl SettleResponse {
    /// Encode the settle response to base64
    pub fn to_base64(&self) -> crate::Result<String> {
        use base64::{engine::general_purpose, Engine as _};
        let json = serde_json::to_string(self)?;
        Ok(general_purpose::STANDARD.encode(json))
    }
}

/// Supported payment schemes and networks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportedKinds {
    /// List of supported payment schemes and networks
    pub kinds: Vec<SupportedKind>,
}

/// Individual supported payment scheme and network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportedKind {
    /// Protocol version
    #[serde(rename = "x402Version")]
    pub x402_version: u32,
    /// Payment scheme identifier
    pub scheme: String,
    /// Blockchain network identifier
    pub network: String,
    /// Additional metadata provided by the facilitator
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}
