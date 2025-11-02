//! Tests for payment middleware

use super::config::PaymentMiddlewareConfig;
use super::payment::PaymentMiddleware;
use rust_decimal::Decimal;
use std::str::FromStr;

#[test]
fn test_payment_middleware_config() {
    let config = PaymentMiddlewareConfig::new(
        Decimal::from_str("0.0001").unwrap(),
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
    )
    .with_description("Test payment")
    .with_testnet(true);

    assert_eq!(config.amount, Decimal::from_str("0.0001").unwrap());
    assert_eq!(config.pay_to, "0x209693bc6afc0c5328ba36faf03c514ef312287c");
    assert_eq!(config.description, Some("Test payment".to_string()));
    assert!(config.testnet);
}

#[test]
fn test_payment_middleware_creation() {
    let middleware = PaymentMiddleware::new(
        Decimal::from_str("0.0001").unwrap(),
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
    )
    .with_description("Test payment");

    assert_eq!(
        middleware.config().amount,
        Decimal::from_str("0.0001").unwrap()
    );
    assert_eq!(
        middleware.config().pay_to,
        "0x209693bc6afc0c5328ba36faf03c514ef312287c"
    );
}

#[test]
fn test_payment_requirements_creation() {
    let config = PaymentMiddlewareConfig::new(
        Decimal::from_str("0.0001").unwrap(),
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
    )
    .with_testnet(true);

    let requirements = config.create_payment_requirements("/test").unwrap();

    assert_eq!(requirements.scheme, "exact");
    assert_eq!(requirements.network, "base-sepolia");
    assert_eq!(requirements.max_amount_required, "100");
    assert_eq!(
        requirements.pay_to,
        "0x209693bc6afc0c5328ba36faf03c514ef312287c"
    );
}

#[test]
fn test_payment_middleware_config_builder() {
    let config = PaymentMiddlewareConfig::new(
        Decimal::from_str("0.01").unwrap(),
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
    )
    .with_description("Test payment")
    .with_mime_type("application/json")
    .with_max_timeout_seconds(120)
    .with_testnet(false)
    .with_resource("https://example.com/test");

    assert_eq!(config.amount, Decimal::from_str("0.01").unwrap());
    assert_eq!(config.pay_to, "0x209693bc6afc0c5328ba36faf03c514ef312287c");
    assert_eq!(config.description, Some("Test payment".to_string()));
    assert_eq!(config.mime_type, Some("application/json".to_string()));
    assert_eq!(config.max_timeout_seconds, 120);
    assert!(!config.testnet);
    assert_eq!(
        config.resource,
        Some("https://example.com/test".to_string())
    );
}

#[test]
fn test_payment_middleware_creation_with_description() {
    let middleware = PaymentMiddleware::new(
        Decimal::from_str("0.001").unwrap(),
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
    )
    .with_description("Test middleware");

    assert_eq!(
        middleware.config().amount,
        Decimal::from_str("0.001").unwrap()
    );
    assert_eq!(
        middleware.config().description,
        Some("Test middleware".to_string())
    );
}
