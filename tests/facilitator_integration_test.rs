//! Comprehensive integration tests for facilitator functionality
//!
//! These tests verify the complete facilitator workflow including:
//! - Payment verification
//! - Payment settlement
//! - Discovery API
//! - Error handling
//! - Network validation
//! - Blockchain facilitator integration

use mockito::{Matcher, Server};
use rust_x402::{
    facilitator::FacilitatorClient,
    types::{
        ExactEvmPayload, ExactEvmPayloadAuthorization, FacilitatorConfig, PaymentPayload,
        PaymentRequirements,
    },
};
use serde_json::json;
use std::collections::HashMap;

// Test account from Anvil
const TEST_PAYER_ADDRESS: &str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

/// Helper function to create a test payment payload
fn create_test_payment_payload() -> PaymentPayload {
    let authorization = ExactEvmPayloadAuthorization::new(
        TEST_PAYER_ADDRESS,
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
        "1000000",
        "1745323800",
        "1745323985",
        "0xf3746613c2d920b5fdabc0856f2aeb2d4f88ee6037b8cc5d04a71a4462f13480",
    );

    let payload = ExactEvmPayload {
        signature: "0x2d6a7588d6acca505cbf0d9a4a227e0c52c6c34008c8e8986a1283259764173608a2ce6496642e377d6da8dbbf5836e9bd15092f9ecab05ded3d6293af148b571c".to_string(),
        authorization,
    };

    PaymentPayload::new("exact", "base-sepolia", payload)
}

/// Helper function to create test payment requirements
fn create_test_payment_requirements() -> PaymentRequirements {
    PaymentRequirements::new(
        "exact",
        "base-sepolia",
        "1000000",
        "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
        "https://example.com/test",
        "Test payment",
    )
}

#[tokio::test]
async fn test_facilitator_complete_verify_settle_flow() {
    let mut server = Server::new_async().await;

    // Mock verify endpoint
    let verify_mock = server
        .mock("POST", "/verify")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "x402Version": 1,
                "isValid": true,
                "payer": TEST_PAYER_ADDRESS
            })
            .to_string(),
        )
        .create();

    // Mock settle endpoint
    let settle_mock = server
        .mock("POST", "/settle")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "x402Version": 1,
                "success": true,
                "transaction": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                "network": "base-sepolia",
                "payer": TEST_PAYER_ADDRESS
            })
            .to_string(),
        )
        .create();

    let config = FacilitatorConfig::new(server.url());
    let client = FacilitatorClient::new(config).unwrap();

    let payment_payload = create_test_payment_payload();
    let payment_requirements = create_test_payment_requirements();

    // Step 1: Verify payment
    let verify_response = client
        .verify(&payment_payload, &payment_requirements)
        .await
        .unwrap();

    assert!(verify_response.is_valid);
    assert_eq!(verify_response.payer, Some(TEST_PAYER_ADDRESS.to_string()));
    verify_mock.assert();

    // Step 2: Settle payment
    let settle_response = client
        .settle(&payment_payload, &payment_requirements)
        .await
        .unwrap();

    assert!(settle_response.success);
    assert_eq!(settle_response.network, "base-sepolia");
    assert_eq!(settle_response.payer, Some(TEST_PAYER_ADDRESS.to_string()));
    assert!(!settle_response.transaction.is_empty());
    settle_mock.assert();
}

#[tokio::test]
async fn test_facilitator_verify_with_invalid_signature() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", "/verify")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "x402Version": 1,
                "isValid": false,
                "invalidReason": "Invalid signature",
                "payer": TEST_PAYER_ADDRESS
            })
            .to_string(),
        )
        .create();

    let config = FacilitatorConfig::new(server.url());
    let client = FacilitatorClient::new(config).unwrap();

    let payment_payload = create_test_payment_payload();
    let payment_requirements = create_test_payment_requirements();

    let verify_response = client
        .verify(&payment_payload, &payment_requirements)
        .await
        .unwrap();

    assert!(!verify_response.is_valid);
    assert_eq!(
        verify_response.invalid_reason,
        Some("Invalid signature".to_string())
    );
}

#[tokio::test]
async fn test_facilitator_verify_with_insufficient_funds() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", "/verify")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "x402Version": 1,
                "isValid": false,
                "invalidReason": "Insufficient funds",
                "payer": TEST_PAYER_ADDRESS
            })
            .to_string(),
        )
        .create();

    let config = FacilitatorConfig::new(server.url());
    let client = FacilitatorClient::new(config).unwrap();

    let payment_payload = create_test_payment_payload();
    let payment_requirements = create_test_payment_requirements();

    let verify_response = client
        .verify(&payment_payload, &payment_requirements)
        .await
        .unwrap();

    assert!(!verify_response.is_valid);
    assert_eq!(
        verify_response.invalid_reason,
        Some("Insufficient funds".to_string())
    );
}

#[tokio::test]
async fn test_facilitator_settle_with_verification_error() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", "/settle")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "x402Version": 1,
                "success": false,
                "errorReason": "Payment not verified",
                "transaction": "",
                "network": "base-sepolia",
                "payer": TEST_PAYER_ADDRESS
            })
            .to_string(),
        )
        .create();

    let config = FacilitatorConfig::new(server.url());
    let client = FacilitatorClient::new(config).unwrap();

    let payment_payload = create_test_payment_payload();
    let payment_requirements = create_test_payment_requirements();

    let settle_response = client
        .settle(&payment_payload, &payment_requirements)
        .await
        .unwrap();

    assert!(!settle_response.success);
    assert_eq!(
        settle_response.error_reason,
        Some("Payment not verified".to_string())
    );
    assert_eq!(settle_response.transaction, "");
}

#[tokio::test]
async fn test_facilitator_supported_kinds() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("GET", "/supported")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "x402Version": 1,
                "kinds": [
                    {
                        "x402Version": 1,
                        "scheme": "exact",
                        "network": "base-sepolia"
                    },
                    {
                        "x402Version": 1,
                        "scheme": "exact",
                        "network": "base"
                    },
                    {
                        "x402Version": 1,
                        "scheme": "exact",
                        "network": "avalanche-fuji"
                    }
                ]
            })
            .to_string(),
        )
        .create();

    let config = FacilitatorConfig::new(server.url());
    let client = FacilitatorClient::new(config).unwrap();

    let supported = client.supported().await.unwrap();

    assert_eq!(supported.kinds.len(), 3);
    assert_eq!(supported.kinds[0].scheme, "exact");
    assert_eq!(supported.kinds[0].network, "base-sepolia");
    assert_eq!(supported.kinds[1].network, "base");
    assert_eq!(supported.kinds[2].network, "avalanche-fuji");
}

#[tokio::test]
async fn test_facilitator_discovery_complete_flow() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("GET", "/discovery/resources")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "x402Version": 1,
                "items": [
                    {
                        "resource": "https://api.example.com/premium",
                        "type": "http",
                        "x402Version": 1,
                        "accepts": [{
                            "scheme": "exact",
                            "network": "base-sepolia",
                            "maxAmountRequired": "1000000",
                            "resource": "https://api.example.com/premium",
                            "description": "Premium API access",
                            "mimeType": "application/json",
                            "payTo": "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
                            "maxTimeoutSeconds": 60,
                            "asset": "0x036CbD53842c5426634e7929541eC2318f3dCF7e"
                        }],
                        "lastUpdated": 1640995200,
                        "metadata": {
                            "category": "api",
                            "provider": "Example Corp"
                        }
                    }
                ],
                "pagination": {
                    "total": 1,
                    "limit": 20,
                    "offset": 0
                }
            })
            .to_string(),
        )
        .create();

    let config = FacilitatorConfig::new(server.url());
    let client = FacilitatorClient::new(config).unwrap();

    // Test list_all
    let response = client.list_all().await.unwrap();
    assert_eq!(response.items.len(), 1);
    assert_eq!(
        response.items[0].resource,
        "https://api.example.com/premium"
    );
    assert_eq!(response.items[0].r#type, "http");
    assert_eq!(response.pagination.total, 1);

    // Create a new mock server for list_by_type since the previous mock was consumed
    let mut server2 = Server::new_async().await;
    let _mock2 = server2
        .mock("GET", "/discovery/resources")
        .match_query(Matcher::UrlEncoded("type".to_string(), "http".to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "x402Version": 1,
                "items": [
                    {
                        "resource": "https://api.example.com/premium",
                        "type": "http",
                        "x402Version": 1,
                        "accepts": [],
                        "lastUpdated": 1640995200
                    }
                ],
                "pagination": {
                    "total": 1,
                    "limit": 20,
                    "offset": 0
                }
            })
            .to_string(),
        )
        .create();

    let config2 = FacilitatorConfig::new(server2.url());
    let client2 = FacilitatorClient::new(config2).unwrap();

    // Test list_by_type
    let response = client2.list_by_type("http").await.unwrap();
    assert_eq!(response.items.len(), 1);
}

#[tokio::test]
async fn test_facilitator_discovery_with_pagination() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("GET", "/discovery/resources")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("limit".to_string(), "10".to_string()),
            Matcher::UrlEncoded("offset".to_string(), "5".to_string()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "x402Version": 1,
                "items": [],
                "pagination": {
                    "total": 20,
                    "limit": 10,
                    "offset": 5
                }
            })
            .to_string(),
        )
        .create();

    let config = FacilitatorConfig::new(server.url());
    let client = FacilitatorClient::new(config).unwrap();

    let filters = rust_x402::client::DiscoveryFilters::new()
        .with_limit(10)
        .with_offset(5);

    let response = client.list(Some(filters)).await.unwrap();
    assert_eq!(response.pagination.total, 20);
    assert_eq!(response.pagination.limit, 10);
    assert_eq!(response.pagination.offset, 5);
}

#[tokio::test]
async fn test_facilitator_error_handling() {
    // Test HTTP error
    let mut server = Server::new_async().await;
    let _mock = server.mock("POST", "/verify").with_status(500).create();

    let config = FacilitatorConfig::new(server.url());
    let client = FacilitatorClient::new(config).unwrap();

    let payment_payload = create_test_payment_payload();
    let payment_requirements = create_test_payment_requirements();

    let result = client.verify(&payment_payload, &payment_requirements).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Verification failed with status: 500"));

    // Test network error
    let config = FacilitatorConfig::new("http://invalid-host-12345:9999");
    let client = FacilitatorClient::new(config).unwrap();

    let result = client.verify(&payment_payload, &payment_requirements).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_facilitator_timeout_configuration() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", "/verify")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "x402Version": 1,
                "isValid": true,
                "payer": TEST_PAYER_ADDRESS
            })
            .to_string(),
        )
        .create();

    // Test with timeout configuration
    let config =
        FacilitatorConfig::new(server.url()).with_timeout(std::time::Duration::from_secs(1));
    let client = FacilitatorClient::new(config).unwrap();

    let payment_payload = create_test_payment_payload();
    let payment_requirements = create_test_payment_requirements();

    let result = client.verify(&payment_payload, &payment_requirements).await;
    assert!(result.is_ok(), "Should succeed with timeout configuration");
}

#[tokio::test]
async fn test_facilitator_auth_headers_for_all_endpoints() {
    let mut server = Server::new_async().await;

    // Mock verify with auth
    let verify_mock = server
        .mock("POST", "/verify")
        .match_header("Authorization", "Bearer verify-token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "x402Version": 1,
                "isValid": true,
                "payer": TEST_PAYER_ADDRESS
            })
            .to_string(),
        )
        .create();

    // Mock settle with auth
    let settle_mock = server
        .mock("POST", "/settle")
        .match_header("Authorization", "Bearer settle-token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "x402Version": 1,
                "success": true,
                "transaction": "0x123",
                "network": "base-sepolia",
                "payer": TEST_PAYER_ADDRESS
            })
            .to_string(),
        )
        .create();

    // Mock supported with auth
    let supported_mock = server
        .mock("GET", "/supported")
        .match_header("Authorization", "Bearer supported-token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "x402Version": 1,
                "kinds": []
            })
            .to_string(),
        )
        .create();

    let create_auth_headers = || -> rust_x402::Result<HashMap<String, HashMap<String, String>>> {
        let mut headers = HashMap::new();
        let mut verify_headers = HashMap::new();
        verify_headers.insert(
            "Authorization".to_string(),
            "Bearer verify-token".to_string(),
        );
        headers.insert("verify".to_string(), verify_headers);

        let mut settle_headers = HashMap::new();
        settle_headers.insert(
            "Authorization".to_string(),
            "Bearer settle-token".to_string(),
        );
        headers.insert("settle".to_string(), settle_headers);

        let mut supported_headers = HashMap::new();
        supported_headers.insert(
            "Authorization".to_string(),
            "Bearer supported-token".to_string(),
        );
        headers.insert("supported".to_string(), supported_headers);

        Ok(headers)
    };

    let config =
        FacilitatorConfig::new(server.url()).with_auth_headers(Box::new(create_auth_headers));
    let client = FacilitatorClient::new(config).unwrap();

    let payment_payload = create_test_payment_payload();
    let payment_requirements = create_test_payment_requirements();

    // Test verify with auth
    client
        .verify(&payment_payload, &payment_requirements)
        .await
        .unwrap();
    verify_mock.assert();

    // Test settle with auth
    client
        .settle(&payment_payload, &payment_requirements)
        .await
        .unwrap();
    settle_mock.assert();

    // Test supported with auth
    client.supported().await.unwrap();
    supported_mock.assert();
}

#[tokio::test]
async fn test_facilitator_network_specific_methods() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("GET", "/supported")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "x402Version": 1,
                "kinds": [
                    {
                        "x402Version": 1,
                        "scheme": "exact",
                        "network": "base-sepolia"
                    }
                ]
            })
            .to_string(),
        )
        .create();

    // Test for_base_mainnet
    let config = FacilitatorConfig::new(server.url());
    let client = FacilitatorClient::for_base_mainnet(config).unwrap();
    assert_eq!(client.url(), server.url());

    // Test for_base_sepolia
    let config = FacilitatorConfig::new(server.url());
    let client = FacilitatorClient::for_base_sepolia(config).unwrap();
    assert_eq!(client.url(), server.url());

    // Test for_network
    let config = FacilitatorConfig::new(server.url());
    let client = FacilitatorClient::for_network("base-sepolia", config).unwrap();
    assert_eq!(client.url(), server.url());
}

#[tokio::test]
async fn test_facilitator_config_validation() {
    // Valid config
    let config = FacilitatorConfig::new("https://example.com/facilitator");
    assert!(FacilitatorClient::new(config).is_ok());

    // Invalid URL
    let config = FacilitatorConfig::new("invalid-url");
    assert!(FacilitatorClient::new(config).is_err());

    // Empty URL
    let config = FacilitatorConfig::new("");
    assert!(FacilitatorClient::new(config).is_err());
}

#[tokio::test]
async fn test_facilitator_request_serialization() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", "/verify")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "x402Version": 1,
                "isValid": true,
                "payer": TEST_PAYER_ADDRESS
            })
            .to_string(),
        )
        .create();

    let config = FacilitatorConfig::new(server.url());
    let client = FacilitatorClient::new(config).unwrap();

    let payment_payload = create_test_payment_payload();
    let payment_requirements = create_test_payment_requirements();

    let response = client
        .verify(&payment_payload, &payment_requirements)
        .await
        .unwrap();

    assert!(response.is_valid);
    // Verify that the request was properly serialized by checking the mock was called
    _mock.assert();
}

#[tokio::test]
async fn test_facilitator_response_deserialization() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", "/verify")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "x402Version": 1,
                "isValid": true,
                "invalidReason": null,
                "payer": TEST_PAYER_ADDRESS
            })
            .to_string(),
        )
        .create();

    let config = FacilitatorConfig::new(server.url());
    let client = FacilitatorClient::new(config).unwrap();

    let payment_payload = create_test_payment_payload();
    let payment_requirements = create_test_payment_requirements();

    let response = client
        .verify(&payment_payload, &payment_requirements)
        .await
        .unwrap();

    assert!(response.is_valid);
    assert!(response.invalid_reason.is_none());
    assert_eq!(response.payer, Some(TEST_PAYER_ADDRESS.to_string()));
}

#[tokio::test]
async fn test_facilitator_settle_response_with_transaction() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", "/settle")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "x402Version": 1,
                "success": true,
                "errorReason": null,
                "transaction": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
                "network": "base-sepolia",
                "payer": TEST_PAYER_ADDRESS
            })
            .to_string(),
        )
        .create();

    let config = FacilitatorConfig::new(server.url());
    let client = FacilitatorClient::new(config).unwrap();

    let payment_payload = create_test_payment_payload();
    let payment_requirements = create_test_payment_requirements();

    let response = client
        .settle(&payment_payload, &payment_requirements)
        .await
        .unwrap();

    assert!(response.success);
    assert!(response.error_reason.is_none());
    assert_eq!(
        response.transaction,
        "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
    );
    assert_eq!(response.network, "base-sepolia");
    assert_eq!(response.payer, Some(TEST_PAYER_ADDRESS.to_string()));
}
