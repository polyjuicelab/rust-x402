//! Integration test backend server for x402 payments

use axum::{response::Json, routing::get};
use rust_decimal::Decimal;
use serde_json::json;
use std::env;
use std::str::FromStr;

use rust_x402::{
    axum::{create_payment_app, examples, AxumPaymentConfig},
    types::FacilitatorConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Get configuration from environment variables
    let facilitator_url =
        env::var("FACILITATOR_URL").unwrap_or_else(|_| "http://localhost:4020".to_string());
    let port = env::var("PORT")
        .unwrap_or_else(|_| "4021".to_string())
        .parse::<u16>()?;

    // Create facilitator configuration
    let facilitator_config = FacilitatorConfig::new(facilitator_url);

    // Create payment configuration
    let payment_config = AxumPaymentConfig::new(
        Decimal::from_str("0.0001")?, // 0.0001 USDC (1/10th of a cent)
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C", // Recipient address
    )
    .with_description("Integration test API access")
    .with_mime_type("application/json")
    .with_facilitator_config(facilitator_config)
    .with_testnet(true)
    .with_tracing()
    .with_cors(vec![
        "http://localhost:3000".to_string(),
        "http://frontend:3000".to_string(),
    ]);

    // Create the application
    let app = create_payment_app(payment_config, |router| {
        router
            .route("/joke", get(examples::joke_handler))
            .route("/api/data", get(examples::api_data_handler))
            .route("/download", get(examples::download_handler))
            .route("/health", get(health_handler))
            .route("/test", get(test_handler))
    });

    // Start the server
    let bind_address = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&bind_address).await?;
    println!(
        "🚀 Integration test backend running on http://{}",
        bind_address
    );
    println!("💰 Protected endpoints:");
    println!("   GET /joke - Premium joke API");
    println!("   GET /api/data - Premium data API");
    println!("   GET /download - Premium file download");
    println!("   GET /test - Test endpoint");
    println!("   GET /health - Health check (free)");

    axum::serve(listener, app).await?;

    Ok(())
}

/// Health check handler (no payment required)
async fn health_handler() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "x402-integration-test-backend",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Test endpoint handler (payment required)
async fn test_handler() -> Json<serde_json::Value> {
    Json(json!({
        "message": "Payment successful! This is a protected endpoint.",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "data": {
            "test": true,
            "integration": "docker"
        }
    }))
}
