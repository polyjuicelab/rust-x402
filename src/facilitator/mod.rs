//! Facilitator client for payment verification and settlement
//!
//! This module provides a client for interacting with x402 facilitator services.
//! A facilitator is responsible for verifying payment authorizations and settling
//! transactions on the blockchain.
//!
//! # Architecture
//!
//! The facilitator module is organized as follows:
//! - [`FacilitatorClient`] - Main client for facilitator API interactions
//! - [`coinbase`] - Coinbase-specific facilitator integration
//! - Tests - Comprehensive test suite for all functionality
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```no_run
//! use rust_x402::facilitator::FacilitatorClient;
//! use rust_x402::types::{FacilitatorConfig, PaymentPayload, PaymentRequirements};
//!
//! # async fn example() -> rust_x402::Result<()> {
//! // Create a facilitator client
//! let config = FacilitatorConfig::new("https://x402.org/facilitator");
//! let client = FacilitatorClient::new(config)?;
//!
//! // Verify a payment
//! # let payment_payload = todo!();
//! # let payment_requirements = todo!();
//! let verify_response = client.verify(&payment_payload, &payment_requirements).await?;
//!
//! if verify_response.is_valid {
//!     // Settle the payment
//!     let settle_response = client.settle(&payment_payload, &payment_requirements).await?;
//!     println!("Payment settled: {}", settle_response.transaction);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Using Coinbase Facilitator
//!
//! ```no_run
//! use rust_x402::facilitator::FacilitatorClient;
//! use rust_x402::facilitator::coinbase;
//!
//! # async fn example() -> rust_x402::Result<()> {
//! // Create a Coinbase facilitator config
//! let config = coinbase::create_facilitator_config("api_key_id", "api_key_secret");
//! let client = FacilitatorClient::new(config)?;
//!
//! // Use the client normally
//! let supported = client.supported().await?;
//! println!("Supported schemes: {:?}", supported.kinds);
//! # Ok(())
//! # }
//! ```
//!
//! ## Discovery API
//!
//! ```no_run
//! use rust_x402::facilitator::FacilitatorClient;
//! use rust_x402::client::DiscoveryFilters;
//! use rust_x402::types::FacilitatorConfig;
//!
//! # async fn example() -> rust_x402::Result<()> {
//! let config = FacilitatorConfig::new("https://x402.org/facilitator");
//! let client = FacilitatorClient::new(config)?;
//!
//! // List all resources
//! let resources = client.list_all().await?;
//!
//! // List with filters
//! let filters = DiscoveryFilters::new()
//!     .with_resource_type("http")
//!     .with_limit(10);
//! let filtered = client.list(Some(filters)).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! - **Payment Verification** - Verify payment authorizations without executing transactions
//! - **Payment Settlement** - Execute verified payments on the blockchain
//! - **Network Support** - Query supported payment schemes and networks
//! - **Discovery API** - Discover x402-enabled resources
//! - **Coinbase Integration** - Built-in support for Coinbase CDP facilitator
//! - **Authentication** - Flexible authentication header configuration

use crate::client::DiscoveryFilters;
use crate::types::{
    DiscoveryResponse, FacilitatorConfig, PaymentPayload, PaymentRequirements, SettleResponse,
    SupportedKinds, VerifyResponse,
};
use crate::{Result, X402Error};
use reqwest::Client;
use serde_json::json;

pub mod coinbase;

#[cfg(test)]
mod tests;

/// Default facilitator URL
pub const DEFAULT_FACILITATOR_URL: &str = "https://x402.org/facilitator";

/// Facilitator client for verifying and settling payments
#[derive(Clone)]
pub struct FacilitatorClient {
    /// Base URL of the facilitator service
    url: String,
    /// HTTP client
    client: Client,
    /// Configuration for authentication headers
    auth_config: Option<crate::types::AuthHeadersFnArc>,
}

impl std::fmt::Debug for FacilitatorClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FacilitatorClient")
            .field("url", &self.url)
            .field("auth_config", &"<function>")
            .finish()
    }
}

impl FacilitatorClient {
    /// Create a new facilitator client
    pub fn new(config: FacilitatorConfig) -> Result<Self> {
        // Validate configuration first
        config.validate()?;

        let mut client_builder = Client::builder();

        if let Some(timeout) = config.timeout {
            client_builder = client_builder.timeout(timeout);
        }

        let client = client_builder
            .build()
            .map_err(|e| X402Error::config(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            url: config.url,
            client,
            auth_config: config.create_auth_headers,
        })
    }

    /// Verify a payment without executing the transaction
    pub async fn verify(
        &self,
        payment_payload: &PaymentPayload,
        payment_requirements: &PaymentRequirements,
    ) -> Result<VerifyResponse> {
        tracing::debug!(
            "Payment payload: {}",
            serde_json::to_string_pretty(payment_payload).unwrap_or_default()
        );
        tracing::debug!(
            "Payment requirements: {}",
            serde_json::to_string_pretty(payment_requirements).unwrap_or_default()
        );

        let request_body = json!({
            "paymentPayload": payment_payload,
            "paymentRequirements": payment_requirements,
        });

        tracing::debug!(
            "Facilitator verify request body: {}",
            serde_json::to_string_pretty(&request_body).unwrap_or_default()
        );
        tracing::debug!("Sending request to: {}/verify", self.url);

        let mut request = self
            .client
            .post(format!("{}/verify", self.url))
            .json(&request_body);

        // Add authentication headers if available
        if let Some(auth_config) = &self.auth_config {
            let headers = auth_config()?;
            if let Some(verify_headers) = headers.get("verify") {
                for (key, value) in verify_headers {
                    request = request.header(key, value);
                }
            }
        }

        let response = request.send().await?;
        let status = response.status();

        if !status.is_success() {
            let response_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read response body".to_string());
            tracing::error!(
                "Facilitator verify failed with status: {}. Request body: {}. Response body: {}",
                status,
                serde_json::to_string_pretty(&request_body).unwrap_or_default(),
                response_body
            );
            return Err(X402Error::facilitator_error(format!(
                "Verification failed with status: {}. Response: {}. Request: {}",
                status,
                response_body,
                serde_json::to_string(&request_body).unwrap_or_default()
            )));
        }

        let verify_response: VerifyResponse = response.json().await?;
        Ok(verify_response)
    }

    /// Settle a verified payment by executing the transaction
    pub async fn settle(
        &self,
        payment_payload: &PaymentPayload,
        payment_requirements: &PaymentRequirements,
    ) -> Result<SettleResponse> {
        let request_body = json!({
            "paymentPayload": payment_payload,
            "paymentRequirements": payment_requirements,
        });

        let mut request = self
            .client
            .post(format!("{}/settle", self.url))
            .json(&request_body);

        // Add authentication headers if available
        if let Some(auth_config) = &self.auth_config {
            let headers = auth_config()?;
            if let Some(settle_headers) = headers.get("settle") {
                for (key, value) in settle_headers {
                    request = request.header(key, value);
                }
            }
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(X402Error::facilitator_error(format!(
                "Settlement failed with status: {}",
                response.status()
            )));
        }

        let settle_response: SettleResponse = response.json().await?;
        Ok(settle_response)
    }

    /// Get supported payment schemes and networks
    pub async fn supported(&self) -> Result<SupportedKinds> {
        let mut request = self.client.get(format!("{}/supported", self.url));

        // Add authentication headers if available
        if let Some(auth_config) = &self.auth_config {
            let headers = auth_config()?;
            if let Some(supported_headers) = headers.get("supported") {
                for (key, value) in supported_headers {
                    request = request.header(key, value);
                }
            }
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(X402Error::facilitator_error(format!(
                "Failed to get supported kinds with status: {}",
                response.status()
            )));
        }

        let supported: SupportedKinds = response.json().await?;
        Ok(supported)
    }

    /// Get the base URL of this facilitator
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Create a facilitator client for a specific network
    pub fn for_network(_network: &str, config: FacilitatorConfig) -> Result<Self> {
        // For now, use the provided config as-is
        // In the future, this could customize the config based on network
        Self::new(config)
    }

    /// Create a facilitator client for Base mainnet
    pub fn for_base_mainnet(config: FacilitatorConfig) -> Result<Self> {
        Self::for_network("base", config)
    }

    /// Create a facilitator client for Base Sepolia testnet
    pub fn for_base_sepolia(config: FacilitatorConfig) -> Result<Self> {
        Self::for_network("base-sepolia", config)
    }

    /// Verify payment with network-specific validation
    pub async fn verify_with_network_validation(
        &self,
        payment_payload: &PaymentPayload,
        payment_requirements: &PaymentRequirements,
    ) -> Result<VerifyResponse> {
        // Validate that the payment network matches requirements - return error on mismatch
        if payment_payload.network != payment_requirements.network {
            return Err(X402Error::payment_verification_failed(format!(
                "CRITICAL ERROR: Network mismatch detected! Payment network '{}' does not match requirements network '{}'. This is a security violation.",
                payment_payload.network, payment_requirements.network
            )));
        }

        // Validate that the payment scheme matches requirements
        if payment_payload.scheme != payment_requirements.scheme {
            return Err(X402Error::payment_verification_failed(format!(
                "Scheme mismatch: payment scheme {} != requirements scheme {}",
                payment_payload.scheme, payment_requirements.scheme
            )));
        }

        // Proceed with normal verification
        self.verify(payment_payload, payment_requirements).await
    }

    /// List discovery resources from the facilitator service
    ///
    /// This method hits the `/discovery/resources` endpoint and forwards any auth headers,
    /// similar to TypeScript's `useFacilitator().list()` and Python's `FacilitatorClient.list()`
    pub async fn list(&self, filters: Option<DiscoveryFilters>) -> Result<DiscoveryResponse> {
        let mut request = self.client.get(format!("{}/discovery/resources", self.url));

        // Add query parameters if filters are provided
        if let Some(filters) = filters {
            if let Some(resource_type) = filters.resource_type {
                request = request.query(&[("type", resource_type)]);
            }
            if let Some(limit) = filters.limit {
                request = request.query(&[("limit", limit.to_string())]);
            }
            if let Some(offset) = filters.offset {
                request = request.query(&[("offset", offset.to_string())]);
            }
        }

        // Add authentication headers if available
        if let Some(auth_config) = &self.auth_config {
            let headers = auth_config()?;
            if let Some(discovery_headers) = headers.get("list") {
                for (key, value) in discovery_headers {
                    request = request.header(key, value);
                }
            }
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(X402Error::facilitator_error(format!(
                "Discovery failed with status: {}",
                response.status()
            )));
        }

        let discovery_response: DiscoveryResponse = response.json().await?;
        Ok(discovery_response)
    }

    /// Get all discovery resources without filters
    pub async fn list_all(&self) -> Result<DiscoveryResponse> {
        self.list(None).await
    }

    /// Get discovery resources by type
    pub async fn list_by_type(&self, resource_type: &str) -> Result<DiscoveryResponse> {
        let filters = DiscoveryFilters::new().with_resource_type(resource_type);
        self.list(Some(filters)).await
    }
}

impl Default for FacilitatorClient {
    fn default() -> Self {
        Self::new(FacilitatorConfig::default()).unwrap_or_else(|_| {
            // Fallback to basic client if configuration fails
            Self {
                url: "https://x402.org/facilitator".to_string(),
                client: Client::new(),
                auth_config: None,
            }
        })
    }
}
