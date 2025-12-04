//! X402 Facilitator Server
//!
//! A standalone facilitator server for verifying and settling x402 micropayments.
//! This server can be run as a binary for production deployment.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

use rand::Rng;
use rust_x402::{types::*, Result, X402Error};

/// Simple in-memory facilitator for demonstration
#[derive(Debug, Clone)]
struct SimpleFacilitator {
    /// Track processed nonces to prevent replay attacks
    processed_nonces: Arc<RwLock<HashMap<String, bool>>>,
}

impl SimpleFacilitator {
    fn new() -> Self {
        Self {
            processed_nonces: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Verify a payment payload
    async fn verify_payment(
        &self,
        payload: &PaymentPayload,
        requirements: &PaymentRequirements,
    ) -> Result<VerifyResponse> {
        // Check if nonce has been used before (replay protection)
        let nonce = &payload.payload.authorization.nonce;
        {
            let nonces = self.processed_nonces.read().await;
            if nonces.contains_key(nonce) {
                return Ok(VerifyResponse {
                    is_valid: false,
                    invalid_reason: Some("nonce_already_used".to_string()),
                    payer: Some(payload.payload.authorization.from.clone()),
                });
            }
        }

        // Verify authorization timing
        if !payload.payload.authorization.is_valid_now()? {
            return Ok(VerifyResponse {
                is_valid: false,
                invalid_reason: Some("authorization_expired".to_string()),
                payer: Some(payload.payload.authorization.from.clone()),
            });
        }

        // Verify amount meets requirements
        let payment_amount: u128 = payload
            .payload
            .authorization
            .value
            .parse()
            .map_err(|_| X402Error::invalid_payment_requirements("Invalid payment amount"))?;
        let required_amount: u128 = requirements
            .max_amount_required
            .parse()
            .map_err(|_| X402Error::invalid_payment_requirements("Invalid required amount"))?;

        if payment_amount < required_amount {
            return Ok(VerifyResponse {
                is_valid: false,
                invalid_reason: Some("insufficient_amount".to_string()),
                payer: Some(payload.payload.authorization.from.clone()),
            });
        }

        // Verify recipient matches
        if payload.payload.authorization.to != requirements.pay_to {
            return Ok(VerifyResponse {
                is_valid: false,
                invalid_reason: Some("recipient_mismatch".to_string()),
                payer: Some(payload.payload.authorization.from.clone()),
            });
        }

        // Mark nonce as processed
        {
            let mut nonces = self.processed_nonces.write().await;
            nonces.insert(nonce.clone(), true);
        }

        Ok(VerifyResponse {
            is_valid: true,
            invalid_reason: None,
            payer: Some(payload.payload.authorization.from.clone()),
        })
    }

    /// Settle a verified payment
    async fn settle_payment(
        &self,
        payload: &PaymentPayload,
        _requirements: &PaymentRequirements,
    ) -> Result<SettleResponse> {
        // In a real implementation, this would:
        // 1. Call the blockchain to execute the transfer
        // 2. Wait for transaction confirmation
        // 3. Return the transaction hash

        // For this example, we'll simulate a successful settlement
        let mock_transaction_hash = format!("0x{:064x}", rand::thread_rng().gen::<u128>());

        Ok(SettleResponse {
            success: true,
            error_reason: None,
            transaction: mock_transaction_hash,
            network: payload.network.clone(),
            payer: Some(payload.payload.authorization.from.clone()),
        })
    }
}

/// Request types for the facilitator API
#[derive(Debug, Deserialize)]
struct VerifyRequest {
    #[serde(rename = "x402Version")]
    x402_version: u32,
    #[serde(rename = "paymentPayload")]
    payment_payload: PaymentPayload,
    #[serde(rename = "paymentRequirements")]
    payment_requirements: PaymentRequirements,
}

#[derive(Debug, Deserialize)]
struct SettleRequest {
    #[serde(rename = "x402Version")]
    x402_version: u32,
    #[serde(rename = "paymentPayload")]
    payment_payload: PaymentPayload,
    #[serde(rename = "paymentRequirements")]
    payment_requirements: PaymentRequirements,
}

/// Supported networks query
#[derive(Debug, Deserialize)]
struct SupportedQuery {
    #[serde(default)]
    #[allow(dead_code)]
    format: Option<String>,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Get bind address from environment or use default
    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    // Create facilitator instance
    let facilitator = SimpleFacilitator::new();

    // Create the API routes
    let app = Router::new()
        .route("/verify", post(verify_handler))
        .route("/settle", post(settle_handler))
        .route("/supported", get(supported_handler))
        .route("/health", get(health_handler))
        .with_state(facilitator);

    // Start the server
    let listener = tokio::net::TcpListener::bind(&bind_address).await?;
    println!(
        "🔧 X402 Facilitator server running on http://{}",
        bind_address
    );
    println!("📋 Available endpoints:");
    println!("   POST /verify - Verify payment authorization");
    println!("   POST /settle - Settle verified payment");
    println!("   GET /supported - Get supported payment schemes");
    println!("   GET /health - Health check endpoint");
    println!("\nEnvironment variables:");
    println!("   BIND_ADDRESS - Server bind address (default: 0.0.0.0:3000)");

    axum::serve(listener, app).await?;

    Ok(())
}

/// Handle payment verification requests
async fn verify_handler(
    State(facilitator): State<SimpleFacilitator>,
    Json(request): Json<VerifyRequest>,
) -> std::result::Result<Json<VerifyResponse>, StatusCode> {
    tracing::debug!("Received verify request: x402_version = {}", request.x402_version);
    if request.x402_version != X402_VERSION {
        return Err(StatusCode::BAD_REQUEST);
    }

    match facilitator
        .verify_payment(&request.payment_payload, &request.payment_requirements)
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            eprintln!("Verification error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Handle payment settlement requests
async fn settle_handler(
    State(facilitator): State<SimpleFacilitator>,
    Json(request): Json<SettleRequest>,
) -> std::result::Result<Json<SettleResponse>, StatusCode> {
    if request.x402_version != X402_VERSION {
        return Err(StatusCode::BAD_REQUEST);
    }

    match facilitator
        .settle_payment(&request.payment_payload, &request.payment_requirements)
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            eprintln!("Settlement error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Handle supported payment schemes requests
async fn supported_handler(Query(_query): Query<SupportedQuery>) -> Json<SupportedKinds> {
    Json(SupportedKinds {
        kinds: vec![
            SupportedKind {
                x402_version: X402_VERSION,
                scheme: schemes::EXACT.to_string(),
                network: networks::BASE_SEPOLIA.to_string(),
                metadata: None,
            },
            SupportedKind {
                x402_version: X402_VERSION,
                scheme: schemes::EXACT.to_string(),
                network: networks::BASE_MAINNET.to_string(),
                metadata: None,
            },
            SupportedKind {
                x402_version: X402_VERSION,
                scheme: schemes::EXACT.to_string(),
                network: networks::AVALANCHE_FUJI.to_string(),
                metadata: None,
            },
            SupportedKind {
                x402_version: X402_VERSION,
                scheme: schemes::EXACT.to_string(),
                network: networks::AVALANCHE_MAINNET.to_string(),
                metadata: None,
            },
        ],
    })
}

/// Health check endpoint
async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "version": rust_x402::VERSION,
        "x402_version": X402_VERSION,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
}
