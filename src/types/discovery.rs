//! Discovery API types

use super::payment::PaymentRequirements;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Discovery API resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResource {
    /// The resource URL or identifier
    pub resource: String,
    /// Resource type (e.g., "http")
    pub r#type: String,
    /// Protocol version supported by the resource
    #[serde(rename = "x402Version")]
    pub x402_version: u32,
    /// Payment requirements for this resource
    pub accepts: Vec<PaymentRequirements>,
    /// Unix timestamp of when the resource was last updated
    #[serde(rename = "lastUpdated")]
    pub last_updated: u64,
    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

/// Discovery API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResponse {
    /// Protocol version
    #[serde(rename = "x402Version")]
    pub x402_version: u32,
    /// List of discoverable resources
    pub items: Vec<DiscoveryResource>,
    /// Pagination information
    pub pagination: PaginationInfo,
}

/// Pagination information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    /// Maximum number of results
    pub limit: u32,
    /// Number of results skipped
    pub offset: u32,
    /// Total number of results
    pub total: u32,
}
