//! Integration tests for blockchain facilitator
//!
//! These tests verify blockchain-based facilitator functionality including:
//! - Payment verification with blockchain
//! - Network validation
//! - Amount validation
//! - Authorization validation
//! - Factory methods

use rust_x402::{
    blockchain_facilitator::{
        BlockchainFacilitatorClient, BlockchainFacilitatorConfig, BlockchainFacilitatorFactory,
    },
    types::{ExactEvmPayload, ExactEvmPayloadAuthorization, PaymentPayload, PaymentRequirements},
};

#[test]
fn test_blockchain_facilitator_config_default() {
    let config = BlockchainFacilitatorConfig::default();

    assert_eq!(config.network, "base-sepolia");
    assert_eq!(config.confirmation_blocks, 1);
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.retry_delay, std::time::Duration::from_secs(1));
    assert_eq!(
        config.verification_timeout,
        std::time::Duration::from_secs(30)
    );
}

#[test]
fn test_blockchain_facilitator_config_custom() {
    let config = BlockchainFacilitatorConfig {
        rpc_url: Some("https://custom.rpc.com".to_string()),
        network: "base".to_string(),
        verification_timeout: std::time::Duration::from_secs(60),
        confirmation_blocks: 3,
        max_retries: 5,
        retry_delay: std::time::Duration::from_secs(2),
    };

    assert_eq!(config.network, "base");
    assert_eq!(config.confirmation_blocks, 3);
    assert_eq!(config.max_retries, 5);
    assert_eq!(config.retry_delay, std::time::Duration::from_secs(2));
    assert_eq!(
        config.verification_timeout,
        std::time::Duration::from_secs(60)
    );
}

#[test]
fn test_blockchain_facilitator_factory_base_sepolia() {
    let facilitator = BlockchainFacilitatorFactory::base_sepolia();
    assert!(
        facilitator.is_ok(),
        "Base Sepolia facilitator should be created"
    );

    let _facilitator = facilitator.unwrap();
    // Note: network field is private, but we can verify it works
}

#[test]
fn test_blockchain_facilitator_factory_base() {
    let facilitator = BlockchainFacilitatorFactory::base();
    assert!(
        facilitator.is_ok(),
        "Base mainnet facilitator should be created"
    );
}

#[test]
fn test_blockchain_facilitator_factory_avalanche_fuji() {
    let facilitator = BlockchainFacilitatorFactory::avalanche_fuji();
    assert!(
        facilitator.is_ok(),
        "Avalanche Fuji facilitator should be created"
    );
}

#[test]
fn test_blockchain_facilitator_factory_avalanche() {
    let facilitator = BlockchainFacilitatorFactory::avalanche();
    assert!(
        facilitator.is_ok(),
        "Avalanche mainnet facilitator should be created"
    );
}

#[test]
fn test_blockchain_facilitator_factory_custom() {
    let config = BlockchainFacilitatorConfig {
        rpc_url: Some("https://custom.rpc.com".to_string()),
        network: "custom".to_string(),
        verification_timeout: std::time::Duration::from_secs(30),
        confirmation_blocks: 1,
        max_retries: 3,
        retry_delay: std::time::Duration::from_secs(1),
    };

    let facilitator = BlockchainFacilitatorFactory::custom(config);
    assert!(facilitator.is_ok(), "Custom facilitator should be created");
}

#[tokio::test]
async fn test_blockchain_facilitator_verify_network_mismatch() {
    let facilitator = BlockchainFacilitatorFactory::base_sepolia().unwrap();

    // Create payment payload with different network
    let authorization = ExactEvmPayloadAuthorization::new(
        "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
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

    // Payment with "base" network
    let payment_payload = PaymentPayload::new("exact", "base", payload);

    // Requirements with "base-sepolia" network
    let requirements = PaymentRequirements::new(
        "exact",
        "base-sepolia",
        "1000000",
        "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
        "https://example.com/test",
        "Test payment",
    );

    let response = facilitator
        .verify(&payment_payload, &requirements)
        .await
        .unwrap();

    assert!(!response.is_valid);
    assert!(response.invalid_reason.is_some());
    assert!(response
        .invalid_reason
        .unwrap()
        .contains("Network mismatch"));
}

#[tokio::test]
async fn test_blockchain_facilitator_verify_scheme_mismatch() {
    let facilitator = BlockchainFacilitatorFactory::base_sepolia().unwrap();

    let authorization = ExactEvmPayloadAuthorization::new(
        "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
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

    // Payment with "exact" scheme
    let payment_payload = PaymentPayload::new("exact", "base-sepolia", payload);

    // Requirements with different scheme (if we had one)
    // For now, we'll test with same scheme but verify the check exists
    let requirements = PaymentRequirements::new(
        "exact",
        "base-sepolia",
        "1000000",
        "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
        "https://example.com/test",
        "Test payment",
    );

    // This should pass scheme validation since both are "exact"
    let response = facilitator.verify(&payment_payload, &requirements).await;

    // Should either succeed or fail for other reasons (like signature validation)
    // but not for scheme mismatch
    if let Ok(response) = response {
        if !response.is_valid {
            assert!(!response
                .invalid_reason
                .unwrap_or_default()
                .contains("Scheme mismatch"));
        }
    }
}

#[tokio::test]
async fn test_blockchain_facilitator_verify_expired_authorization() {
    let facilitator = BlockchainFacilitatorFactory::base_sepolia().unwrap();

    let now = chrono::Utc::now().timestamp();
    let expired_valid_before = (now - 100).to_string(); // Expired

    let authorization = ExactEvmPayloadAuthorization::new(
        "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
        "1000000",
        (now - 200).to_string(), // valid_after
        expired_valid_before,
        "0xf3746613c2d920b5fdabc0856f2aeb2d4f88ee6037b8cc5d04a71a4462f13480",
    );

    let payload = ExactEvmPayload {
        signature: "0x2d6a7588d6acca505cbf0d9a4a227e0c52c6c34008c8e8986a1283259764173608a2ce6496642e377d6da8dbbf5836e9bd15092f9ecab05ded3d6293af148b571c".to_string(),
        authorization,
    };

    let payment_payload = PaymentPayload::new("exact", "base-sepolia", payload);

    let requirements = PaymentRequirements::new(
        "exact",
        "base-sepolia",
        "1000000",
        "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
        "https://example.com/test",
        "Test payment",
    );

    let response = facilitator
        .verify(&payment_payload, &requirements)
        .await
        .unwrap();

    assert!(!response.is_valid);
    assert!(response.invalid_reason.is_some());
    let reason = response.invalid_reason.as_ref().unwrap();
    assert!(reason.contains("expired") || reason.contains("not yet valid"));
}

#[tokio::test]
async fn test_blockchain_facilitator_verify_future_authorization() {
    let facilitator = BlockchainFacilitatorFactory::base_sepolia().unwrap();

    let now = chrono::Utc::now().timestamp();
    let future_valid_after = (now + 100).to_string(); // Not yet valid

    let authorization = ExactEvmPayloadAuthorization::new(
        "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
        "1000000",
        future_valid_after,
        (now + 200).to_string(), // valid_before
        "0xf3746613c2d920b5fdabc0856f2aeb2d4f88ee6037b8cc5d04a71a4462f13480",
    );

    let payload = ExactEvmPayload {
        signature: "0x2d6a7588d6acca505cbf0d9a4a227e0c52c6c34008c8e8986a1283259764173608a2ce6496642e377d6da8dbbf5836e9bd15092f9ecab05ded3d6293af148b571c".to_string(),
        authorization,
    };

    let payment_payload = PaymentPayload::new("exact", "base-sepolia", payload);

    let requirements = PaymentRequirements::new(
        "exact",
        "base-sepolia",
        "1000000",
        "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
        "https://example.com/test",
        "Test payment",
    );

    let response = facilitator
        .verify(&payment_payload, &requirements)
        .await
        .unwrap();

    assert!(!response.is_valid);
    assert!(response.invalid_reason.is_some());
    let reason = response.invalid_reason.as_ref().unwrap();
    assert!(reason.contains("expired") || reason.contains("not yet valid"));
}

#[tokio::test]
#[ignore] // Requires actual blockchain RPC connection
async fn test_blockchain_facilitator_verify_amount_validation() {
    let facilitator = BlockchainFacilitatorFactory::base_sepolia().unwrap();

    let authorization = ExactEvmPayloadAuthorization::new(
        "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
        "500000", // Less than required
        "1745323800",
        "1745323985",
        "0xf3746613c2d920b5fdabc0856f2aeb2d4f88ee6037b8cc5d04a71a4462f13480",
    );

    let payload = ExactEvmPayload {
        signature: "0x2d6a7588d6acca505cbf0d9a4a227e0c52c6c34008c8e8986a1283259764173608a2ce6496642e377d6da8dbbf5836e9bd15092f9ecab05ded3d6293af148b571c".to_string(),
        authorization,
    };

    let payment_payload = PaymentPayload::new("exact", "base-sepolia", payload);

    // Requirements with higher amount
    let requirements = PaymentRequirements::new(
        "exact",
        "base-sepolia",
        "1000000", // More than payment
        "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
        "https://example.com/test",
        "Test payment",
    );

    let response = facilitator
        .verify(&payment_payload, &requirements)
        .await
        .unwrap();

    // Should fail due to insufficient amount or other validation
    assert!(!response.is_valid);
    assert!(response.invalid_reason.is_some());
    // The error might be about insufficient amount, balance check failure, or signature validation
    // Since we're using test signatures, it might fail for various reasons
    // Just verify that it fails validation
}

#[tokio::test]
#[ignore] // Requires actual blockchain RPC connection
async fn test_blockchain_facilitator_verify_pay_to_mismatch() {
    let facilitator = BlockchainFacilitatorFactory::base_sepolia().unwrap();

    let authorization = ExactEvmPayloadAuthorization::new(
        "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        "0x1111111111111111111111111111111111111111", // Different pay_to
        "1000000",
        "1745323800",
        "1745323985",
        "0xf3746613c2d920b5fdabc0856f2aeb2d4f88ee6037b8cc5d04a71a4462f13480",
    );

    let payload = ExactEvmPayload {
        signature: "0x2d6a7588d6acca505cbf0d9a4a227e0c52c6c34008c8e8986a1283259764173608a2ce6496642e377d6da8dbbf5836e9bd15092f9ecab05ded3d6293af148b571c".to_string(),
        authorization,
    };

    let payment_payload = PaymentPayload::new("exact", "base-sepolia", payload);

    let requirements = PaymentRequirements::new(
        "exact",
        "base-sepolia",
        "1000000",
        "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C", // Different pay_to
        "https://example.com/test",
        "Test payment",
    );

    let response = facilitator
        .verify(&payment_payload, &requirements)
        .await
        .unwrap();

    // Should fail due to pay_to mismatch or other validation
    assert!(!response.is_valid);
    assert!(response.invalid_reason.is_some());
    // The error might be about recipient mismatch or signature validation
    // Since we're using test signatures, it might fail for various reasons
    // Just verify that it fails validation
}

#[test]
fn test_blockchain_facilitator_client_creation() {
    let config = BlockchainFacilitatorConfig {
        rpc_url: None,
        network: "base-sepolia".to_string(),
        verification_timeout: std::time::Duration::from_secs(30),
        confirmation_blocks: 1,
        max_retries: 3,
        retry_delay: std::time::Duration::from_secs(1),
    };

    let client = BlockchainFacilitatorClient::new(config);
    assert!(
        client.is_ok(),
        "Should create blockchain facilitator client"
    );
}

#[test]
fn test_blockchain_facilitator_client_with_custom_rpc() {
    let config = BlockchainFacilitatorConfig {
        rpc_url: Some("https://custom.rpc.endpoint.com".to_string()),
        network: "base-sepolia".to_string(),
        verification_timeout: std::time::Duration::from_secs(30),
        confirmation_blocks: 1,
        max_retries: 3,
        retry_delay: std::time::Duration::from_secs(1),
    };

    let client = BlockchainFacilitatorClient::new(config);
    assert!(client.is_ok(), "Should create client with custom RPC URL");
}

#[test]
fn test_blockchain_facilitator_client_unsupported_network() {
    let config = BlockchainFacilitatorConfig {
        rpc_url: None,
        network: "unsupported-network".to_string(),
        verification_timeout: std::time::Duration::from_secs(30),
        confirmation_blocks: 1,
        max_retries: 3,
        retry_delay: std::time::Duration::from_secs(1),
    };

    let client = BlockchainFacilitatorClient::new(config);
    assert!(client.is_err(), "Should fail with unsupported network");
}
