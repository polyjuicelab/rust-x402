//! Integration tests for proxy server functionality
//!
//! These tests verify proxy server configuration, payment integration,
//! and request forwarding capabilities.

use rust_x402::proxy::ProxyConfig;
use std::collections::HashMap;

#[test]
fn test_proxy_config_default() {
    let config = ProxyConfig::default();

    assert_eq!(config.amount, 0.0001);
    assert_eq!(config.max_timeout_seconds, 60);
    assert_eq!(config.facilitator_url, "https://x402.org/facilitator");
    assert!(config.testnet);
    assert!(config.headers.is_empty());
}

#[test]
fn test_proxy_config_creation() {
    let mut headers = HashMap::new();
    headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());

    let config = ProxyConfig {
        target_url: "https://api.example.com".to_string(),
        amount: 0.01,
        pay_to: "0x209693Bc6afc0C5328bA36FaF03C514EF312287C".to_string(),
        description: Some("Proxy payment".to_string()),
        mime_type: Some("application/json".to_string()),
        max_timeout_seconds: 120,
        facilitator_url: "https://facilitator.example.com".to_string(),
        testnet: false,
        headers,
        cdp_api_key_id: None,
        cdp_api_key_secret: None,
    };

    assert_eq!(config.target_url, "https://api.example.com");
    assert_eq!(config.amount, 0.01);
    assert_eq!(config.pay_to, "0x209693Bc6afc0C5328bA36FaF03C514EF312287C");
    assert_eq!(config.description, Some("Proxy payment".to_string()));
    assert_eq!(config.mime_type, Some("application/json".to_string()));
    assert_eq!(config.max_timeout_seconds, 120);
    assert_eq!(config.facilitator_url, "https://facilitator.example.com");
    assert!(!config.testnet);
    assert_eq!(config.headers.len(), 1);
}

#[test]
fn test_proxy_config_validation() {
    // Valid config
    let mut config = ProxyConfig::default();
    config.target_url = "https://api.example.com".to_string();
    config.pay_to = "0x209693Bc6afc0C5328bA36FaF03C514EF312287C".to_string();

    assert!(config.validate().is_ok());

    // Invalid config - empty target URL
    let mut invalid_config = ProxyConfig::default();
    invalid_config.pay_to = "0x209693Bc6afc0C5328bA36FaF03C514EF312287C".to_string();
    assert!(invalid_config.validate().is_err());

    // Invalid config - empty pay_to
    let mut invalid_config2 = ProxyConfig::default();
    invalid_config2.target_url = "https://api.example.com".to_string();
    assert!(invalid_config2.validate().is_err());
}

#[test]
fn test_proxy_config_to_payment_config() {
    let mut config = ProxyConfig::default();
    config.target_url = "https://api.example.com".to_string();
    config.amount = 0.001;
    config.pay_to = "0x209693Bc6afc0C5328bA36FaF03C514EF312287C".to_string();
    config.description = Some("Test payment".to_string());
    config.mime_type = Some("application/json".to_string());
    config.max_timeout_seconds = 90;
    config.testnet = true;

    let payment_config = config
        .to_payment_config()
        .expect("Should convert to payment config");

    assert_eq!(payment_config.amount.to_string(), "0.001");
    assert_eq!(
        payment_config.pay_to,
        "0x209693bc6afc0c5328ba36faf03c514ef312287c"
    );
    assert_eq!(payment_config.description, Some("Test payment".to_string()));
    assert_eq!(
        payment_config.mime_type,
        Some("application/json".to_string())
    );
    assert_eq!(payment_config.max_timeout_seconds, 90);
    assert!(payment_config.testnet);
}

#[test]
fn test_proxy_config_with_custom_headers() {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer token123".to_string());
    headers.insert("X-API-Key".to_string(), "api-key-456".to_string());

    let config = ProxyConfig {
        target_url: "https://api.example.com".to_string(),
        amount: 0.0001,
        pay_to: "0x209693Bc6afc0C5328bA36FaF03C514EF312287C".to_string(),
        description: None,
        mime_type: None,
        max_timeout_seconds: 60,
        facilitator_url: "https://x402.org/facilitator".to_string(),
        testnet: true,
        headers: headers.clone(),
        cdp_api_key_id: None,
        cdp_api_key_secret: None,
    };

    assert_eq!(config.headers.len(), 2);
    assert_eq!(
        config.headers.get("Authorization"),
        Some(&"Bearer token123".to_string())
    );
    assert_eq!(
        config.headers.get("X-API-Key"),
        Some(&"api-key-456".to_string())
    );
}

#[test]
fn test_proxy_config_with_cdp_credentials() {
    let config = ProxyConfig {
        target_url: "https://api.example.com".to_string(),
        amount: 0.0001,
        pay_to: "0x209693Bc6afc0C5328bA36FaF03C514EF312287C".to_string(),
        description: None,
        mime_type: None,
        max_timeout_seconds: 60,
        facilitator_url: "https://x402.org/facilitator".to_string(),
        testnet: true,
        headers: HashMap::new(),
        cdp_api_key_id: Some("cdp-key-id-123".to_string()),
        cdp_api_key_secret: Some("cdp-secret-456".to_string()),
    };

    assert_eq!(config.cdp_api_key_id, Some("cdp-key-id-123".to_string()));
    assert_eq!(
        config.cdp_api_key_secret,
        Some("cdp-secret-456".to_string())
    );
}

#[test]
fn test_proxy_config_serialization() {
    let config = ProxyConfig {
        target_url: "https://api.example.com".to_string(),
        amount: 0.0001,
        pay_to: "0x209693Bc6afc0C5328bA36FaF03C514EF312287C".to_string(),
        description: Some("Test payment".to_string()),
        mime_type: Some("application/json".to_string()),
        max_timeout_seconds: 60,
        facilitator_url: "https://x402.org/facilitator".to_string(),
        testnet: true,
        headers: HashMap::new(),
        cdp_api_key_id: None,
        cdp_api_key_secret: None,
    };

    // Test JSON serialization
    let json = serde_json::to_string(&config).expect("Should serialize to JSON");
    let deserialized: ProxyConfig =
        serde_json::from_str(&json).expect("Should deserialize from JSON");

    assert_eq!(config.target_url, deserialized.target_url);
    assert_eq!(config.amount, deserialized.amount);
    assert_eq!(config.pay_to, deserialized.pay_to);
}

#[test]
fn test_proxy_config_amount_validation() {
    // Test amount validation
    let mut config = ProxyConfig::default();
    config.target_url = "https://api.example.com".to_string();
    config.pay_to = "0x209693Bc6afc0C5328bA36FaF03C514EF312287C".to_string();

    // Valid amount
    config.amount = 0.001;
    assert!(config.validate().is_ok());

    // Invalid amount (zero)
    config.amount = 0.0;
    assert!(config.validate().is_err());

    // Invalid amount (negative)
    config.amount = -0.001;
    assert!(config.validate().is_err());
}

#[test]
fn test_proxy_config_mainnet() {
    let config = ProxyConfig {
        target_url: "https://api.example.com".to_string(),
        amount: 0.01,
        pay_to: "0x209693Bc6afc0C5328bA36FaF03C514EF312287C".to_string(),
        description: Some("Mainnet payment".to_string()),
        mime_type: None,
        max_timeout_seconds: 60,
        facilitator_url: "https://x402.org/facilitator".to_string(),
        testnet: false, // Mainnet
        headers: HashMap::new(),
        cdp_api_key_id: None,
        cdp_api_key_secret: None,
    };

    let payment_config = config
        .to_payment_config()
        .expect("Should convert to payment config");
    assert!(!payment_config.testnet);
}
