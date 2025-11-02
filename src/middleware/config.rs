//! Middleware configuration

use crate::types::{FacilitatorConfig, Network, PaymentRequirements};
use crate::{Result, X402Error};
use rust_decimal::Decimal;

/// Configuration for payment middleware
#[derive(Debug, Clone)]
pub struct PaymentMiddlewareConfig {
    /// Payment amount in decimal units (e.g., 0.0001 for 1/10th of a cent)
    pub amount: Decimal,
    /// Recipient wallet address
    pub pay_to: String,
    /// Payment description
    pub description: Option<String>,
    /// MIME type of the expected response
    pub mime_type: Option<String>,
    /// Maximum timeout in seconds
    pub max_timeout_seconds: u32,
    /// JSON schema for response format
    pub output_schema: Option<serde_json::Value>,
    /// Facilitator configuration
    pub facilitator_config: FacilitatorConfig,
    /// Whether this is a testnet
    pub testnet: bool,
    /// Custom paywall HTML for web browsers
    pub custom_paywall_html: Option<String>,
    /// Resource URL (if different from request URL)
    pub resource: Option<String>,
    /// Resource root URL for constructing full resource URLs
    pub resource_root_url: Option<String>,
}

impl PaymentMiddlewareConfig {
    /// Create a new payment middleware config
    pub fn new(amount: Decimal, pay_to: impl Into<String>) -> Self {
        // Normalize pay_to to lowercase to avoid EIP-55 checksum mismatches
        let pay_to_normalized = pay_to.into().to_lowercase();
        Self {
            amount,
            pay_to: pay_to_normalized,
            description: None,
            mime_type: None,
            max_timeout_seconds: 60,
            output_schema: None,
            facilitator_config: FacilitatorConfig::default(),
            testnet: true,
            custom_paywall_html: None,
            resource: None,
            resource_root_url: None,
        }
    }

    /// Set the payment description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the MIME type
    pub fn with_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    /// Set the maximum timeout
    pub fn with_max_timeout_seconds(mut self, max_timeout_seconds: u32) -> Self {
        self.max_timeout_seconds = max_timeout_seconds;
        self
    }

    /// Set the output schema
    pub fn with_output_schema(mut self, output_schema: serde_json::Value) -> Self {
        self.output_schema = Some(output_schema);
        self
    }

    /// Set the facilitator configuration
    pub fn with_facilitator_config(mut self, facilitator_config: FacilitatorConfig) -> Self {
        self.facilitator_config = facilitator_config;
        self
    }

    /// Set whether this is a testnet
    pub fn with_testnet(mut self, testnet: bool) -> Self {
        self.testnet = testnet;
        self
    }

    /// Set custom paywall HTML
    pub fn with_custom_paywall_html(mut self, html: impl Into<String>) -> Self {
        self.custom_paywall_html = Some(html.into());
        self
    }

    /// Set the resource URL
    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    /// Set the resource root URL
    pub fn with_resource_root_url(mut self, url: impl Into<String>) -> Self {
        self.resource_root_url = Some(url.into());
        self
    }

    /// Create payment requirements from this config
    pub fn create_payment_requirements(&self, request_uri: &str) -> Result<PaymentRequirements> {
        let network = if self.testnet {
            crate::types::networks::BASE_SEPOLIA
        } else {
            crate::types::networks::BASE_MAINNET
        };

        let usdc_address = crate::types::networks::get_usdc_address(network).ok_or_else(|| {
            X402Error::NetworkNotSupported {
                network: network.to_string(),
            }
        })?;

        let resource = if let Some(ref resource_url) = self.resource {
            resource_url.clone()
        } else if let Some(ref root_url) = self.resource_root_url {
            format!("{}{}", root_url, request_uri)
        } else {
            request_uri.to_string()
        };

        let max_amount_required = (self.amount * Decimal::from(1_000_000u64))
            .normalize()
            .to_string();

        // Normalize pay_to to lowercase to avoid EIP-55 checksum mismatches
        let pay_to_normalized = self.pay_to.to_lowercase();

        let mut requirements = PaymentRequirements::new(
            crate::types::schemes::EXACT,
            network,
            max_amount_required,
            usdc_address,
            &pay_to_normalized,
            resource,
            self.description.as_deref().unwrap_or("Payment required"),
        );

        requirements.mime_type = self.mime_type.clone();
        requirements.output_schema = self.output_schema.clone();
        requirements.max_timeout_seconds = self.max_timeout_seconds;

        let network = if self.testnet {
            Network::Testnet
        } else {
            Network::Mainnet
        };
        requirements.set_usdc_info(network)?;

        Ok(requirements)
    }
}
