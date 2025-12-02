//! Docker-based integration tests for x402
//!
//! These tests require Docker and docker-compose to be running.
//! Run with: `cargo test --test docker_integration_test --features axum,redis`
//!
//! Prerequisites:
//! - Docker and docker-compose installed
//! - Services started: `docker-compose up -d`
//! - Services healthy and ready

use base64::Engine;
use rust_x402::{
    client::X402Client,
    types::{PaymentRequirements, PaymentRequirementsResponse, SettleResponse},
    wallet::WalletFactory,
    Result,
};
use std::time::{Duration, Instant};
use tokio::time::sleep;

// Test configuration
const BACKEND_URL: &str = "http://localhost:4021";
const FACILITATOR_URL: &str = "http://localhost:4020";
const ANVIL_URL: &str = "http://localhost:8545";

// Test account from Anvil (first pre-funded account)
const TEST_PRIVATE_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
const TEST_PAYER_ADDRESS: &str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

/// Custom test error for better error messages
#[derive(Debug)]
struct TestError {
    message: String,
    context: Option<String>,
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref ctx) = self.context {
            write!(f, "{} (Context: {})", self.message, ctx)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for TestError {}

/// Wait for a service to be ready by checking its health endpoint
async fn wait_for_service(url: &str, max_wait: Duration) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .tcp_keepalive(Duration::from_secs(30))
        .build()
        .map_err(|e| {
            rust_x402::X402Error::network_error(format!("Failed to create client: {}", e))
        })?;

    let start = Instant::now();
    let mut last_error = None;
    let mut last_status = None;
    let mut attempt = 0;

    while start.elapsed() < max_wait {
        attempt += 1;
        match client.get(url).send().await {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    return Ok(());
                }
                last_status = Some(status.as_u16());
                // If we get 503, the service might be starting up, wait longer
                if status == 503 && attempt < 10 {
                    sleep(Duration::from_millis(2000)).await;
                    continue;
                }
            }
            Err(e) => {
                last_error = Some(e.to_string());
                // For connection errors, wait a bit longer before retrying
                if attempt < 10 {
                    sleep(Duration::from_millis(2000)).await;
                    continue;
                }
            }
        }
        sleep(Duration::from_millis(1000)).await;
    }

    Err(rust_x402::X402Error::network_error(format!(
        "Service at {} did not become ready within {:?} (attempted {} times). Last status: {:?}, Last error: {:?}",
        url, max_wait, attempt, last_status, last_error
    )))
}

/// Create a test payment payload using wallet integration
fn create_test_payment_payload(
    requirements: &PaymentRequirements,
) -> Result<rust_x402::types::PaymentPayload> {
    let wallet =
        WalletFactory::from_private_key(TEST_PRIVATE_KEY, &requirements.network).map_err(|e| {
            rust_x402::X402Error::invalid_payment_payload(format!("Failed to create wallet: {}", e))
        })?;

    wallet
        .create_signed_payment_payload(requirements, TEST_PAYER_ADDRESS)
        .map_err(|e| {
            rust_x402::X402Error::invalid_payment_payload(format!(
                "Failed to create payment payload: {}",
                e
            ))
        })
}

/// Assert that a response has status 402 and contains valid payment requirements
async fn assert_payment_required(
    response: reqwest::Response,
    endpoint: &str,
) -> Result<PaymentRequirementsResponse> {
    let status = response.status();

    if status != 402 {
        return Err(rust_x402::X402Error::invalid_payment_requirements(format!(
            "Expected 402 Payment Required for endpoint {}, but got status {}",
            endpoint, status
        )));
    }

    let payment_req: PaymentRequirementsResponse = response.json().await.map_err(|e| {
        rust_x402::X402Error::invalid_payment_requirements(format!(
            "Failed to parse payment requirements JSON for endpoint {}: {}",
            endpoint, e
        ))
    })?;

    // Validate payment requirements structure
    if payment_req.x402_version != 1 {
        return Err(rust_x402::X402Error::invalid_payment_requirements(format!(
            "Invalid x402 version: expected 1, got {}",
            payment_req.x402_version
        )));
    }

    if payment_req.accepts.is_empty() {
        return Err(rust_x402::X402Error::invalid_payment_requirements(
            "Payment requirements must include at least one accept option".to_string(),
        ));
    }

    // Validate first accept option
    let req = &payment_req.accepts[0];
    if req.scheme != "exact" {
        return Err(rust_x402::X402Error::invalid_payment_requirements(format!(
            "Invalid payment scheme: expected 'exact', got '{}'",
            req.scheme
        )));
    }

    if req.network != "base-sepolia" {
        return Err(rust_x402::X402Error::invalid_payment_requirements(format!(
            "Invalid network: expected 'base-sepolia', got '{}'",
            req.network
        )));
    }

    if req.pay_to.is_empty() {
        return Err(rust_x402::X402Error::invalid_payment_requirements(
            "Payment requirements must include pay_to address".to_string(),
        ));
    }

    if req.max_amount_required.is_empty() {
        return Err(rust_x402::X402Error::invalid_payment_requirements(
            "Payment requirements must include max_amount_required".to_string(),
        ));
    }

    Ok(payment_req)
}

/// Assert that a payment response is successful and contains settlement information
async fn assert_payment_success(
    response: reqwest::Response,
    endpoint: &str,
) -> Result<serde_json::Value> {
    let status = response.status();

    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(rust_x402::X402Error::invalid_payment_payload(format!(
            "Expected successful response (2xx) for endpoint {} after payment, but got status {} with body: {}",
            endpoint, status, body
        )));
    }

    // Verify X-PAYMENT-RESPONSE header exists (must check before consuming response)
    let settlement_header = response
        .headers()
        .get("X-PAYMENT-RESPONSE")
        .ok_or_else(|| {
            rust_x402::X402Error::invalid_payment_payload(format!(
                "Missing X-PAYMENT-RESPONSE header in response for endpoint {}",
                endpoint
            ))
        })?
        .clone();

    // Decode and verify settlement response
    let settlement_b64 = settlement_header.to_str().map_err(|e| {
        rust_x402::X402Error::invalid_payment_payload(format!(
            "Invalid X-PAYMENT-RESPONSE header encoding for endpoint {}: {}",
            endpoint, e
        ))
    })?;

    let settlement_bytes = base64::engine::general_purpose::STANDARD
        .decode(settlement_b64)
        .map_err(|e| {
            rust_x402::X402Error::invalid_payment_payload(format!(
                "Failed to decode X-PAYMENT-RESPONSE header for endpoint {}: {}",
                endpoint, e
            ))
        })?;

    let settlement: SettleResponse = serde_json::from_slice(&settlement_bytes).map_err(|e| {
        rust_x402::X402Error::invalid_payment_payload(format!(
            "Failed to parse settlement response JSON for endpoint {}: {}",
            endpoint, e
        ))
    })?;

    // Verify settlement was successful
    if !settlement.success {
        return Err(rust_x402::X402Error::invalid_payment_payload(format!(
            "Payment settlement failed for endpoint {}: {}",
            endpoint,
            settlement
                .error_reason
                .as_deref()
                .unwrap_or("Unknown error")
        )));
    }

    // Verify transaction hash is present and valid format
    if settlement.transaction.is_empty() {
        return Err(rust_x402::X402Error::invalid_payment_payload(format!(
            "Settlement response missing transaction hash for endpoint {}",
            endpoint
        )));
    }

    if !settlement.transaction.starts_with("0x") || settlement.transaction.len() != 66 {
        return Err(rust_x402::X402Error::invalid_payment_payload(format!(
            "Invalid transaction hash format for endpoint {}: expected 0x-prefixed 64-char hex string, got: {}",
            endpoint, settlement.transaction
        )));
    }

    // Parse response body
    let data: serde_json::Value = response.json().await.map_err(|e| {
        rust_x402::X402Error::invalid_payment_payload(format!(
            "Failed to parse response JSON for endpoint {}: {}",
            endpoint, e
        ))
    })?;

    Ok(data)
}

#[tokio::test]
#[ignore] // Ignore by default - requires Docker services
async fn test_services_health() {
    // Services should already be running, but wait a bit for them to be ready
    sleep(Duration::from_secs(2)).await;

    // Test backend health endpoint
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Client creation should succeed");

    // Retry a few times in case service is still starting
    let mut response = None;
    for attempt in 0..10 {
        match client.get(&format!("{}/health", BACKEND_URL)).send().await {
            Ok(resp) if resp.status().is_success() => {
                response = Some(resp);
                break;
            }
            Ok(resp) => {
                response = Some(resp);
                if attempt < 9 {
                    sleep(Duration::from_millis(500)).await;
                }
            }
            Err(e) => {
                if attempt < 9 {
                    eprintln!("Attempt {} failed: {}, retrying...", attempt + 1, e);
                    sleep(Duration::from_millis(500)).await;
                } else {
                    panic!("Failed to connect to backend after 10 attempts: {}", e);
                }
            }
        }
    }

    let response = response.expect("Health check request should succeed");

    if response.status() != 200 {
        panic!(
            "Backend health endpoint should return 200, but got {} with body: {:?}",
            response.status(),
            response.text().await.ok()
        );
    }

    let health: serde_json::Value = response
        .json()
        .await
        .expect("Health response should be JSON");

    if health["status"] != "healthy" {
        panic!(
            "Backend health status should be 'healthy', but got: {:?}",
            health["status"]
        );
    }

    // Test facilitator health endpoint
    let mut facilitator_response = None;
    for attempt in 0..10 {
        match client
            .get(&format!("{}/health", FACILITATOR_URL))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                facilitator_response = Some(resp);
                break;
            }
            Ok(resp) => {
                facilitator_response = Some(resp);
                if attempt < 9 {
                    sleep(Duration::from_millis(500)).await;
                }
            }
            Err(e) => {
                if attempt < 9 {
                    eprintln!("Attempt {} failed: {}, retrying...", attempt + 1, e);
                    sleep(Duration::from_millis(500)).await;
                } else {
                    panic!("Failed to connect to facilitator after 10 attempts: {}", e);
                }
            }
        }
    }

    let response = facilitator_response.expect("Facilitator health check request should succeed");

    if response.status() != 200 {
        panic!(
            "Facilitator health endpoint should return 200, but got {} with body: {:?}",
            response.status(),
            response.text().await.ok()
        );
    }

    let health: serde_json::Value = response
        .json()
        .await
        .expect("Health response should be JSON");

    if health["status"] != "healthy" {
        panic!(
            "Facilitator health status should be 'healthy', but got: {:?}",
            health["status"]
        );
    }
}

#[tokio::test]
#[ignore] // Ignore by default - requires Docker services
async fn test_payment_required_response() {
    wait_for_service(&format!("{}/health", BACKEND_URL), Duration::from_secs(60))
        .await
        .expect("Backend should be healthy");

    let client = X402Client::new().expect("Client creation should succeed");
    let response = client
        .get(&format!("{}/test", BACKEND_URL))
        .send()
        .await
        .expect("Request should succeed");

    // Assert payment required with detailed error messages
    let payment_req_response = assert_payment_required(response, "/test")
        .await
        .expect("Should receive valid payment requirements");

    // Additional validations
    let req = &payment_req_response.accepts[0];
    assert_eq!(
        req.scheme, "exact",
        "Payment scheme should be 'exact', got '{}'",
        req.scheme
    );
    assert_eq!(
        req.network, "base-sepolia",
        "Network should be 'base-sepolia', got '{}'",
        req.network
    );
}

#[tokio::test]
#[ignore] // Ignore by default - requires Docker services
async fn test_end_to_end_payment_flow() {
    wait_for_service(&format!("{}/health", BACKEND_URL), Duration::from_secs(60))
        .await
        .expect("Backend should be healthy");

    wait_for_service(
        &format!("{}/health", FACILITATOR_URL),
        Duration::from_secs(60),
    )
    .await
    .expect("Facilitator should be healthy");

    let client = X402Client::new().expect("Client creation should succeed");
    let endpoint = "/test";

    // Step 1: Make request without payment (should get 402)
    let response = client
        .get(&format!("{}{}", BACKEND_URL, endpoint))
        .send()
        .await
        .expect("Initial request should succeed");

    if response.status() != 402 {
        panic!(
            "First request to {} should return 402 Payment Required, but got status {}",
            endpoint,
            response.status()
        );
    }

    // Step 2: Parse and validate payment requirements
    let payment_req_response = assert_payment_required(response, endpoint)
        .await
        .expect("Should receive valid payment requirements");

    let payment_req = &payment_req_response.accepts[0];

    // Step 3: Create payment payload
    let payment_payload =
        create_test_payment_payload(payment_req).expect("Payment payload creation should succeed");

    // Step 4: Retry request with payment
    let final_response = client
        .get(&format!("{}{}", BACKEND_URL, endpoint))
        .payment(&payment_payload)
        .expect("Payment header creation should succeed")
        .send()
        .await
        .expect("Payment request should succeed");

    // Step 5: Verify success and settlement
    let data = assert_payment_success(final_response, endpoint)
        .await
        .expect("Payment should be successful");

    // Step 6: Verify response content
    if data["message"] != "Payment successful! This is a protected endpoint." {
        panic!(
            "Response message should match expected value. Got: {:?}",
            data["message"]
        );
    }

    // Verify response structure
    assert!(
        data["timestamp"].is_string(),
        "Response should include timestamp field"
    );
    assert!(
        data["data"].is_object(),
        "Response should include data object"
    );
}

#[tokio::test]
#[ignore] // Ignore by default - requires Docker services
async fn test_health_endpoint_no_payment() {
    wait_for_service(&format!("{}/health", BACKEND_URL), Duration::from_secs(60))
        .await
        .expect("Backend should be healthy");

    let client = X402Client::new().expect("Client creation should succeed");
    let response = client
        .get(&format!("{}/health", BACKEND_URL))
        .send()
        .await
        .expect("Health check request should succeed");

    if response.status() != 200 {
        panic!(
            "Health endpoint should not require payment (expected 200), but got status {}",
            response.status()
        );
    }

    let health: serde_json::Value = response
        .json()
        .await
        .expect("Health response should be JSON");

    if health["status"] != "healthy" {
        panic!(
            "Health status should be 'healthy', but got: {:?}",
            health["status"]
        );
    }
}

#[tokio::test]
#[ignore] // Ignore by default - requires Docker services
async fn test_multiple_endpoints() {
    wait_for_service(&format!("{}/health", BACKEND_URL), Duration::from_secs(60))
        .await
        .expect("Backend should be healthy");

    let client = X402Client::new().expect("Client creation should succeed");

    // Test multiple protected endpoints
    let endpoints = vec!["/joke", "/api/data", "/test"];

    for endpoint in endpoints {
        // First request should return 402
        let response = client
            .get(&format!("{}{}", BACKEND_URL, endpoint))
            .send()
            .await
            .expect(&format!("Request to {} should succeed", endpoint));

        if response.status() != 402 {
            panic!(
                "Endpoint {} should return 402 Payment Required, but got status {}",
                endpoint,
                response.status()
            );
        }

        // Parse payment requirements
        let payment_req_response =
            assert_payment_required(response, endpoint)
                .await
                .expect(&format!(
                    "Should receive valid payment requirements for {}",
                    endpoint
                ));

        let payment_req = &payment_req_response.accepts[0];

        // Create payment payload
        let payment_payload = create_test_payment_payload(payment_req).expect(&format!(
            "Payment payload creation should succeed for {}",
            endpoint
        ));

        // Retry with payment
        let final_response = client
            .get(&format!("{}{}", BACKEND_URL, endpoint))
            .payment(&payment_payload)
            .expect(&format!(
                "Payment header creation should succeed for {}",
                endpoint
            ))
            .send()
            .await
            .expect(&format!("Payment request should succeed for {}", endpoint));

        // Verify success
        assert_payment_success(final_response, endpoint)
            .await
            .expect(&format!("Payment should be successful for {}", endpoint));
    }
}

#[tokio::test]
#[ignore] // Ignore by default - requires Docker services
async fn test_facilitator_supported_endpoint() {
    wait_for_service(
        &format!("{}/health", FACILITATOR_URL),
        Duration::from_secs(60),
    )
    .await
    .expect("Facilitator should be healthy");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Client creation should succeed");

    let response = client
        .get(&format!("{}/supported", FACILITATOR_URL))
        .send()
        .await
        .expect("Supported endpoint request should succeed");

    if response.status() != 200 {
        panic!(
            "Facilitator supported endpoint should return 200, but got status {}",
            response.status()
        );
    }

    let supported: serde_json::Value = response
        .json()
        .await
        .expect("Supported response should be JSON");

    if !supported["kinds"].is_array() {
        panic!(
            "Supported response should include 'kinds' array, but got: {:?}",
            supported["kinds"]
        );
    }

    let kinds = supported["kinds"].as_array().unwrap();
    if kinds.is_empty() {
        panic!("Facilitator should support at least one payment kind");
    }

    // Verify at least one kind supports base-sepolia
    let has_base_sepolia = kinds
        .iter()
        .any(|k| k["network"].as_str() == Some("base-sepolia"));

    if !has_base_sepolia {
        panic!("Facilitator should support base-sepolia network");
    }
}
