//! Integration tests for template system
//!
//! These tests verify HTML template generation, configuration injection,
//! and paywall customization features.

use rust_x402::{
    template::config::PaywallConfigBuilder,
    template::{PaywallConfig, ThemeConfig},
    types::PaymentRequirements,
};

#[test]
fn test_template_default_config() {
    let config = PaywallConfig::default();

    assert!(config.app_name.is_none());
    assert!(config.app_logo.is_none());
    assert!(config.cdp_client_key.is_none());
    assert!(config.session_token_endpoint.is_none());
    assert!(config.custom_css.is_none());
    assert!(config.custom_js.is_none());
    assert!(config.theme.is_none());
    assert!(config.branding.is_none());
}

#[test]
fn test_template_config_builder() {
    let config = PaywallConfigBuilder::new()
        .app_name("Test App")
        .app_logo("https://example.com/logo.png")
        .cdp_client_key("test-key")
        .session_token_endpoint("https://example.com/token")
        .build();

    assert_eq!(config.app_name, Some("Test App".to_string()));
    assert_eq!(
        config.app_logo,
        Some("https://example.com/logo.png".to_string())
    );
    assert_eq!(config.cdp_client_key, Some("test-key".to_string()));
    assert_eq!(
        config.session_token_endpoint,
        Some("https://example.com/token".to_string())
    );
}

#[test]
fn test_template_config_direct() {
    let config = PaywallConfig::new()
        .with_app_name("Test App")
        .with_app_logo("https://example.com/logo.png")
        .with_cdp_client_key("test-key")
        .with_session_token_endpoint("https://example.com/token");

    assert_eq!(config.app_name, Some("Test App".to_string()));
    assert_eq!(
        config.app_logo,
        Some("https://example.com/logo.png".to_string())
    );
    assert_eq!(config.cdp_client_key, Some("test-key".to_string()));
    assert_eq!(
        config.session_token_endpoint,
        Some("https://example.com/token".to_string())
    );
}

#[test]
fn test_template_theme_config() {
    let theme = ThemeConfig {
        primary_color: "#FF0000".to_string(),
        secondary_color: "#00FF00".to_string(),
        background_color: "#FFFFFF".to_string(),
        text_color: "#000000".to_string(),
        border_radius: "8px".to_string(),
    };

    assert_eq!(theme.primary_color, "#FF0000");
    assert_eq!(theme.secondary_color, "#00FF00");
    assert_eq!(theme.background_color, "#FFFFFF");
    assert_eq!(theme.text_color, "#000000");
    assert_eq!(theme.border_radius, "8px");
}

#[test]
fn test_template_theme_config_default() {
    let theme = ThemeConfig::default();

    assert_eq!(theme.primary_color, "#667eea");
    assert_eq!(theme.secondary_color, "#764ba2");
    assert_eq!(theme.background_color, "#ffffff");
    assert_eq!(theme.text_color, "#1a1a1a");
    assert_eq!(theme.border_radius, "16px");
}

#[test]
fn test_template_config_with_theme() {
    let theme = ThemeConfig {
        primary_color: "#007bff".to_string(),
        secondary_color: "#6c757d".to_string(),
        background_color: "#ffffff".to_string(),
        text_color: "#212529".to_string(),
        border_radius: "12px".to_string(),
    };

    let config = PaywallConfig::new()
        .with_app_name("Themed App")
        .with_theme(theme.clone());

    assert_eq!(config.app_name, Some("Themed App".to_string()));
    assert!(config.theme.is_some());
    let config_theme = config.theme.unwrap();
    assert_eq!(config_theme.primary_color, theme.primary_color);
    assert_eq!(config_theme.secondary_color, theme.secondary_color);
}

#[test]
fn test_template_generate_html_with_config() {
    let requirements = vec![PaymentRequirements::new(
        "exact",
        "base-sepolia",
        "1000000", // 1 USDC
        "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
        "https://example.com/protected",
        "Test payment",
    )];

    let config = PaywallConfigBuilder::new()
        .app_name("Test App")
        .app_logo("https://example.com/logo.png")
        .build();

    // Generate HTML
    let html = rust_x402::template::generate_paywall_html(
        "Payment required",
        &requirements,
        Some(&config),
    );

    // Verify HTML contains expected elements
    assert!(html.contains("Test App"));
    assert!(html.contains("https://example.com/logo.png"));
    assert!(html.contains("Payment required"));
    assert!(html.contains("x402"));
    assert!(html.contains("window.x402"));
}

#[test]
fn test_template_generate_html_without_config() {
    let requirements = vec![PaymentRequirements::new(
        "exact",
        "base-sepolia",
        "1000000",
        "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
        "https://example.com/protected",
        "Test payment",
    )];

    // Generate HTML without config
    let html = rust_x402::template::generate_paywall_html("", &requirements, None);

    // Verify HTML contains basic structure
    assert!(html.contains("<!DOCTYPE html"));
    assert!(html.contains("<html"));
    assert!(html.contains("x402"));
    assert!(html.contains("paymentRequirements"));
}

#[test]
fn test_template_generate_html_with_theme() {
    let requirements = vec![PaymentRequirements::new(
        "exact",
        "base-sepolia",
        "1000000",
        "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
        "https://example.com/protected",
        "Test payment",
    )];

    let theme = ThemeConfig {
        primary_color: "#FF5733".to_string(),
        secondary_color: "#33FF57".to_string(),
        background_color: "#F0F0F0".to_string(),
        text_color: "#333333".to_string(),
        border_radius: "20px".to_string(),
    };

    let config = PaywallConfig::new()
        .with_app_name("Themed App")
        .with_theme(theme);

    let html = rust_x402::template::generate_paywall_html("", &requirements, Some(&config));

    // Verify theme colors are included in the configuration JSON
    assert!(html.contains("#FF5733"));
    assert!(html.contains("#33FF57"));
    assert!(html.contains("primaryColor"));
    assert!(html.contains("secondaryColor"));
}

#[test]
fn test_template_generate_html_with_custom_css_js() {
    let requirements = vec![PaymentRequirements::new(
        "exact",
        "base-sepolia",
        "1000000",
        "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
        "https://example.com/protected",
        "Test payment",
    )];

    let config = PaywallConfig::new()
        .with_app_name("Custom App")
        .with_custom_css(".custom { color: red; }")
        .with_custom_js("console.log('Custom JS');");

    let html = rust_x402::template::generate_paywall_html("", &requirements, Some(&config));

    // Verify custom CSS and JS are included
    assert!(html.contains(".custom { color: red; }"));
    assert!(html.contains("console.log('Custom JS');"));
}

#[test]
fn test_template_generate_html_with_cdp_config() {
    let requirements = vec![PaymentRequirements::new(
        "exact",
        "base-sepolia",
        "1000000",
        "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
        "https://example.com/protected",
        "Test payment",
    )];

    let config = PaywallConfig::new()
        .with_app_name("CDP App")
        .with_cdp_client_key("cdp-test-key-123")
        .with_session_token_endpoint("https://api.example.com/session");

    let html = rust_x402::template::generate_paywall_html("", &requirements, Some(&config));

    // Verify CDP configuration is included
    assert!(html.contains("cdp-test-key-123"));
    assert!(html.contains("https://api.example.com/session"));
    assert!(html.contains("cdpClientKey"));
    assert!(html.contains("sessionTokenEndpoint"));
}

#[test]
fn test_template_html_contains_payment_requirements() {
    let requirements = vec![PaymentRequirements::new(
        "exact",
        "base-sepolia",
        "1000000",
        "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
        "https://example.com/protected",
        "Test payment description",
    )];

    let html = rust_x402::template::generate_paywall_html("", &requirements, None);

    // Verify payment requirements are embedded in the HTML
    assert!(html.contains("base-sepolia"));
    assert!(html.contains("Test payment description"));
    assert!(html.contains("0x036CbD53842c5426634e7929541eC2318f3dCF7e"));
    assert!(html.contains("0x209693Bc6afc0C5328bA36FaF03C514EF312287C"));
}

#[test]
fn test_template_html_contains_error_message() {
    let requirements = vec![PaymentRequirements::new(
        "exact",
        "base-sepolia",
        "1000000",
        "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
        "https://example.com/protected",
        "Test payment",
    )];

    let error_message = "Payment verification failed: Invalid signature";
    let html = rust_x402::template::generate_paywall_html(error_message, &requirements, None);

    // Verify error message is included
    assert!(html.contains(error_message));
    assert!(html.contains("error"));
}

#[test]
fn test_template_config_validation() {
    use rust_x402::template::config::validate_payment_requirements;

    // Valid requirements
    let valid_requirements = vec![PaymentRequirements::new(
        "exact",
        "base-sepolia",
        "1000000",
        "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
        "https://example.com/protected",
        "Test payment",
    )];

    assert!(validate_payment_requirements(&valid_requirements).is_ok());

    // Empty requirements
    let empty_requirements = vec![];
    assert!(validate_payment_requirements(&empty_requirements).is_err());

    // Invalid requirement (empty scheme)
    let invalid_req = PaymentRequirements::new(
        "",
        "base-sepolia",
        "1000000",
        "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
        "https://example.com/protected",
        "Test payment",
    );
    assert!(validate_payment_requirements(&[invalid_req]).is_err());
}
