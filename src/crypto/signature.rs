//! Signature utilities

use super::eip712;
use crate::{Result, X402Error};
use ethereum_types::{Address, H256, U256};
use k256::ecdsa::{RecoveryId, Signature as K256Signature, VerifyingKey};
use secp256k1::{Message, Secp256k1, SecretKey};
use std::str::FromStr;

/// Verify an EIP-712 signature
pub fn verify_eip712_signature(
    signature: &str,
    message_hash: H256,
    expected_address: Address,
) -> Result<bool> {
    let sig_bytes = hex::decode(signature.trim_start_matches("0x"))
        .map_err(|_| X402Error::invalid_signature("Invalid hex signature"))?;

    if sig_bytes.len() != 65 {
        return Err(X402Error::invalid_signature("Signature must be 65 bytes"));
    }

    let r = H256::from_slice(&sig_bytes[0..32]);
    let s = H256::from_slice(&sig_bytes[32..64]);
    let v = sig_bytes[64];

    let recovery_id =
        RecoveryId::try_from(v).map_err(|_| X402Error::invalid_signature("Invalid recovery ID"))?;

    // Create k256 signature from r and s
    let mut sig_bytes = [0u8; 64];
    sig_bytes[0..32].copy_from_slice(r.as_bytes());
    sig_bytes[32..64].copy_from_slice(s.as_bytes());

    let k256_sig = K256Signature::try_from(&sig_bytes[..])
        .map_err(|_| X402Error::invalid_signature("Invalid signature format"))?;

    // Recover the public key
    let verifying_key =
        VerifyingKey::recover_from_prehash(message_hash.as_bytes(), &k256_sig, recovery_id)
            .map_err(|_| X402Error::invalid_signature("Failed to recover public key"))?;

    // Convert to Ethereum address
    let recovered_address = ethereum_address_from_pubkey(&verifying_key)?;

    Ok(recovered_address == expected_address)
}

/// Sign a message hash with a private key
pub fn sign_message_hash(message_hash: H256, private_key: &str) -> Result<String> {
    let private_key_bytes = hex::decode(private_key.trim_start_matches("0x"))
        .map_err(|_| X402Error::invalid_signature("Invalid hex private key"))?;

    let secret_key = SecretKey::from_slice(&private_key_bytes)
        .map_err(|_| X402Error::invalid_signature("Invalid private key"))?;

    let secp = Secp256k1::new();
    let message = Message::from_digest_slice(message_hash.as_bytes())
        .map_err(|_| X402Error::invalid_signature("Invalid message hash"))?;

    let signature = secp.sign_ecdsa(&message, &secret_key);
    let serialized = signature.serialize_compact();

    // Compute the recovery ID properly
    // The recovery ID is used to recover the public key from the signature
    let recovery_id = compute_recovery_id(&signature, &message, &secret_key)?;

    // Convert to k256 signature for consistency
    let _k256_sig = K256Signature::try_from(&serialized[..])
        .map_err(|_| X402Error::invalid_signature("Failed to convert signature"))?;

    // Create the full signature with recovery ID
    let mut sig_bytes = [0u8; 65];
    sig_bytes[0..32].copy_from_slice(&serialized[0..32]);
    sig_bytes[32..64].copy_from_slice(&serialized[32..64]);
    sig_bytes[64] = recovery_id;

    Ok(format!("0x{}", hex::encode(sig_bytes)))
}

/// Convert a public key to an Ethereum address
fn ethereum_address_from_pubkey(pubkey: &k256::ecdsa::VerifyingKey) -> Result<Address> {
    let pubkey_bytes = pubkey.to_sec1_bytes();
    if pubkey_bytes.len() != 65 {
        return Err(X402Error::invalid_signature("Invalid public key length"));
    }

    // Remove the first byte (0x04) and hash the remaining 64 bytes
    let pubkey_hash = keccak256(&pubkey_bytes[1..]);

    // Take the last 20 bytes as the address
    let mut address_bytes = [0u8; 20];
    address_bytes.copy_from_slice(&pubkey_hash[12..]);

    Ok(Address::from(address_bytes))
}

/// Compute the recovery ID for a signature
fn compute_recovery_id(
    signature: &secp256k1::ecdsa::Signature,
    message: &Message,
    private_key: &SecretKey,
) -> Result<u8> {
    let secp = Secp256k1::new();

    // Get the public key from the private key
    let public_key = private_key.public_key(&secp);

    // Try both possible recovery IDs (0 and 1)
    for recovery_id in 0..2 {
        // Create RecoveryId from i32 (secp256k1 uses i32, not u8)
        let recovery_id_enum = secp256k1::ecdsa::RecoveryId::from_i32(recovery_id as i32);
        if let Ok(recovery_id_enum) = recovery_id_enum {
            // Create a recoverable signature with this recovery ID
            if let Ok(recoverable_sig) = secp256k1::ecdsa::RecoverableSignature::from_compact(
                &signature.serialize_compact(),
                recovery_id_enum,
            ) {
                // Try to recover the public key using this recovery ID
                if let Ok(recovered_key) = secp.recover_ecdsa(message, &recoverable_sig) {
                    // If the recovered key matches our public key, this is the correct recovery ID
                    if recovered_key == public_key {
                        return Ok(recovery_id);
                    }
                }
            }
        }
    }

    Err(X402Error::invalid_signature(
        "Could not determine recovery ID",
    ))
}

/// Keccak-256 hash function
fn keccak256(data: &[u8]) -> [u8; 32] {
    use sha3::{Digest, Keccak256};
    Keccak256::digest(data).into()
}

/// Generate a random nonce for EIP-3009 authorization
pub fn generate_nonce() -> H256 {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    H256::from_slice(&bytes)
}

/// Verify a payment payload signature
pub fn verify_payment_payload(
    payload: &crate::types::ExactEvmPayload,
    expected_from: &str,
    network: &str,
) -> Result<bool> {
    let from_addr = Address::from_str(expected_from)
        .map_err(|_| X402Error::invalid_signature("Invalid from address"))?;

    // Create the message hash from authorization
    let auth = &payload.authorization;

    // Get network configuration based on the payment network
    let network_config = crate::types::NetworkConfig::from_name(network)
        .ok_or_else(|| X402Error::invalid_signature("Unsupported network"))?;

    let message_hash = eip712::create_transfer_with_authorization_hash(
        &eip712::Domain {
            name: "USD Coin".to_string(),
            version: "2".to_string(),
            chain_id: network_config.chain_id,
            verifying_contract: Address::from_str(&network_config.usdc_contract)
                .map_err(|_| X402Error::invalid_signature("Invalid verifying contract"))?,
        },
        Address::from_str(&auth.from)
            .map_err(|_| X402Error::invalid_signature("Invalid from address"))?,
        Address::from_str(&auth.to)
            .map_err(|_| X402Error::invalid_signature("Invalid to address"))?,
        U256::from_str_radix(&auth.value, 10)
            .map_err(|_| X402Error::invalid_signature("Invalid value"))?,
        U256::from_str_radix(&auth.valid_after, 10)
            .map_err(|_| X402Error::invalid_signature("Invalid valid_after"))?,
        U256::from_str_radix(&auth.valid_before, 10)
            .map_err(|_| X402Error::invalid_signature("Invalid valid_before"))?,
        H256::from_str(&auth.nonce).map_err(|_| X402Error::invalid_signature("Invalid nonce"))?,
    )?;

    verify_eip712_signature(&payload.signature, message_hash, from_addr)
}
