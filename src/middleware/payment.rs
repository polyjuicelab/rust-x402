//! Payment middleware implementation

use super::config::PaymentMiddlewareConfig;
use crate::types::{
    PaymentPayload, PaymentRequirements, PaymentRequirementsResponse, SettleResponse,
};
use crate::{Result, X402Error};
use axum::{
    extract::{Request, State},
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;

/// Axum middleware for x402 payments
#[derive(Debug, Clone)]
pub struct PaymentMiddleware {
    pub config: Arc<PaymentMiddlewareConfig>,
    pub facilitator: Option<crate::facilitator::FacilitatorClient>,
    pub template_config: Option<crate::template::PaywallConfig>,
}

/// Payment processing result
#[derive(Debug)]
pub enum PaymentResult {
    /// Payment verified and settled successfully
    Success {
        response: axum::response::Response,
        settlement: SettleResponse,
    },
    /// Payment required (402 response)
    PaymentRequired { response: axum::response::Response },
    /// Payment verification failed
    VerificationFailed { response: axum::response::Response },
    /// Payment settlement failed
    SettlementFailed { response: axum::response::Response },
}

impl PaymentMiddleware {
    /// Create a new payment middleware
    pub fn new(amount: rust_decimal::Decimal, pay_to: impl Into<String>) -> Self {
        Self {
            config: Arc::new(PaymentMiddlewareConfig::new(amount, pay_to)),
            facilitator: None,
            template_config: None,
        }
    }

    /// Set the payment description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        Arc::make_mut(&mut self.config).description = Some(description.into());
        self
    }

    /// Set the MIME type
    pub fn with_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        Arc::make_mut(&mut self.config).mime_type = Some(mime_type.into());
        self
    }

    /// Set the maximum timeout
    pub fn with_max_timeout_seconds(mut self, max_timeout_seconds: u32) -> Self {
        Arc::make_mut(&mut self.config).max_timeout_seconds = max_timeout_seconds;
        self
    }

    /// Set the output schema
    pub fn with_output_schema(mut self, output_schema: serde_json::Value) -> Self {
        Arc::make_mut(&mut self.config).output_schema = Some(output_schema);
        self
    }

    /// Set the facilitator configuration
    pub fn with_facilitator_config(
        mut self,
        facilitator_config: crate::types::FacilitatorConfig,
    ) -> Self {
        Arc::make_mut(&mut self.config).facilitator_config = facilitator_config;
        self
    }

    /// Set whether this is a testnet
    pub fn with_testnet(mut self, testnet: bool) -> Self {
        Arc::make_mut(&mut self.config).testnet = testnet;
        self
    }

    /// Set custom paywall HTML
    pub fn with_custom_paywall_html(mut self, html: impl Into<String>) -> Self {
        Arc::make_mut(&mut self.config).custom_paywall_html = Some(html.into());
        self
    }

    /// Set the resource URL
    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        Arc::make_mut(&mut self.config).resource = Some(resource.into());
        self
    }

    /// Set the resource root URL
    pub fn with_resource_root_url(mut self, url: impl Into<String>) -> Self {
        Arc::make_mut(&mut self.config).resource_root_url = Some(url.into());
        self
    }

    /// Get the middleware configuration
    pub fn config(&self) -> &PaymentMiddlewareConfig {
        &self.config
    }

    /// Set the facilitator client
    pub fn with_facilitator(mut self, facilitator: crate::facilitator::FacilitatorClient) -> Self {
        self.facilitator = Some(facilitator);
        self
    }

    /// Set the template configuration
    pub fn with_template_config(mut self, template_config: crate::template::PaywallConfig) -> Self {
        self.template_config = Some(template_config);
        self
    }

    /// Verify a payment payload
    pub async fn verify(&self, payment_payload: &PaymentPayload) -> bool {
        // Create facilitator if not already configured
        let facilitator = if let Some(facilitator) = &self.facilitator {
            facilitator.clone()
        } else {
            match crate::facilitator::FacilitatorClient::new(self.config.facilitator_config.clone())
            {
                Ok(facilitator) => facilitator,
                Err(_) => return false,
            }
        };

        if let Ok(requirements) = self.config.create_payment_requirements("/") {
            if let Ok(response) = facilitator.verify(payment_payload, &requirements).await {
                return response.is_valid;
            }
        }
        false
    }

    /// Settle a payment
    pub async fn settle(&self, payment_payload: &PaymentPayload) -> Result<SettleResponse> {
        // Create facilitator if not already configured
        let facilitator = if let Some(facilitator) = &self.facilitator {
            facilitator.clone()
        } else {
            crate::facilitator::FacilitatorClient::new(self.config.facilitator_config.clone())?
        };

        let requirements = self.config.create_payment_requirements("/")?;
        facilitator.settle(payment_payload, &requirements).await
    }

    /// Verify payment with specific requirements
    pub async fn verify_with_requirements(
        &self,
        payment_payload: &PaymentPayload,
        requirements: &PaymentRequirements,
    ) -> Result<bool> {
        let facilitator = if let Some(facilitator) = &self.facilitator {
            facilitator.clone()
        } else {
            crate::facilitator::FacilitatorClient::new(self.config.facilitator_config.clone())?
        };

        let response = facilitator.verify(payment_payload, requirements).await?;
        Ok(response.is_valid)
    }

    /// Settle payment with specific requirements
    pub async fn settle_with_requirements(
        &self,
        payment_payload: &PaymentPayload,
        requirements: &PaymentRequirements,
    ) -> Result<SettleResponse> {
        let facilitator = if let Some(facilitator) = &self.facilitator {
            facilitator.clone()
        } else {
            crate::facilitator::FacilitatorClient::new(self.config.facilitator_config.clone())?
        };

        facilitator.settle(payment_payload, requirements).await
    }

    /// Process payment with unified flow
    pub async fn process_payment(&self, request: Request, next: Next) -> Result<PaymentResult> {
        let headers = request.headers();
        let uri = request.uri().to_string();

        // Check if this is a web browser request
        let user_agent = headers
            .get("User-Agent")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let accept = headers
            .get("Accept")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let is_web_browser = accept.contains("text/html") && user_agent.contains("Mozilla");

        // Create payment requirements
        let payment_requirements = self.config.create_payment_requirements(&uri)?;

        // Check for payment header
        let payment_header = headers.get("X-PAYMENT").and_then(|v| v.to_str().ok());

        match payment_header {
            Some(payment_b64) => {
                // Decode payment payload
                let payment_payload = PaymentPayload::from_base64(payment_b64).map_err(|e| {
                    X402Error::invalid_payment_payload(format!("Failed to decode payment: {}", e))
                })?;

                // Get facilitator client
                let facilitator = if let Some(facilitator) = &self.facilitator {
                    facilitator.clone()
                } else {
                    crate::facilitator::FacilitatorClient::new(
                        self.config.facilitator_config.clone(),
                    )?
                };

                // Verify payment
                let verify_response = facilitator
                    .verify(&payment_payload, &payment_requirements)
                    .await
                    .map_err(|e| {
                        X402Error::facilitator_error(format!("Payment verification failed: {}", e))
                    })?;

                if !verify_response.is_valid {
                    let error_response = self.create_payment_required_response(
                        "Payment verification failed",
                        &payment_requirements,
                        is_web_browser,
                    )?;
                    return Ok(PaymentResult::VerificationFailed {
                        response: error_response,
                    });
                }

                // Execute the handler
                let mut response = next.run(request).await;

                // Settle the payment
                let settle_response = facilitator
                    .settle(&payment_payload, &payment_requirements)
                    .await
                    .map_err(|e| {
                        X402Error::facilitator_error(format!("Payment settlement failed: {}", e))
                    })?;

                // Add settlement header
                let settlement_header = settle_response.to_base64().map_err(|e| {
                    X402Error::config(format!("Failed to encode settlement response: {}", e))
                })?;

                if let Ok(header_value) = HeaderValue::from_str(&settlement_header) {
                    response
                        .headers_mut()
                        .insert("X-PAYMENT-RESPONSE", header_value);
                }

                Ok(PaymentResult::Success {
                    response,
                    settlement: settle_response,
                })
            }
            None => {
                // No payment provided, return 402 with requirements
                let response = self.create_payment_required_response(
                    "X-PAYMENT header is required",
                    &payment_requirements,
                    is_web_browser,
                )?;
                Ok(PaymentResult::PaymentRequired { response })
            }
        }
    }

    /// Create payment required response
    fn create_payment_required_response(
        &self,
        error: &str,
        payment_requirements: &PaymentRequirements,
        is_web_browser: bool,
    ) -> Result<axum::response::Response> {
        if is_web_browser {
            let html = if let Some(custom_html) = &self.config.custom_paywall_html {
                custom_html.clone()
            } else {
                // Use the template system
                let paywall_config = self.template_config.clone().unwrap_or_else(|| {
                    crate::template::PaywallConfig::new()
                        .with_app_name("x402 Service")
                        .with_app_logo("💰")
                });

                crate::template::generate_paywall_html(
                    error,
                    std::slice::from_ref(payment_requirements),
                    Some(&paywall_config),
                )
            };

            let response = Response::builder()
                .status(StatusCode::PAYMENT_REQUIRED)
                .header("Content-Type", "text/html")
                .body(html.into())
                .map_err(|e| X402Error::config(format!("Failed to create HTML response: {}", e)))?;

            Ok(response)
        } else {
            let payment_response =
                PaymentRequirementsResponse::new(error, vec![payment_requirements.clone()]);

            Ok(Json(payment_response).into_response())
        }
    }
}

/// Axum middleware function for handling x402 payments
pub async fn payment_middleware(
    State(middleware): State<PaymentMiddleware>,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse> {
    match middleware.process_payment(request, next).await? {
        PaymentResult::Success { response, .. } => Ok(response),
        PaymentResult::PaymentRequired { response } => Ok(response),
        PaymentResult::VerificationFailed { response } => Ok(response),
        PaymentResult::SettlementFailed { response } => Ok(response),
    }
}
