//! Integration tests for middleware payment flow
//!
//! These tests verify the middleware payment processing logic,
//! including payment requirements creation and configuration.

use rust_decimal::Decimal;
use rust_x402::{
    middleware::{PaymentMiddleware, PaymentMiddlewareConfig},
    types::PaymentRequirements,
};
use std::str::FromStr;

#[test]
fn test_middleware_payment_requirements_creation() {
    let middleware = PaymentMiddleware::new(
        Decimal::from_str("0.0001").unwrap(),
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
    )
    .with_description("Test payment")
    .with_testnet(true);

    // Test payment requirements creation
    let requirements = middleware
        .config()
        .create_payment_requirements("/protected")
        .expect("Should create payment requirements");

    assert_eq!(requirements.scheme, "exact");
    assert_eq!(requirements.network, "base-sepolia");
    assert_eq!(requirements.description, "Test payment");
    assert_eq!(
        requirements.pay_to.to_lowercase(),
        "0x209693bc6afc0c5328ba36faf03c514ef312287c"
    );
}

#[test]
fn test_middleware_configuration_options() {
    let middleware = PaymentMiddleware::new(
        Decimal::from_str("0.001").unwrap(),
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
    )
    .with_description("Custom payment")
    .with_mime_type("application/json")
    .with_max_timeout_seconds(120)
    .with_testnet(true);

    let config = middleware.config();
    assert_eq!(config.amount, Decimal::from_str("0.001").unwrap());
    assert_eq!(config.description, Some("Custom payment".to_string()));
    assert_eq!(config.mime_type, Some("application/json".to_string()));
    assert_eq!(config.max_timeout_seconds, 120);
    assert!(config.testnet);
}

#[test]
fn test_middleware_facilitator_config() {
    let facilitator_config =
        rust_x402::types::FacilitatorConfig::new("https://example.com/facilitator");

    let middleware = PaymentMiddleware::new(
        Decimal::from_str("0.0001").unwrap(),
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
    )
    .with_description("Test payment")
    .with_testnet(true)
    .with_facilitator_config(facilitator_config.clone());

    let config = middleware.config();
    assert_eq!(config.facilitator_config.url, facilitator_config.url);
}

#[test]
fn test_middleware_template_config() {
    let template_config = rust_x402::template::PaywallConfig::new()
        .with_app_name("Test App")
        .with_app_logo("https://example.com/logo.png");

    let middleware = PaymentMiddleware::new(
        Decimal::from_str("0.0001").unwrap(),
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
    )
    .with_template_config(template_config.clone());

    assert!(middleware.template_config.is_some());
    let config = middleware.template_config.unwrap();
    assert_eq!(config.app_name, Some("Test App".to_string()));
    assert_eq!(
        config.app_logo,
        Some("https://example.com/logo.png".to_string())
    );
}

#[test]
fn test_middleware_config_builder() {
    let config = PaymentMiddlewareConfig::new(
        Decimal::from_str("0.0001").unwrap(),
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
    )
    .with_description("Test payment")
    .with_mime_type("application/json")
    .with_max_timeout_seconds(60)
    .with_testnet(true);

    assert_eq!(config.amount, Decimal::from_str("0.0001").unwrap());
    assert_eq!(config.description, Some("Test payment".to_string()));
    assert_eq!(config.mime_type, Some("application/json".to_string()));
    assert_eq!(config.max_timeout_seconds, 60);
    assert!(config.testnet);
}

#[test]
fn test_middleware_payment_requirements_for_different_paths() {
    let middleware = PaymentMiddleware::new(
        Decimal::from_str("0.0001").unwrap(),
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
    )
    .with_description("Test payment")
    .with_testnet(true);

    let paths = vec!["/protected", "/api/data", "/premium/content"];

    for path in paths {
        let requirements = middleware
            .config()
            .create_payment_requirements(path)
            .unwrap_or_else(|_| panic!("Should create payment requirements for {}", path));

        assert_eq!(requirements.scheme, "exact");
        assert_eq!(requirements.network, "base-sepolia");
        assert_eq!(requirements.resource, path);
    }
}

#[test]
fn test_middleware_custom_resource_url() {
    let middleware = PaymentMiddleware::new(
        Decimal::from_str("0.0001").unwrap(),
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
    )
    .with_description("Test payment")
    .with_testnet(true)
    .with_resource("https://api.example.com/premium");

    let requirements = middleware
        .config()
        .create_payment_requirements("/test")
        .expect("Should create payment requirements");

    assert_eq!(requirements.resource, "https://api.example.com/premium");
}

#[test]
fn test_middleware_mainnet_configuration() {
    let middleware = PaymentMiddleware::new(
        Decimal::from_str("0.01").unwrap(),
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
    )
    .with_description("Mainnet payment")
    .with_testnet(false); // Use mainnet

    let requirements = middleware
        .config()
        .create_payment_requirements("/protected")
        .expect("Should create payment requirements");

    assert_eq!(requirements.network, "base"); // Mainnet network
    assert_eq!(
        requirements.asset,
        "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"
    ); // Mainnet USDC
}
