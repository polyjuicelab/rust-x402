//! Middleware implementations for web frameworks
//!
//! This module provides middleware for integrating x402 payment protection into web applications.
//! It supports Axum framework with Tower service layer integration.
//!
//! # Architecture
//!
//! The middleware module is organized as follows:
//! - [`config`] - Middleware configuration and payment requirements builder
//! - [`payment`] - Core payment processing logic and middleware implementation
//! - [`service`] - Tower service layer for framework integration
//!
//! # Examples
//!
//! ## Basic Axum Integration
//!
//! ```ignore
//! use rust_x402::middleware::PaymentMiddleware;
//! use rust_decimal::Decimal;
//! use std::str::FromStr;
//! use axum::{Router, routing::get};
//!
//! # async fn example() {
//! // Create payment middleware
//! let middleware = PaymentMiddleware::new(
//!     Decimal::from_str("0.0001").unwrap(),           // 0.0001 USDC
//!     "0x209693Bc6afc0C5328bA36FaF03C514EF312287C"   // recipient address
//! )
//! .with_description("API Access")
//! .with_testnet(true);
//!
//! // Apply to routes
//! let app = Router::new()
//!     .route("/api/protected", get(|| async { "Protected content" }))
//!     .layer(axum::middleware::from_fn_with_state(
//!         middleware,
//!         rust_x402::middleware::payment_middleware
//!     ));
//! # }
//! ```
//!
//! ## Advanced Configuration
//!
//! ```ignore
//! use rust_x402::middleware::PaymentMiddleware;
//! use rust_x402::types::FacilitatorConfig;
//! use rust_decimal::Decimal;
//! use std::str::FromStr;
//!
//! # fn example() -> rust_x402::Result<()> {
//! // Configure facilitator
//! let facilitator_config = FacilitatorConfig::new("https://x402.org/facilitator");
//!
//! // Create middleware with full configuration
//! let middleware = PaymentMiddleware::new(
//!     Decimal::from_str("0.01")?,
//!     "0x209693Bc6afc0C5328bA36FaF03C514EF312287C"
//! )
//! .with_description("Premium API Access")
//! .with_mime_type("application/json")
//! .with_max_timeout_seconds(120)
//! .with_facilitator_config(facilitator_config)
//! .with_testnet(false)  // Use mainnet
//! .with_resource("https://api.example.com/premium");
//! # Ok(())
//! # }
//! ```
//!
//! ## Custom Paywall HTML
//!
//! ```ignore
//! use rust_x402::middleware::PaymentMiddleware;
//! use rust_x402::template::PaywallConfig;
//! use rust_decimal::Decimal;
//! use std::str::FromStr;
//!
//! # fn example() -> rust_x402::Result<()> {
//! let paywall_config = PaywallConfig::new()
//!     .with_app_name("My API")
//!     .with_app_logo("https://example.com/logo.png")
//!     .with_primary_color("#007bff");
//!
//! let middleware = PaymentMiddleware::new(
//!     Decimal::from_str("0.0001")?,
//!     "0x209693Bc6afc0C5328bA36FaF03C514EF312287C"
//! )
//! .with_template_config(paywall_config);
//! # Ok(())
//! # }
//! ```
//!
//! ## Using with Facilitator Client
//!
//! ```ignore
//! use rust_x402::middleware::PaymentMiddleware;
//! use rust_x402::facilitator::FacilitatorClient;
//! use rust_x402::types::FacilitatorConfig;
//! use rust_decimal::Decimal;
//! use std::str::FromStr;
//!
//! # async fn example() -> rust_x402::Result<()> {
//! // Create facilitator client
//! let facilitator_config = FacilitatorConfig::new("https://x402.org/facilitator");
//! let facilitator = FacilitatorClient::new(facilitator_config)?;
//!
//! // Create middleware with facilitator
//! let middleware = PaymentMiddleware::new(
//!     Decimal::from_str("0.0001")?,
//!     "0x209693Bc6afc0C5328bA36FaF03C514EF312287C"
//! )
//! .with_facilitator(facilitator);
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! - **Payment Verification** - Automatic payment verification before handling requests
//! - **Payment Settlement** - Automatic settlement after successful responses
//! - **Browser Support** - Automatic paywall HTML for web browsers
//! - **API Support** - JSON 402 responses for API clients
//! - **Flexible Configuration** - Rich configuration options for all payment parameters
//! - **Template System** - Customizable paywall HTML templates
//! - **Tower Integration** - Full Tower service layer support
//! - **Type Safety** - Strongly typed configuration with builder pattern
//!
//! # Response Handling
//!
//! The middleware automatically detects the client type:
//! - **Web Browsers** (Accept: text/html) - Returns HTML paywall page
//! - **API Clients** - Returns JSON with payment requirements
//!
//! # Payment Flow
//!
//! 1. Request arrives without X-PAYMENT header → 402 Payment Required
//! 2. Request arrives with X-PAYMENT header → Verify payment
//! 3. Payment valid → Execute handler
//! 4. Handler succeeds → Settle payment
//! 5. Return response with X-PAYMENT-RESPONSE header

pub mod config;
pub mod payment;
pub mod service;

#[cfg(test)]
mod tests;

// Re-export commonly used types
pub use config::PaymentMiddlewareConfig;
pub use payment::{payment_middleware, PaymentMiddleware, PaymentResult};
pub use service::{create_payment_service, PaymentService, PaymentServiceLayer};
