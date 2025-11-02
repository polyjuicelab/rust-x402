//! Payment-related types

use super::network::Network;
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

/// x402 protocol version
pub const X402_VERSION: u32 = 1;

/// Payment requirements for a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequirements {
    /// Payment scheme identifier (e.g., "exact")
    pub scheme: String,
    /// Blockchain network identifier (e.g., "base-sepolia", "ethereum-mainnet")
    pub network: String,
    /// Required payment amount in atomic token units
    #[serde(rename = "maxAmountRequired")]
    pub max_amount_required: String,
    /// Token contract address
    pub asset: String,
    /// Recipient wallet address for the payment
    #[serde(rename = "payTo")]
    pub pay_to: String,
    /// URL of the protected resource
    pub resource: String,
    /// Human-readable description of the resource
    pub description: String,
    /// MIME type of the expected response
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// JSON schema describing the response format
    #[serde(rename = "outputSchema", skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,
    /// Maximum time allowed for payment completion in seconds
    #[serde(rename = "maxTimeoutSeconds")]
    pub max_timeout_seconds: u32,
    /// Scheme-specific additional information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Value>,
}

impl PaymentRequirements {
    /// Create a new payment requirements instance
    pub fn new(
        scheme: impl Into<String>,
        network: impl Into<String>,
        max_amount_required: impl Into<String>,
        asset: impl Into<String>,
        pay_to: impl Into<String>,
        resource: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            scheme: scheme.into(),
            network: network.into(),
            max_amount_required: max_amount_required.into(),
            asset: asset.into(),
            pay_to: pay_to.into(),
            resource: resource.into(),
            description: description.into(),
            mime_type: None,
            output_schema: None,
            max_timeout_seconds: 60,
            extra: None,
        }
    }

    /// Set USDC token information in the extra field
    pub fn set_usdc_info(&mut self, network: Network) -> crate::Result<()> {
        let mut usdc_info = HashMap::new();
        usdc_info.insert("name".to_string(), network.usdc_name().to_string());
        usdc_info.insert("version".to_string(), "2".to_string());

        self.extra = Some(serde_json::to_value(usdc_info)?);
        Ok(())
    }

    /// Get the amount as a decimal
    pub fn amount_as_decimal(&self) -> crate::Result<Decimal> {
        self.max_amount_required
            .parse()
            .map_err(|_| crate::X402Error::invalid_payment_requirements("Invalid amount format"))
    }

    /// Get the amount in decimal units (e.g., 0.01 for 1 cent)
    pub fn amount_in_decimal_units(&self, decimals: u8) -> crate::Result<Decimal> {
        let amount = self.amount_as_decimal()?;
        let divisor = Decimal::from(10u64.pow(decimals as u32));
        Ok(amount / divisor)
    }
}

/// Payment payload for client payment authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentPayload {
    /// Protocol version identifier
    #[serde(rename = "x402Version")]
    pub x402_version: u32,
    /// Payment scheme identifier
    pub scheme: String,
    /// Blockchain network identifier
    pub network: String,
    /// Payment data object
    pub payload: ExactEvmPayload,
}

impl PaymentPayload {
    /// Create a new payment payload
    pub fn new(
        scheme: impl Into<String>,
        network: impl Into<String>,
        payload: ExactEvmPayload,
    ) -> Self {
        Self {
            x402_version: X402_VERSION,
            scheme: scheme.into(),
            network: network.into(),
            payload,
        }
    }

    /// Decode a base64-encoded payment payload
    pub fn from_base64(encoded: &str) -> crate::Result<Self> {
        use base64::{engine::general_purpose, Engine as _};
        let decoded = general_purpose::STANDARD.decode(encoded)?;
        let payload: PaymentPayload = serde_json::from_slice(&decoded)?;
        Ok(payload)
    }

    /// Encode the payment payload to base64
    pub fn to_base64(&self) -> crate::Result<String> {
        use base64::{engine::general_purpose, Engine as _};
        let json = serde_json::to_string(self)?;
        Ok(general_purpose::STANDARD.encode(json))
    }
}

/// Exact EVM payment payload (EIP-3009)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExactEvmPayload {
    /// EIP-712 signature for authorization
    pub signature: String,
    /// EIP-3009 authorization parameters
    pub authorization: ExactEvmPayloadAuthorization,
}

/// EIP-3009 authorization parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExactEvmPayloadAuthorization {
    /// Payer's wallet address
    pub from: String,
    /// Recipient's wallet address
    pub to: String,
    /// Payment amount in atomic units
    pub value: String,
    /// Unix timestamp when authorization becomes valid
    #[serde(rename = "validAfter")]
    pub valid_after: String,
    /// Unix timestamp when authorization expires
    #[serde(rename = "validBefore")]
    pub valid_before: String,
    /// 32-byte random nonce to prevent replay attacks
    pub nonce: String,
}

impl ExactEvmPayloadAuthorization {
    /// Create a new authorization
    pub fn new(
        from: impl Into<String>,
        to: impl Into<String>,
        value: impl Into<String>,
        valid_after: impl Into<String>,
        valid_before: impl Into<String>,
        nonce: impl Into<String>,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            value: value.into(),
            valid_after: valid_after.into(),
            valid_before: valid_before.into(),
            nonce: nonce.into(),
        }
    }

    /// Check if the authorization is currently valid
    pub fn is_valid_now(&self) -> crate::Result<bool> {
        let now = Utc::now().timestamp();
        let valid_after: i64 = self.valid_after.parse().map_err(|_| {
            crate::X402Error::invalid_authorization("Invalid valid_after timestamp")
        })?;
        let valid_before: i64 = self.valid_before.parse().map_err(|_| {
            crate::X402Error::invalid_authorization("Invalid valid_before timestamp")
        })?;

        Ok(now >= valid_after && now <= valid_before)
    }

    /// Get the validity duration
    pub fn validity_duration(&self) -> crate::Result<Duration> {
        let valid_after: i64 = self.valid_after.parse().map_err(|_| {
            crate::X402Error::invalid_authorization("Invalid valid_after timestamp")
        })?;
        let valid_before: i64 = self.valid_before.parse().map_err(|_| {
            crate::X402Error::invalid_authorization("Invalid valid_before timestamp")
        })?;

        Ok(Duration::from_secs((valid_before - valid_after) as u64))
    }
}

/// Payment requirements response (HTTP 402 response)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequirementsResponse {
    /// Protocol version
    #[serde(rename = "x402Version")]
    pub x402_version: u32,
    /// Human-readable error message
    pub error: String,
    /// Array of acceptable payment methods
    pub accepts: Vec<PaymentRequirements>,
}

impl PaymentRequirementsResponse {
    /// Create a new payment requirements response
    pub fn new(error: impl Into<String>, accepts: Vec<PaymentRequirements>) -> Self {
        Self {
            x402_version: X402_VERSION,
            error: error.into(),
            accepts,
        }
    }
}
