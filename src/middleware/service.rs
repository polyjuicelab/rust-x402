//! Tower service layer for payment middleware

use super::payment::PaymentMiddleware;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

/// Create a service builder with x402 payment middleware
pub fn create_payment_service(
    middleware: PaymentMiddleware,
) -> impl tower::Layer<tower::ServiceBuilder<tower::layer::util::Identity>> + Clone {
    ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(tower::layer::util::Stack::new(
            tower::layer::util::Identity::new(),
            PaymentServiceLayer::new(middleware),
        ))
}

/// Tower service layer for x402 payment middleware
#[derive(Clone)]
pub struct PaymentServiceLayer {
    middleware: PaymentMiddleware,
}

impl PaymentServiceLayer {
    pub fn new(middleware: PaymentMiddleware) -> Self {
        Self { middleware }
    }
}

impl<S> tower::Layer<S> for PaymentServiceLayer {
    type Service = PaymentService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        PaymentService {
            inner,
            middleware: self.middleware.clone(),
        }
    }
}

/// Tower service for x402 payment middleware
#[derive(Clone)]
pub struct PaymentService<S> {
    inner: S,
    middleware: PaymentMiddleware,
}

impl<S, ReqBody, ResBody> tower::Service<http::Request<ReqBody>> for PaymentService<S>
where
    S: tower::Service<
            http::Request<ReqBody>,
            Response = http::Response<ResBody>,
            Error = Box<dyn std::error::Error + Send + Sync>,
        > + Send
        + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<
            dyn std::future::Future<Output = std::result::Result<Self::Response, Self::Error>>
                + Send,
        >,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        let middleware = self.middleware.clone();

        // Extract payment header before moving the request
        let payment_header = req
            .headers()
            .get("X-PAYMENT")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());
        let uri_path = req.uri().path().to_string();

        let future = self.inner.call(req);

        Box::pin(async move {
            match payment_header {
                Some(payment_b64) => {
                    // Parse payment payload
                    match crate::types::PaymentPayload::from_base64(&payment_b64) {
                        Ok(payment_payload) => {
                            // Create payment requirements
                            let requirements =
                                match middleware.config.create_payment_requirements(&uri_path) {
                                    Ok(req) => req,
                                    Err(e) => {
                                        // Return 500 error if we can't create requirements
                                        return Err(
                                            Box::new(e) as Box<dyn std::error::Error + Send + Sync>
                                        );
                                    }
                                };

                            // Verify payment
                            match middleware
                                .verify_with_requirements(&payment_payload, &requirements)
                                .await
                            {
                                Ok(true) => {
                                    // Payment is valid, proceed with request
                                    let response = future.await?;

                                    // Settle payment after successful response
                                    if let Ok(settlement) = middleware
                                        .settle_with_requirements(&payment_payload, &requirements)
                                        .await
                                    {
                                        // Note: In a real implementation, we would need to modify the response
                                        // to add the X-PAYMENT-RESPONSE header, but this requires
                                        // more complex response handling in Tower
                                        let _ = settlement; // Acknowledge settlement
                                    }

                                    Ok(response)
                                }
                                Ok(false) => {
                                    // Payment verification failed
                                    Err(Box::new(crate::X402Error::payment_verification_failed(
                                        "Payment verification failed",
                                    ))
                                        as Box<dyn std::error::Error + Send + Sync>)
                                }
                                Err(e) => {
                                    // Error during verification
                                    Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                                }
                            }
                        }
                        Err(e) => {
                            // Invalid payment payload
                            Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                        }
                    }
                }
                None => {
                    // No payment header provided
                    Err(Box::new(crate::X402Error::payment_verification_failed(
                        "X-PAYMENT header is required",
                    ))
                        as Box<dyn std::error::Error + Send + Sync>)
                }
            }
        })
    }
}
