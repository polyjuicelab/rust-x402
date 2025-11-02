//! Core types for the x402 protocol
//!
//! This module defines all the core data structures and types used throughout the x402
//! protocol implementation. It provides type-safe representations of payment requirements,
//! payment payloads, network configurations, and facilitator responses.
//!
//! # Architecture
//!
//! The types module is organized as follows:
//! - [`network`] - Network configuration and chain-specific details
//! - [`payment`] - Payment requirements and payload structures
//! - [`facilitator`] - Facilitator configuration and response types
//! - [`discovery`] - Discovery API types for resource discovery
//! - [`constants`] - Protocol constants (networks, schemes, addresses)
//!
//! # Examples
//!
//! ## Creating Payment Requirements
//!
//! ```
//! use rust_x402::types::{PaymentRequirements, Network};
//!
//! # fn example() -> rust_x402::Result<()> {
//! let mut requirements = PaymentRequirements::new(
//!     "exact",                                          // scheme
//!     "base-sepolia",                                   // network
//!     "1000000",                                        // amount (1 USDC)
//!     "0x036CbD53842c5426634e7929541eC2318f3dCF7e",   // USDC contract
//!     "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",   // recipient
//!     "https://api.example.com/resource",              // resource URL
//!     "API access payment",                             // description
//! );
//!
//! // Set USDC metadata
//! requirements.set_usdc_info(Network::Testnet)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Creating Payment Payload
//!
//! ```
//! use rust_x402::types::{PaymentPayload, ExactEvmPayload, ExactEvmPayloadAuthorization};
//!
//! # fn example() -> rust_x402::Result<()> {
//! let authorization = ExactEvmPayloadAuthorization::new(
//!     "0x857b06519E91e3A54538791bDbb0E22373e36b66",   // from
//!     "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",   // to
//!     "1000000",                                        // value
//!     "1745323800",                                     // validAfter
//!     "1745323985",                                     // validBefore
//!     "0xf3746613c2d920b5fdabc0856f2aeb2d4f88ee6037b8cc5d04a71a4462f13480", // nonce
//! );
//!
//! let payload = ExactEvmPayload {
//!     signature: "0x2d6a...".to_string(),
//!     authorization,
//! };
//!
//! let payment = PaymentPayload::new("exact", "base-sepolia", payload);
//!
//! // Encode to base64 for HTTP header
//! let encoded = payment.to_base64()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Network Configuration
//!
//! ```
//! use rust_x402::types::{Network, NetworkConfig, networks};
//!
//! // Using the Network enum
//! let network = Network::Testnet;
//! println!("Network: {}", network.as_str());
//! println!("USDC: {}", network.usdc_address());
//!
//! // Using NetworkConfig for detailed info
//! let config = NetworkConfig::base_sepolia();
//! println!("Chain ID: {}", config.chain_id);
//! println!("USDC Contract: {}", config.usdc_contract);
//!
//! // Using constants
//! let usdc = networks::get_usdc_address("base-sepolia");
//! println!("USDC address: {:?}", usdc);
//! ```
//!
//! ## Facilitator Configuration
//!
//! ```
//! use rust_x402::types::FacilitatorConfig;
//! use std::time::Duration;
//!
//! # fn example() -> rust_x402::Result<()> {
//! let config = FacilitatorConfig::new("https://x402.org/facilitator")
//!     .with_timeout(Duration::from_secs(30));
//!
//! // Validate the configuration
//! config.validate()?;
//! # Ok(())
//! # }
//! ```
//!
//! # Type Categories
//!
//! ## Payment Types
//! - [`PaymentRequirements`] - Describes what payment is required
//! - [`PaymentPayload`] - Contains the actual payment authorization
//! - [`ExactEvmPayload`] - EVM-specific payment data (EIP-3009)
//! - [`ExactEvmPayloadAuthorization`] - Authorization parameters
//!
//! ## Response Types
//! - [`VerifyResponse`] - Payment verification result
//! - [`SettleResponse`] - Payment settlement result
//! - [`SupportedKinds`] - Supported payment schemes and networks
//! - [`DiscoveryResponse`] - Resource discovery results
//!
//! ## Configuration Types
//! - [`FacilitatorConfig`] - Facilitator client configuration
//! - [`NetworkConfig`] - Chain-specific network configuration
//! - [`Network`] - Simple network enum (Mainnet/Testnet)

pub mod constants;
pub mod discovery;
pub mod facilitator;
pub mod network;
pub mod payment;

// Re-export commonly used types
pub use constants::{networks, schemes};
pub use discovery::{DiscoveryResource, DiscoveryResponse, PaginationInfo};
pub use facilitator::{
    AuthHeadersFn, AuthHeadersFnArc, AuthHeadersFnBox, FacilitatorConfig, SettleResponse,
    SupportedKind, SupportedKinds, VerifyResponse,
};
pub use network::{Network, NetworkConfig};
pub use payment::{
    ExactEvmPayload, ExactEvmPayloadAuthorization, PaymentPayload, PaymentRequirements,
    PaymentRequirementsResponse, X402_VERSION,
};
