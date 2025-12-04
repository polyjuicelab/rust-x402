//! Unit tests for facilitator request serialization/deserialization

use rust_x402::types::{PaymentPayload, PaymentRequirements};
use serde::Deserialize;
use serde_json::json;

/// Request types for the facilitator API (same as in facilitator.rs)
#[derive(Debug, Deserialize)]
struct VerifyRequest {
    #[serde(rename = "x402Version")]
    x402_version: u32,
    #[serde(rename = "paymentPayload")]
    payment_payload: PaymentPayload,
    #[serde(rename = "paymentRequirements")]
    #[allow(dead_code)] // Required for deserialization, even if not directly accessed
    payment_requirements: PaymentRequirements,
}

#[derive(Debug, Deserialize)]
struct SettleRequest {
    #[serde(rename = "x402Version")]
    x402_version: u32,
    #[serde(rename = "paymentPayload")]
    payment_payload: PaymentPayload,
    #[serde(rename = "paymentRequirements")]
    #[allow(dead_code)] // Required for deserialization, even if not directly accessed
    payment_requirements: PaymentRequirements,
}

#[test]
fn test_verify_request_deserialization_with_camelcase() {
    // Test that VerifyRequest can deserialize JSON with camelCase field names
    let json = json!({
        "x402Version": 1,
        "paymentPayload": {
            "x402Version": 1,
            "scheme": "exact",
            "network": "base-sepolia",
            "payload": {
                "signature": "0x123",
                "authorization": {
                    "from": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
                    "to": "0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d",
                    "value": "100",
                    "nonce": "0x123",
                    "validAfter": "1764754567",
                    "validBefore": "1764754927"
                }
            }
        },
        "paymentRequirements": {
            "asset": "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
            "network": "base-sepolia",
            "payTo": "0x209693bc6afc0c5328ba36faf03c514ef312287c",
            "scheme": "exact",
            "maxAmountRequired": "100",
            "description": "test",
            "resource": "/test",
            "maxTimeoutSeconds": 60
        }
    });

    let result: Result<VerifyRequest, _> = serde_json::from_value(json);
    assert!(
        result.is_ok(),
        "Failed to deserialize VerifyRequest: {:?}",
        result.err()
    );
    let request = result.unwrap();
    assert_eq!(request.x402_version, 1);
    assert_eq!(request.payment_payload.x402_version, 1);
    assert_eq!(request.payment_payload.scheme, "exact");
    assert_eq!(request.payment_payload.network, "base-sepolia");
}

#[test]
fn test_settle_request_deserialization_with_camelcase() {
    // Test that SettleRequest can deserialize JSON with camelCase field names
    let json = json!({
        "x402Version": 1,
        "paymentPayload": {
            "x402Version": 1,
            "scheme": "exact",
            "network": "base-sepolia",
            "payload": {
                "signature": "0x123",
                "authorization": {
                    "from": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
                    "to": "0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d",
                    "value": "100",
                    "nonce": "0x123",
                    "validAfter": "1764754567",
                    "validBefore": "1764754927"
                }
            }
        },
        "paymentRequirements": {
            "asset": "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
            "network": "base-sepolia",
            "payTo": "0x209693bc6afc0c5328ba36faf03c514ef312287c",
            "scheme": "exact",
            "maxAmountRequired": "100",
            "description": "test",
            "resource": "/test",
            "maxTimeoutSeconds": 60
        }
    });

    let result: Result<SettleRequest, _> = serde_json::from_value(json);
    assert!(
        result.is_ok(),
        "Failed to deserialize SettleRequest: {:?}",
        result.err()
    );
    let request = result.unwrap();
    assert_eq!(request.x402_version, 1);
    assert_eq!(request.payment_payload.x402_version, 1);
}

#[test]
fn test_verify_request_fails_with_snake_case() {
    // Test that VerifyRequest fails to deserialize JSON with snake_case field names
    let json = json!({
        "x402_version": 1,
        "payment_payload": {
            "x402_version": 1,
            "scheme": "exact",
            "network": "base-sepolia",
            "payload": {
                "signature": "0x123",
                "authorization": {
                    "from": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
                    "to": "0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d",
                    "value": "100",
                    "nonce": "0x123",
                    "validAfter": "1764754567",
                    "validBefore": "1764754927"
                }
            }
        },
        "payment_requirements": {
            "asset": "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
            "network": "base-sepolia",
            "payTo": "0x209693bc6afc0c5328ba36faf03c514ef312287c",
            "scheme": "exact",
            "maxAmountRequired": "100",
            "description": "test",
            "resource": "/test",
            "maxTimeoutSeconds": 60
        }
    });

    let result: Result<VerifyRequest, _> = serde_json::from_value(json);
    assert!(
        result.is_err(),
        "Should fail to deserialize with snake_case field names"
    );
}

#[test]
fn test_facilitator_client_serialization_format() {
    // Test that FacilitatorClient serializes PaymentPayload and PaymentRequirements correctly
    use rust_x402::types::{
        ExactEvmPayload, ExactEvmPayloadAuthorization, PaymentPayload, PaymentRequirements,
    };
    use serde_json::json;

    // Create a PaymentPayload
    let payload = PaymentPayload {
        x402_version: 1,
        scheme: "exact".to_string(),
        network: "base-sepolia".to_string(),
        payload: ExactEvmPayload {
            signature: "0x123".to_string(),
            authorization: ExactEvmPayloadAuthorization {
                from: "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".to_string(),
                to: "0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d".to_string(),
                value: "100".to_string(),
                nonce: "0x123".to_string(),
                valid_after: "1764754567".to_string(),
                valid_before: "1764754927".to_string(),
            },
        },
    };

    // Create PaymentRequirements
    let requirements = PaymentRequirements {
        scheme: "exact".to_string(),
        network: "base-sepolia".to_string(),
        max_amount_required: "100".to_string(),
        asset: "0x036CbD53842c5426634e7929541eC2318f3dCF7e".to_string(),
        pay_to: "0x209693bc6afc0c5328ba36faf03c514ef312287c".to_string(),
        resource: "/test".to_string(),
        description: "test".to_string(),
        mime_type: None,
        output_schema: None,
        max_timeout_seconds: 60,
        extra: None,
    };

    // Serialize using serde_json::to_value (same as FacilitatorClient does)
    let payment_payload_value =
        serde_json::to_value(&payload).expect("Failed to serialize PaymentPayload");
    let payment_requirements_value =
        serde_json::to_value(&requirements).expect("Failed to serialize PaymentRequirements");

    // Build request body (same as FacilitatorClient does)
    let request_body = json!({
        "x402Version": 1,
        "paymentPayload": payment_payload_value,
        "paymentRequirements": payment_requirements_value,
    });

    // Verify that the serialized PaymentPayload has x402Version (camelCase)
    let payment_payload_obj = request_body
        .get("paymentPayload")
        .expect("paymentPayload should exist");
    assert!(
        payment_payload_obj.get("x402Version").is_some(),
        "PaymentPayload should have x402Version field (camelCase), got: {:?}",
        payment_payload_obj
    );
    assert!(
        payment_payload_obj.get("x402_version").is_none(),
        "PaymentPayload should NOT have x402_version field (snake_case)"
    );

    // Verify that the request body can be deserialized by VerifyRequest
    let result: Result<VerifyRequest, _> = serde_json::from_value(request_body);
    assert!(
        result.is_ok(),
        "Failed to deserialize request body created by FacilitatorClient: {:?}",
        result.err()
    );
    let request = result.unwrap();
    assert_eq!(request.x402_version, 1);
    assert_eq!(request.payment_payload.x402_version, 1);
}
