//! Integration tests for Redis storage backend
//!
//! These tests verify Redis-based nonce storage functionality,
//! including distributed storage, TTL, and replay protection.
//!
//! Note: These tests require Redis to be running. They will be skipped
//! if Redis is not available.

use std::env;
use rust_x402::facilitator_storage::{InMemoryStorage, NonceStorage};

#[cfg(feature = "redis")]
use rust_x402::facilitator_storage::redis_storage::RedisStorage;

/// Check if Redis is available at the given URL
#[cfg(feature = "redis")]
async fn check_redis_available(redis_url: &str) -> bool {
    // Use async connection check to avoid panics when Redis is not available
    use redis::Client;
    match Client::open(redis_url) {
        Ok(client) => {
            match client.get_multiplexed_async_connection().await {
                Ok(mut conn) => {
                    // Try to ping Redis using AsyncCommands
                    use redis::AsyncCommands;
                    match conn.exists::<&str, bool>("__test_key__").await {
                        Ok(_) => true,
                        Err(_) => false,
                    }
                }
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}

#[tokio::test]
async fn test_in_memory_storage_basic_operations() {
    let storage = InMemoryStorage::new();

    let test_nonce = "test_nonce_basic_123";

    // Initially should not exist
    assert!(!storage.has_nonce(test_nonce).await.unwrap());

    // Mark nonce
    storage.mark_nonce(test_nonce).await.unwrap();

    // Should exist now
    assert!(storage.has_nonce(test_nonce).await.unwrap());

    // Remove nonce
    storage.remove_nonce(test_nonce).await.unwrap();

    // Should not exist after removal
    assert!(!storage.has_nonce(test_nonce).await.unwrap());
}

#[tokio::test]
async fn test_in_memory_storage_multiple_nonces() {
    let storage = InMemoryStorage::new();

    let nonce1 = "nonce_1";
    let nonce2 = "nonce_2";
    let nonce3 = "nonce_3";

    // Mark multiple nonces
    storage.mark_nonce(nonce1).await.unwrap();
    storage.mark_nonce(nonce2).await.unwrap();
    storage.mark_nonce(nonce3).await.unwrap();

    // All should exist
    assert!(storage.has_nonce(nonce1).await.unwrap());
    assert!(storage.has_nonce(nonce2).await.unwrap());
    assert!(storage.has_nonce(nonce3).await.unwrap());

    // Remove one
    storage.remove_nonce(nonce2).await.unwrap();

    // Only nonce2 should not exist
    assert!(storage.has_nonce(nonce1).await.unwrap());
    assert!(!storage.has_nonce(nonce2).await.unwrap());
    assert!(storage.has_nonce(nonce3).await.unwrap());
}

#[tokio::test]
async fn test_in_memory_storage_replay_protection() {
    let storage = InMemoryStorage::new();

    let test_nonce = "replay_nonce_456";

    // First mark should succeed
    storage.mark_nonce(test_nonce).await.unwrap();
    assert!(storage.has_nonce(test_nonce).await.unwrap());

    // Second mark should also succeed (idempotent)
    storage.mark_nonce(test_nonce).await.unwrap();
    assert!(storage.has_nonce(test_nonce).await.unwrap());
}

#[tokio::test]
#[cfg(feature = "redis")]
#[ignore] // Requires Redis service
async fn test_redis_storage_creation() {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    if !check_redis_available(&redis_url).await {
        println!("Skipping Redis test: Redis not available at {}", redis_url);
        return;
    }

    let storage = RedisStorage::new(&redis_url, None).await;
    assert!(storage.is_ok(), "RedisStorage creation should succeed");

    let storage = storage.unwrap();
    // Note: key_prefix is private, so we can't directly test it
    // But we can test that operations work
    assert!(storage.has_nonce("test").await.is_ok());
}

#[tokio::test]
#[cfg(feature = "redis")]
#[ignore] // Requires Redis service
async fn test_redis_storage_custom_prefix() {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    if !check_redis_available(&redis_url).await {
        println!("Skipping Redis test: Redis not available at {}", redis_url);
        return;
    }

    let test_prefix = format!(
        "test:prefix:{}:",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    let storage = RedisStorage::new(&redis_url, Some(&test_prefix)).await;
    assert!(storage.is_ok());

    let storage = storage.unwrap();
    let test_nonce = "test_nonce_custom_prefix";

    // Should not exist initially
    assert!(!storage.has_nonce(test_nonce).await.unwrap());

    // Mark nonce
    storage.mark_nonce(test_nonce).await.unwrap();

    // Should exist now
    assert!(storage.has_nonce(test_nonce).await.unwrap());
}

#[tokio::test]
#[cfg(feature = "redis")]
#[ignore] // Requires Redis service
async fn test_redis_storage_basic_operations() {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    if !check_redis_available(&redis_url).await {
        println!("Skipping Redis test: Redis not available at {}", redis_url);
        return;
    }

    let test_prefix = format!(
        "test:basic:{}:",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let storage = RedisStorage::new(&redis_url, Some(&test_prefix))
        .await
        .unwrap();

    let test_nonce = "test_nonce_basic_789";

    // Initially should not exist
    assert!(!storage.has_nonce(test_nonce).await.unwrap());

    // Mark nonce
    storage.mark_nonce(test_nonce).await.unwrap();

    // Should exist now
    assert!(storage.has_nonce(test_nonce).await.unwrap());

    // Remove nonce
    storage.remove_nonce(test_nonce).await.unwrap();

    // Should not exist anymore
    assert!(!storage.has_nonce(test_nonce).await.unwrap());
}

#[tokio::test]
#[cfg(feature = "redis")]
#[ignore] // Requires Redis service
async fn test_redis_storage_replay_protection() {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    if !check_redis_available(&redis_url).await {
        println!("Skipping Redis test: Redis not available at {}", redis_url);
        return;
    }

    let test_prefix = format!(
        "test:replay:{}:",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let storage = RedisStorage::new(&redis_url, Some(&test_prefix))
        .await
        .unwrap();

    let test_nonce = "replay_nonce_redis_abc";

    // First mark should succeed
    storage.mark_nonce(test_nonce).await.unwrap();
    assert!(storage.has_nonce(test_nonce).await.unwrap());

    // Second mark should also succeed (idempotent)
    storage.mark_nonce(test_nonce).await.unwrap();
    assert!(storage.has_nonce(test_nonce).await.unwrap());
}

#[tokio::test]
#[cfg(feature = "redis")]
#[ignore] // Requires Redis service
async fn test_redis_storage_multiple_nonces() {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    if !check_redis_available(&redis_url).await {
        println!("Skipping Redis test: Redis not available at {}", redis_url);
        return;
    }

    let test_prefix = format!(
        "test:multiple:{}:",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let storage = RedisStorage::new(&redis_url, Some(&test_prefix))
        .await
        .unwrap();

    let nonce1 = "redis_nonce_1";
    let nonce2 = "redis_nonce_2";
    let nonce3 = "redis_nonce_3";

    // Mark multiple nonces
    storage.mark_nonce(nonce1).await.unwrap();
    storage.mark_nonce(nonce2).await.unwrap();
    storage.mark_nonce(nonce3).await.unwrap();

    // All should exist
    assert!(storage.has_nonce(nonce1).await.unwrap());
    assert!(storage.has_nonce(nonce2).await.unwrap());
    assert!(storage.has_nonce(nonce3).await.unwrap());

    // Remove one
    storage.remove_nonce(nonce2).await.unwrap();

    // Only nonce2 should not exist
    assert!(storage.has_nonce(nonce1).await.unwrap());
    assert!(!storage.has_nonce(nonce2).await.unwrap());
    assert!(storage.has_nonce(nonce3).await.unwrap());
}

#[tokio::test]
#[cfg(feature = "redis")]
#[ignore] // Requires Redis service
async fn test_redis_storage_concurrent_access() {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    if !check_redis_available(&redis_url).await {
        println!("Skipping Redis test: Redis not available at {}", redis_url);
        return;
    }

    let test_prefix = format!(
        "test:concurrent:{}:",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let storage = RedisStorage::new(&redis_url, Some(&test_prefix))
        .await
        .unwrap();

    // Test concurrent nonce operations
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let storage = storage.clone();
            let nonce = format!("concurrent_nonce_{}", i);
            tokio::spawn(async move {
                storage.mark_nonce(&nonce).await.unwrap();
                assert!(storage.has_nonce(&nonce).await.unwrap());
            })
        })
        .collect();

    // Wait for all operations to complete
    for handle in handles {
        handle.await.unwrap();
    }
}
