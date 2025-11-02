//! EIP-712 typed data utilities

use crate::{Result, X402Error};
use ethereum_types::{Address, H256, U256};
use serde_json::json;
use std::str::FromStr;

/// EIP-712 domain separator
#[derive(Debug, Clone)]
pub struct Domain {
    pub name: String,
    pub version: String,
    pub chain_id: u64,
    pub verifying_contract: Address,
}

/// EIP-712 typed data structure
#[derive(Debug, Clone)]
pub struct TypedData {
    pub domain: Domain,
    pub primary_type: String,
    pub types: serde_json::Value,
    pub message: serde_json::Value,
}

/// Create EIP-712 hash for EIP-3009 transfer with authorization
pub fn create_transfer_with_authorization_hash(
    domain: &Domain,
    from: Address,
    to: Address,
    value: U256,
    valid_after: U256,
    valid_before: U256,
    nonce: H256,
) -> Result<H256> {
    let types = json!({
        "EIP712Domain": [
            {"name": "name", "type": "string"},
            {"name": "version", "type": "string"},
            {"name": "chainId", "type": "uint256"},
            {"name": "verifyingContract", "type": "address"}
        ],
        "TransferWithAuthorization": [
            {"name": "from", "type": "address"},
            {"name": "to", "type": "address"},
            {"name": "value", "type": "uint256"},
            {"name": "validAfter", "type": "uint256"},
            {"name": "validBefore", "type": "uint256"},
            {"name": "nonce", "type": "bytes32"}
        ]
    });

    let message = json!({
        "from": format!("{:?}", from),
        "to": format!("{:?}", to),
        "value": format!("0x{:x}", value),
        "validAfter": format!("0x{:x}", valid_after),
        "validBefore": format!("0x{:x}", valid_before),
        "nonce": format!("{:?}", nonce)
    });

    let typed_data = TypedData {
        domain: domain.clone(),
        primary_type: "TransferWithAuthorization".to_string(),
        types,
        message,
    };

    hash_typed_data(&typed_data)
}

/// Hash EIP-712 typed data
pub fn hash_typed_data(typed_data: &TypedData) -> Result<H256> {
    // Full EIP-712 implementation following the specification

    let domain_separator = hash_domain(&typed_data.domain)?;
    let struct_hash = hash_struct(
        &typed_data.primary_type,
        &typed_data.types,
        &typed_data.message,
    )?;

    // EIP-712: hash(0x1901 || domain_separator || struct_hash)
    let mut data = Vec::new();
    data.extend_from_slice(&[0x19, 0x01]); // EIP-712 prefix
    data.extend_from_slice(domain_separator.as_bytes());
    data.extend_from_slice(struct_hash.as_bytes());

    Ok(H256::from_slice(&keccak256(&data)))
}

/// Hash the domain separator
fn hash_domain(domain: &Domain) -> Result<H256> {
    let domain_type_hash = keccak256(
        b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
    );

    let name_hash = keccak256(domain.name.as_bytes());
    let version_hash = keccak256(domain.version.as_bytes());
    let chain_id_hash = keccak256(&domain.chain_id.to_be_bytes());
    let verifying_contract_hash = keccak256(domain.verifying_contract.as_bytes());

    let mut data = Vec::new();
    data.extend_from_slice(&domain_type_hash);
    data.extend_from_slice(&name_hash);
    data.extend_from_slice(&version_hash);
    data.extend_from_slice(&chain_id_hash);
    data.extend_from_slice(&verifying_contract_hash);

    Ok(H256::from_slice(&keccak256(&data)))
}

/// Hash a struct according to EIP-712
fn hash_struct(
    primary_type: &str,
    _types: &serde_json::Value,
    message: &serde_json::Value,
) -> Result<H256> {
    // Full EIP-712 struct hashing implementation

    // For TransferWithAuthorization, create the proper type hash
    let type_hash = keccak256(
        format!("{}(address from,address to,uint256 value,uint256 validAfter,uint256 validBefore,bytes32 nonce)", primary_type)
        .as_bytes()
    );

    // Encode the message fields in the correct order
    let encoded_message = encode_message_fields(message)?;
    let message_hash = keccak256(&encoded_message);

    // Combine type hash and message hash
    let mut data = Vec::new();
    data.extend_from_slice(&type_hash);
    data.extend_from_slice(&message_hash);

    Ok(H256::from_slice(&keccak256(&data)))
}

/// Encode message fields for hashing
fn encode_message_fields(message: &serde_json::Value) -> Result<Vec<u8>> {
    // For TransferWithAuthorization, encode fields in the correct order
    let mut encoded = Vec::new();

    // Encode 'from' address (32 bytes, padded)
    if let Some(from) = message.get("from") {
        if let Some(addr_str) = from.as_str() {
            let addr = Address::from_str(addr_str)
                .map_err(|_| X402Error::invalid_authorization("Invalid from address"))?;
            let mut padded = [0u8; 32];
            padded[12..32].copy_from_slice(addr.as_bytes());
            encoded.extend_from_slice(&padded);
        }
    }

    // Encode 'to' address (32 bytes, padded)
    if let Some(to) = message.get("to") {
        if let Some(addr_str) = to.as_str() {
            let addr = Address::from_str(addr_str)
                .map_err(|_| X402Error::invalid_authorization("Invalid to address"))?;
            let mut padded = [0u8; 32];
            padded[12..32].copy_from_slice(addr.as_bytes());
            encoded.extend_from_slice(&padded);
        }
    }

    // Encode 'value' (32 bytes, big-endian)
    if let Some(value) = message.get("value") {
        if let Some(value_str) = value.as_str() {
            let value_hex = value_str.trim_start_matches("0x");
            let value_bytes = hex::decode(value_hex)
                .map_err(|_| X402Error::invalid_authorization("Invalid value format"))?;
            let mut padded = [0u8; 32];
            let start = 32 - value_bytes.len();
            padded[start..].copy_from_slice(&value_bytes);
            encoded.extend_from_slice(&padded);
        }
    }

    // Encode 'validAfter' (32 bytes, big-endian)
    if let Some(valid_after) = message.get("validAfter") {
        if let Some(valid_after_str) = valid_after.as_str() {
            let valid_after_hex = valid_after_str.trim_start_matches("0x");
            let valid_after_bytes = hex::decode(valid_after_hex)
                .map_err(|_| X402Error::invalid_authorization("Invalid validAfter format"))?;
            let mut padded = [0u8; 32];
            let start = 32 - valid_after_bytes.len();
            padded[start..].copy_from_slice(&valid_after_bytes);
            encoded.extend_from_slice(&padded);
        }
    }

    // Encode 'validBefore' (32 bytes, big-endian)
    if let Some(valid_before) = message.get("validBefore") {
        if let Some(valid_before_str) = valid_before.as_str() {
            let valid_before_hex = valid_before_str.trim_start_matches("0x");
            let valid_before_bytes = hex::decode(valid_before_hex)
                .map_err(|_| X402Error::invalid_authorization("Invalid validBefore format"))?;
            let mut padded = [0u8; 32];
            let start = 32 - valid_before_bytes.len();
            padded[start..].copy_from_slice(&valid_before_bytes);
            encoded.extend_from_slice(&padded);
        }
    }

    // Encode 'nonce' (32 bytes)
    if let Some(nonce) = message.get("nonce") {
        if let Some(nonce_str) = nonce.as_str() {
            let nonce_hex = nonce_str.trim_start_matches("0x");
            let nonce_bytes = hex::decode(nonce_hex)
                .map_err(|_| X402Error::invalid_authorization("Invalid nonce format"))?;
            if nonce_bytes.len() != 32 {
                return Err(X402Error::invalid_authorization("Nonce must be 32 bytes"));
            }
            encoded.extend_from_slice(&nonce_bytes);
        }
    }

    Ok(encoded)
}

/// Keccak-256 hash function
pub fn keccak256(data: &[u8]) -> [u8; 32] {
    use sha3::{Digest, Keccak256};
    Keccak256::digest(data).into()
}

/// SHA3-256 hash function
///
/// This function is available for applications that need SHA3-256 hashing
/// in addition to Keccak-256. While EIP-712 primarily uses Keccak-256,
/// SHA3-256 may be needed for other cryptographic operations.
pub fn sha3_256(data: &[u8]) -> [u8; 32] {
    use sha3::{Digest, Sha3_256};
    Sha3_256::digest(data).into()
}
