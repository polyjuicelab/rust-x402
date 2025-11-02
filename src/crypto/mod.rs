//! Cryptographic utilities for x402 payments
//!
//! This module provides cryptographic primitives for the x402 protocol, including
//! JWT authentication, EIP-712 typed data hashing, and ECDSA signature verification.
//!
//! # Architecture
//!
//! The crypto module is organized as follows:
//! - [`jwt`] - JWT token generation for API authentication (primarily Coinbase CDP)
//! - [`eip712`] - EIP-712 typed data hashing for Ethereum transactions
//! - [`signature`] - ECDSA signature creation and verification
//!
//! # Examples
//!
//! ## JWT Authentication
//!
//! ```no_run
//! use rust_x402::crypto::jwt;
//!
//! # fn example() -> rust_x402::Result<()> {
//! // Create an authorization header for Coinbase API
//! let auth_header = jwt::create_auth_header(
//!     "api_key_id",
//!     "api_key_secret",
//!     "api.cdp.coinbase.com",
//!     "/platform/v2/x402/verify"
//! )?;
//!
//! // Use the auth_header in HTTP requests
//! println!("Authorization: {}", auth_header);
//! # Ok(())
//! # }
//! ```
//!
//! ## EIP-712 Typed Data Hashing
//!
//! ```no_run
//! use rust_x402::crypto::eip712::{Domain, create_transfer_with_authorization_hash};
//! use ethereum_types::{Address, H256, U256};
//! use std::str::FromStr;
//!
//! # fn example() -> rust_x402::Result<()> {
//! let domain = Domain {
//!     name: "USD Coin".to_string(),
//!     version: "2".to_string(),
//!     chain_id: 8453,
//!     verifying_contract: Address::from_str("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913")?,
//! };
//!
//! let message_hash = create_transfer_with_authorization_hash(
//!     &domain,
//!     Address::from_str("0x...")?,
//!     Address::from_str("0x...")?,
//!     U256::from(1000000),
//!     U256::from(0),
//!     U256::from(u64::MAX),
//!     H256::random(),
//! )?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Signature Verification
//!
//! ```no_run
//! use rust_x402::crypto::signature;
//! use rust_x402::types::ExactEvmPayload;
//!
//! # fn example() -> rust_x402::Result<()> {
//! # let payload: ExactEvmPayload = todo!();
//! // Verify a payment payload signature
//! let is_valid = signature::verify_payment_payload(
//!     &payload,
//!     "0x857b06519E91e3A54538791bDbb0E22373e36b66",
//!     "base-sepolia"
//! )?;
//!
//! if is_valid {
//!     println!("Signature is valid!");
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Generating Nonces
//!
//! ```
//! use rust_x402::crypto::signature::generate_nonce;
//!
//! // Generate a random nonce for EIP-3009 authorization
//! let nonce = generate_nonce();
//! println!("Nonce: {:?}", nonce);
//! ```
//!
//! # Features
//!
//! - **JWT Authentication** - Generate JWT tokens for API authentication
//! - **EIP-712 Hashing** - Hash typed data according to EIP-712 specification
//! - **Signature Verification** - Verify ECDSA signatures with public key recovery
//! - **Nonce Generation** - Generate cryptographically secure random nonces
//! - **Payment Verification** - Complete payment payload signature verification

/// EIP-712 domain separator for EIP-3009 transfers
pub const EIP712_DOMAIN: &str = r#"{"name":"USD Coin","version":"2","chainId":8453,"verifyingContract":"0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"}"#;

pub mod eip712;
pub mod jwt;
pub mod signature;

#[cfg(test)]
mod tests;

// Re-export commonly used items
pub use eip712::{Domain, TypedData};
pub use jwt::{create_auth_header, create_auth_header_with_method, generate_jwt, JwtOptions};
pub use signature::{
    generate_nonce, sign_message_hash, verify_eip712_signature, verify_payment_payload,
};
