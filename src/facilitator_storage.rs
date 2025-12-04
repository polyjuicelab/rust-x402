//! Storage trait for facilitator nonce tracking
//!
//! This module provides a trait-based storage abstraction for tracking
//! processed nonces to prevent replay attacks.

use crate::Result;
use async_trait::async_trait;

/// Trait for storing and retrieving nonce information
///
/// This trait allows different storage backends to be used by the facilitator,
/// enabling flexibility in deployment scenarios.
#[async_trait]
pub trait NonceStorage: Send + Sync {
    /// Check if a nonce has been processed
    async fn has_nonce(&self, nonce: &str) -> Result<bool>;

    /// Mark a nonce as processed
    async fn mark_nonce(&self, nonce: &str) -> Result<()>;

    /// Remove a nonce (optional cleanup)
    async fn remove_nonce(&self, nonce: &str) -> Result<()>;
}

/// In-memory storage implementation
///
/// This is the default storage implementation that uses an in-memory HashMap.
/// Data is lost when the server restarts.
#[derive(Debug, Clone)]
pub struct InMemoryStorage {
    nonces: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, bool>>>,
}

impl InMemoryStorage {
    /// Create a new in-memory storage instance
    pub fn new() -> Self {
        Self {
            nonces: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NonceStorage for InMemoryStorage {
    async fn has_nonce(&self, nonce: &str) -> Result<bool> {
        let nonces = self.nonces.read().await;
        Ok(nonces.contains_key(nonce))
    }

    async fn mark_nonce(&self, nonce: &str) -> Result<()> {
        let mut nonces = self.nonces.write().await;
        nonces.insert(nonce.to_string(), true);
        Ok(())
    }

    async fn remove_nonce(&self, nonce: &str) -> Result<()> {
        let mut nonces = self.nonces.write().await;
        nonces.remove(nonce);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_storage_creation() {
        let storage = InMemoryStorage::new();
        assert!(!storage.has_nonce("test").await.unwrap());
    }

    #[tokio::test]
    async fn test_in_memory_storage_has_nonce() {
        let storage = InMemoryStorage::new();
        let test_nonce = "test_nonce_123";

        // Initially, nonce should not exist
        let exists = storage.has_nonce(test_nonce).await.unwrap();
        assert!(!exists, "Nonce should not exist initially");

        // Mark nonce as processed
        storage.mark_nonce(test_nonce).await.unwrap();

        // Now nonce should exist
        let exists = storage.has_nonce(test_nonce).await.unwrap();
        assert!(exists, "Nonce should exist after marking");
    }

    #[tokio::test]
    async fn test_in_memory_storage_mark_nonce() {
        let storage = InMemoryStorage::new();
        let test_nonce = "test_nonce_mark_456";

        // Mark nonce should succeed
        let result = storage.mark_nonce(test_nonce).await;
        assert!(result.is_ok(), "mark_nonce should succeed");

        // Verify nonce was marked
        let exists = storage.has_nonce(test_nonce).await.unwrap();
        assert!(exists, "Nonce should exist after marking");
    }

    #[tokio::test]
    async fn test_in_memory_storage_remove_nonce() {
        let storage = InMemoryStorage::new();
        let test_nonce = "test_nonce_remove_789";

        // Mark nonce first
        storage.mark_nonce(test_nonce).await.unwrap();
        assert!(storage.has_nonce(test_nonce).await.unwrap());

        // Remove nonce
        let result = storage.remove_nonce(test_nonce).await;
        assert!(result.is_ok(), "remove_nonce should succeed");

        // Verify nonce was removed
        let exists = storage.has_nonce(test_nonce).await.unwrap();
        assert!(!exists, "Nonce should not exist after removal");
    }

    #[tokio::test]
    async fn test_in_memory_storage_replay_protection() {
        let storage = InMemoryStorage::new();
        let test_nonce = "test_nonce_replay_abc";

        // First mark should succeed
        assert!(!storage.has_nonce(test_nonce).await.unwrap());
        storage.mark_nonce(test_nonce).await.unwrap();

        // Second mark should still work (idempotent), but has_nonce should return true
        storage.mark_nonce(test_nonce).await.unwrap();
        assert!(
            storage.has_nonce(test_nonce).await.unwrap(),
            "Nonce should still exist after second mark"
        );
    }

    #[tokio::test]
    async fn test_in_memory_storage_multiple_nonces() {
        let storage = InMemoryStorage::new();

        let nonce1 = "nonce1";
        let nonce2 = "nonce2";
        let nonce3 = "nonce3";

        // Mark multiple nonces
        storage.mark_nonce(nonce1).await.unwrap();
        storage.mark_nonce(nonce2).await.unwrap();
        storage.mark_nonce(nonce3).await.unwrap();

        // Verify all exist
        assert!(storage.has_nonce(nonce1).await.unwrap());
        assert!(storage.has_nonce(nonce2).await.unwrap());
        assert!(storage.has_nonce(nonce3).await.unwrap());

        // Remove one
        storage.remove_nonce(nonce2).await.unwrap();
        assert!(!storage.has_nonce(nonce2).await.unwrap());
        assert!(storage.has_nonce(nonce1).await.unwrap());
        assert!(storage.has_nonce(nonce3).await.unwrap());
    }
}

#[cfg(feature = "redis")]
pub mod redis_storage {
    use super::{NonceStorage, Result};
    use redis::{AsyncCommands, Client};

    /// Redis-based storage implementation
    ///
    /// This implementation uses Redis for persistent nonce storage,
    /// allowing data to survive server restarts and enabling distributed
    /// facilitator deployments.
    #[derive(Debug, Clone)]
    pub struct RedisStorage {
        client: Client,
        key_prefix: String,
    }

    impl RedisStorage {
        /// Create a new Redis storage instance
        ///
        /// # Arguments
        ///
        /// * `redis_url` - Redis connection URL (e.g., "redis://localhost:6379")
        /// * `key_prefix` - Optional prefix for Redis keys (default: "x402:nonce:")
        pub async fn new(redis_url: &str, key_prefix: Option<&str>) -> Result<Self> {
            let client = Client::open(redis_url).map_err(|e| {
                crate::X402Error::config(format!("Failed to connect to Redis: {}", e))
            })?;

            let key_prefix = key_prefix.unwrap_or("x402:nonce:").to_string();

            Ok(Self { client, key_prefix })
        }

        fn make_key(&self, nonce: &str) -> String {
            format!("{}{}", self.key_prefix, nonce)
        }
    }

    #[async_trait::async_trait]
    impl NonceStorage for RedisStorage {
        async fn has_nonce(&self, nonce: &str) -> Result<bool> {
            let mut conn = self
                .client
                .get_multiplexed_async_connection()
                .await
                .map_err(|e| {
                    crate::X402Error::config(format!("Failed to get Redis connection: {}", e))
                })?;

            let key = self.make_key(nonce);
            let exists: bool = conn.exists(&key).await.map_err(|e| {
                crate::X402Error::config(format!("Redis EXISTS command failed: {}", e))
            })?;

            Ok(exists)
        }

        async fn mark_nonce(&self, nonce: &str) -> Result<()> {
            let mut conn = self
                .client
                .get_multiplexed_async_connection()
                .await
                .map_err(|e| {
                    crate::X402Error::config(format!("Failed to get Redis connection: {}", e))
                })?;

            let key = self.make_key(nonce);
            // Set with TTL of 24 hours to prevent unbounded growth
            conn.set_ex::<_, _, ()>(&key, "1", 86400)
                .await
                .map_err(|e| {
                    crate::X402Error::config(format!("Redis SET command failed: {}", e))
                })?;

            Ok(())
        }

        async fn remove_nonce(&self, nonce: &str) -> Result<()> {
            let mut conn = self
                .client
                .get_multiplexed_async_connection()
                .await
                .map_err(|e| {
                    crate::X402Error::config(format!("Failed to get Redis connection: {}", e))
                })?;

            let key = self.make_key(nonce);
            conn.del::<_, ()>(&key).await.map_err(|e| {
                crate::X402Error::config(format!("Redis DEL command failed: {}", e))
            })?;

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::env;

        /// Helper function to check if Redis is available
        /// Tests will be skipped if Redis is not available
        async fn check_redis_available(redis_url: &str) -> bool {
            match Client::open(redis_url) {
                Ok(client) => {
                    match client.get_multiplexed_async_connection().await {
                        Ok(mut conn) => {
                            // Try to ping Redis using AsyncCommands
                            match conn.get::<&str, Option<String>>("__test_key__").await {
                                Ok(_) => true,
                                Err(_) => {
                                    // If GET fails but connection works, try EXISTS
                                    match conn.exists::<&str, bool>("__test_key__").await {
                                        Ok(_) => true,
                                        Err(_) => false,
                                    }
                                }
                            }
                        }
                        Err(_) => false,
                    }
                }
                Err(_) => false,
            }
        }

        #[tokio::test]
        async fn test_redis_storage_creation() {
            let redis_url =
                env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

            if !check_redis_available(&redis_url).await {
                println!("Skipping Redis test: Redis not available at {}", redis_url);
                return;
            }

            let storage = RedisStorage::new(&redis_url, None).await;
            assert!(storage.is_ok(), "RedisStorage creation should succeed");

            let storage = storage.unwrap();
            assert_eq!(storage.key_prefix, "x402:nonce:");
        }

        #[tokio::test]
        async fn test_redis_storage_custom_prefix() {
            let redis_url =
                env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

            if !check_redis_available(&redis_url).await {
                println!("Skipping Redis test: Redis not available at {}", redis_url);
                return;
            }

            let storage = RedisStorage::new(&redis_url, Some("test:prefix:")).await;
            assert!(storage.is_ok());

            let storage = storage.unwrap();
            assert_eq!(storage.key_prefix, "test:prefix:");
        }

        #[tokio::test]
        async fn test_redis_storage_has_nonce() {
            let redis_url =
                env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

            if !check_redis_available(&redis_url).await {
                println!("Skipping Redis test: Redis not available at {}", redis_url);
                return;
            }

            // Use a unique prefix for this test to avoid conflicts
            let test_prefix = format!("test:{}:", uuid::Uuid::new_v4());
            let storage = RedisStorage::new(&redis_url, Some(&test_prefix))
                .await
                .unwrap();

            let test_nonce = "test_nonce_123";

            // Initially, nonce should not exist
            let exists = storage.has_nonce(test_nonce).await.unwrap();
            assert!(!exists, "Nonce should not exist initially");

            // Mark nonce as processed
            storage.mark_nonce(test_nonce).await.unwrap();

            // Now nonce should exist
            let exists = storage.has_nonce(test_nonce).await.unwrap();
            assert!(exists, "Nonce should exist after marking");

            // Clean up
            storage.remove_nonce(test_nonce).await.unwrap();
        }

        #[tokio::test]
        async fn test_redis_storage_mark_nonce() {
            let redis_url =
                env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

            if !check_redis_available(&redis_url).await {
                println!("Skipping Redis test: Redis not available at {}", redis_url);
                return;
            }

            let test_prefix = format!("test:{}:", uuid::Uuid::new_v4());
            let storage = RedisStorage::new(&redis_url, Some(&test_prefix))
                .await
                .unwrap();

            let test_nonce = "test_nonce_mark_456";

            // Mark nonce should succeed
            let result = storage.mark_nonce(test_nonce).await;
            assert!(result.is_ok(), "mark_nonce should succeed");

            // Verify nonce was marked
            let exists = storage.has_nonce(test_nonce).await.unwrap();
            assert!(exists, "Nonce should exist after marking");

            // Clean up
            storage.remove_nonce(test_nonce).await.unwrap();
        }

        #[tokio::test]
        async fn test_redis_storage_remove_nonce() {
            let redis_url =
                env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

            if !check_redis_available(&redis_url).await {
                println!("Skipping Redis test: Redis not available at {}", redis_url);
                return;
            }

            let test_prefix = format!("test:{}:", uuid::Uuid::new_v4());
            let storage = RedisStorage::new(&redis_url, Some(&test_prefix))
                .await
                .unwrap();

            let test_nonce = "test_nonce_remove_789";

            // Mark nonce first
            storage.mark_nonce(test_nonce).await.unwrap();
            assert!(storage.has_nonce(test_nonce).await.unwrap());

            // Remove nonce
            let result = storage.remove_nonce(test_nonce).await;
            assert!(result.is_ok(), "remove_nonce should succeed");

            // Verify nonce was removed
            let exists = storage.has_nonce(test_nonce).await.unwrap();
            assert!(!exists, "Nonce should not exist after removal");
        }

        #[tokio::test]
        async fn test_redis_storage_replay_protection() {
            let redis_url =
                env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

            if !check_redis_available(&redis_url).await {
                println!("Skipping Redis test: Redis not available at {}", redis_url);
                return;
            }

            let test_prefix = format!("test:{}:", uuid::Uuid::new_v4());
            let storage = RedisStorage::new(&redis_url, Some(&test_prefix))
                .await
                .unwrap();

            let test_nonce = "test_nonce_replay_abc";

            // First mark should succeed
            assert!(storage.has_nonce(test_nonce).await.unwrap() == false);
            storage.mark_nonce(test_nonce).await.unwrap();

            // Second mark should still work (idempotent), but has_nonce should return true
            storage.mark_nonce(test_nonce).await.unwrap();
            assert!(
                storage.has_nonce(test_nonce).await.unwrap(),
                "Nonce should still exist after second mark"
            );

            // Clean up
            storage.remove_nonce(test_nonce).await.unwrap();
        }

        #[tokio::test]
        async fn test_redis_storage_ttl() {
            let redis_url =
                env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

            if !check_redis_available(&redis_url).await {
                println!("Skipping Redis test: Redis not available at {}", redis_url);
                return;
            }

            let test_prefix = format!("test:{}:", uuid::Uuid::new_v4());
            let storage = RedisStorage::new(&redis_url, Some(&test_prefix))
                .await
                .unwrap();

            let test_nonce = "test_nonce_ttl_xyz";

            // Mark nonce (should have TTL of 24 hours)
            storage.mark_nonce(test_nonce).await.unwrap();

            // Verify key exists and has TTL
            let mut conn = storage
                .client
                .get_multiplexed_async_connection()
                .await
                .unwrap();
            let key = storage.make_key(test_nonce);
            let ttl: i64 = conn.ttl(&key).await.unwrap();

            // TTL should be positive (less than 86400 seconds = 24 hours)
            assert!(ttl > 0, "Key should have a positive TTL");
            assert!(
                ttl <= 86400,
                "TTL should be at most 24 hours (86400 seconds)"
            );

            // Clean up
            storage.remove_nonce(test_nonce).await.unwrap();
        }

        #[tokio::test]
        async fn test_redis_storage_multiple_nonces() {
            let redis_url =
                env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

            if !check_redis_available(&redis_url).await {
                println!("Skipping Redis test: Redis not available at {}", redis_url);
                return;
            }

            let test_prefix = format!("test:{}:", uuid::Uuid::new_v4());
            let storage = RedisStorage::new(&redis_url, Some(&test_prefix))
                .await
                .unwrap();

            let nonce1 = "nonce1";
            let nonce2 = "nonce2";
            let nonce3 = "nonce3";

            // Mark multiple nonces
            storage.mark_nonce(nonce1).await.unwrap();
            storage.mark_nonce(nonce2).await.unwrap();
            storage.mark_nonce(nonce3).await.unwrap();

            // Verify all exist
            assert!(storage.has_nonce(nonce1).await.unwrap());
            assert!(storage.has_nonce(nonce2).await.unwrap());
            assert!(storage.has_nonce(nonce3).await.unwrap());

            // Remove one
            storage.remove_nonce(nonce2).await.unwrap();
            assert!(!storage.has_nonce(nonce2).await.unwrap());
            assert!(storage.has_nonce(nonce1).await.unwrap());
            assert!(storage.has_nonce(nonce3).await.unwrap());

            // Clean up
            storage.remove_nonce(nonce1).await.unwrap();
            storage.remove_nonce(nonce3).await.unwrap();
        }
    }
}
