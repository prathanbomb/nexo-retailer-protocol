//! Message deduplication cache for replay attack prevention
//!
//! This module provides a deduplication cache that tracks seen message IDs
//! with timestamps to prevent replay attacks. Each message ID is stored with
//! its insertion time, and entries older than the TTL are automatically expired
//! to prevent unbounded memory growth.

use crate::error::NexoError;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Default time-to-live for deduplication cache entries
///
/// Message IDs older than this duration are considered expired and can be reused.
const DEFAULT_DEDUP_TTL: Duration = Duration::from_secs(5 * 60); // 5 minutes

/// Message deduplication cache with TTL-based expiry
///
/// This cache tracks seen message IDs to prevent replay attacks. Each message ID
/// is stored with its insertion timestamp, and entries older than the TTL are
/// automatically removed on the next insert operation (lazy cleanup).
///
/// # Thread Safety
///
/// This type is NOT thread-safe by itself. It should be protected by a Mutex
/// when used in concurrent contexts (e.g., within `ConnectionState` which is
/// already wrapped in `Arc<Mutex<>>` in the server).
///
/// # Examples
///
/// ```
/// use nexo_retailer_protocol::server::dedup::DeduplicationCache;
/// use std::time::Duration;
///
/// let mut cache = DeduplicationCache::new(Duration::from_secs(60));
///
/// // First occurrence is accepted
/// assert!(cache.check_and_insert("MSG-001").is_ok());
///
/// // Duplicate within TTL is rejected
/// assert!(cache.check_and_insert("MSG-001").is_err());
///
/// // Different message ID is accepted
/// assert!(cache.check_and_insert("MSG-002").is_ok());
/// ```
#[derive(Debug, Clone)]
pub struct DeduplicationCache {
    /// Map of message ID to insertion timestamp
    seen: HashMap<String, Instant>,
    /// Time-to-live for cache entries
    ttl: Duration,
}

impl DeduplicationCache {
    /// Create a new deduplication cache with the specified TTL
    ///
    /// # Arguments
    ///
    /// * `ttl` - Time-to-live for cache entries (default: 5 minutes)
    ///
    /// # Examples
    ///
    /// ```
    /// use nexo_retailer_protocol::server::dedup::DeduplicationCache;
    /// use std::time::Duration;
    ///
    /// let cache = DeduplicationCache::new(Duration::from_secs(60));
    /// ```
    pub fn new(ttl: Duration) -> Self {
        Self {
            seen: HashMap::new(),
            ttl,
        }
    }

    /// Create a new deduplication cache with default TTL (5 minutes)
    ///
    /// # Examples
    ///
    /// ```
    /// use nexo_retailer_protocol::server::dedup::DeduplicationCache;
    ///
    /// let cache = DeduplicationCache::default();
    /// assert_eq!(cache.ttl(), std::time::Duration::from_secs(300));
    /// ```
    pub fn default() -> Self {
        Self::new(DEFAULT_DEDUP_TTL)
    }

    /// Check if a message ID is a duplicate and insert if not
    ///
    /// This method performs lazy cleanup of expired entries before checking
    /// for duplicates. If the message ID is already in the cache (and not
    /// expired), it returns an error. Otherwise, it inserts the ID with the
    /// current timestamp.
    ///
    /// # Arguments
    ///
    /// * `id` - The message ID to check and insert
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the message ID is not a duplicate (inserted successfully)
    /// * `Err(NexoError::Validation)` - If the message ID is a duplicate
    ///
    /// # Examples
    ///
    /// ```
    /// use nexo_retailer_protocol::server::dedup::DeduplicationCache;
    ///
    /// let mut cache = DeduplicationCache::default();
    ///
    /// // First occurrence is accepted
    /// assert!(cache.check_and_insert("MSG-001").is_ok());
    ///
    /// // Duplicate is rejected
    /// let result = cache.check_and_insert("MSG-001");
    /// assert!(result.is_err());
    /// assert!(matches!(result, Err(NexoError::Validation { .. })));
    /// ```
    #[cfg(feature = "alloc")]
    pub fn check_and_insert(&mut self, id: &str) -> Result<(), NexoError> {
        // Lazy cleanup: remove expired entries
        self.cleanup_expired();

        // Check if message ID exists
        if self.seen.contains_key(id) {
            return Err(NexoError::validation_owned("message_id", format!("duplicate message ID: {}", id)));
        }

        // Insert new entry with current timestamp
        self.seen.insert(id.to_string(), Instant::now());

        Ok(())
    }

    /// Check if a message ID is a duplicate and insert if not (no_std version)
    ///
    /// This no_std-compatible version uses static strings for error messages.
    /// When the `alloc` feature is available, use `check_and_insert()` instead
    /// for more detailed error messages.
    ///
    /// # Arguments
    ///
    /// * `id` - The message ID to check and insert
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the message ID is not a duplicate (inserted successfully)
    /// * `Err(NexoError::Validation)` - If the message ID is a duplicate
    pub fn check_and_insert_static(&mut self, id: &str) -> Result<(), NexoError> {
        // Lazy cleanup: remove expired entries
        self.cleanup_expired();

        // Check if message ID exists
        if self.seen.contains_key(id) {
            return Err(NexoError::Validation {
                field: "message_id",
                reason: "duplicate message ID",
            });
        }

        // Insert new entry with current timestamp
        self.seen.insert(id.to_string(), Instant::now());

        Ok(())
    }

    /// Remove expired entries from the cache
    ///
    /// This method removes all entries older than the TTL. It's called
    /// automatically by `check_and_insert()`, but can also be called manually
    /// if needed (e.g., for periodic cleanup in a background task).
    ///
    /// # Examples
    ///
    /// ```
    /// use nexo_retailer_protocol::server::dedup::DeduplicationCache;
    /// use std::time::Duration;
    ///
    /// let mut cache = DeduplicationCache::new(Duration::from_millis(100));
    ///
    /// cache.check_and_insert("MSG-001").unwrap();
    /// assert_eq!(cache.count(), 1);
    ///
    /// // Wait for expiry
    /// std::thread::sleep(Duration::from_millis(150));
    ///
    /// // Manual cleanup
    /// cache.cleanup_expired();
    /// assert_eq!(cache.count(), 0);
    /// ```
    pub fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.seen
            .retain(|_, timestamp| now.duration_since(*timestamp) < self.ttl);
    }

    /// Get the number of entries in the cache
    ///
    /// This includes both active and expired entries (expired entries will
    /// be removed on the next call to `check_and_insert()` or `cleanup_expired()`).
    ///
    /// # Examples
    ///
    /// ```
    /// use nexo_retailer_protocol::server::dedup::DeduplicationCache;
    ///
    /// let mut cache = DeduplicationCache::default();
    /// assert_eq!(cache.count(), 0);
    ///
    /// cache.check_and_insert("MSG-001").unwrap();
    /// assert_eq!(cache.count(), 1);
    /// ```
    pub fn count(&self) -> usize {
        self.seen.len()
    }

    /// Get the TTL for this cache
    ///
    /// # Returns
    ///
    /// The time-to-live duration
    pub fn ttl(&self) -> Duration {
        self.ttl
    }

    /// Check if a message ID is currently in the cache (without inserting)
    ///
    /// This is a read-only check that doesn't modify the cache. Note that
    /// expired entries are not removed by this method (use `cleanup_expired()`
    /// or `check_and_insert()` for cleanup).
    ///
    /// # Arguments
    ///
    /// * `id` - The message ID to check
    ///
    /// # Returns
    ///
    /// `true` if the message ID is in the cache, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use nexo_retailer_protocol::server::dedup::DeduplicationCache;
    ///
    /// let mut cache = DeduplicationCache::default();
    ///
    /// assert!(!cache.contains("MSG-001"));
    /// cache.check_and_insert("MSG-001").unwrap();
    /// assert!(cache.contains("MSG-001"));
    /// ```
    pub fn contains(&self, id: &str) -> bool {
        self.seen.contains_key(id)
    }

    /// Clear all entries from the cache
    ///
    /// This is primarily useful for testing. In production, the cache should
    /// be allowed to expire entries naturally via the TTL mechanism.
    ///
    /// # Examples
    ///
    /// ```
    /// use nexo_retailer_protocol::server::dedup::DeduplicationCache;
    ///
    /// let mut cache = DeduplicationCache::default();
    ///
    /// cache.check_and_insert("MSG-001").unwrap();
    /// cache.check_and_insert("MSG-002").unwrap();
    /// assert_eq!(cache.count(), 2);
    ///
    /// cache.clear();
    /// assert_eq!(cache.count(), 0);
    /// ```
    pub fn clear(&mut self) {
        self.seen.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dedup_cache_new() {
        let ttl = Duration::from_secs(60);
        let cache = DeduplicationCache::new(ttl);

        assert_eq!(cache.count(), 0);
        assert_eq!(cache.ttl(), ttl);
    }

    #[test]
    fn test_dedup_cache_default() {
        let cache = DeduplicationCache::default();

        assert_eq!(cache.count(), 0);
        assert_eq!(cache.ttl(), DEFAULT_DEDUP_TTL);
    }

    #[test]
    fn test_dedup_cache_check_and_insert_first_occurrence() {
        let mut cache = DeduplicationCache::default();

        // First occurrence should succeed
        let result = cache.check_and_insert("MSG-001");
        assert!(result.is_ok());
        assert_eq!(cache.count(), 1);
        assert!(cache.contains("MSG-001"));
    }

    #[test]
    fn test_dedup_cache_check_and_insert_duplicate_rejected() {
        let mut cache = DeduplicationCache::default();

        // First occurrence
        assert!(cache.check_and_insert("MSG-001").is_ok());

        // Duplicate should be rejected
        let result = cache.check_and_insert("MSG-001");
        assert!(result.is_err());

        match result {
            Err(NexoError::Validation { field, reason }) => {
                assert_eq!(field, "message_id");
                assert!(reason.contains("duplicate"));
                assert!(reason.contains("MSG-001"));
            }
            _ => panic!("Expected Validation error"),
        }

        // Count should still be 1 (duplicate not inserted)
        assert_eq!(cache.count(), 1);
    }

    #[test]
    fn test_dedup_cache_different_ids_accepted() {
        let mut cache = DeduplicationCache::default();

        assert!(cache.check_and_insert("MSG-001").is_ok());
        assert!(cache.check_and_insert("MSG-002").is_ok());
        assert!(cache.check_and_insert("MSG-003").is_ok());

        assert_eq!(cache.count(), 3);
    }

    #[test]
    fn test_dedup_cache_cleanup_expired() {
        let ttl = Duration::from_millis(100);
        let mut cache = DeduplicationCache::new(ttl);

        // Insert some entries
        cache.check_and_insert("MSG-001").unwrap();
        cache.check_and_insert("MSG-002").unwrap();
        cache.check_and_insert("MSG-003").unwrap();
        assert_eq!(cache.count(), 3);

        // Wait for expiry
        std::thread::sleep(Duration::from_millis(150));

        // Manually cleanup
        cache.cleanup_expired();
        assert_eq!(cache.count(), 0);

        // Expired entries can be reinserted
        assert!(cache.check_and_insert("MSG-001").is_ok());
        assert!(cache.check_and_insert("MSG-002").is_ok());
        assert_eq!(cache.count(), 2);
    }

    #[test]
    fn test_dedup_cache_lazy_cleanup_on_insert() {
        let ttl = Duration::from_millis(100);
        let mut cache = DeduplicationCache::new(ttl);

        // Insert entry
        cache.check_and_insert("MSG-001").unwrap();
        assert_eq!(cache.count(), 1);

        // Wait for expiry
        std::thread::sleep(Duration::from_millis(150));

        // Insert new entry (should trigger lazy cleanup)
        cache.check_and_insert("MSG-002").unwrap();

        // Expired entry should be removed
        assert_eq!(cache.count(), 1);
        assert!(cache.contains("MSG-002"));
        assert!(!cache.contains("MSG-001"));

        // Original ID can be reinserted after expiry
        assert!(cache.check_and_insert("MSG-001").is_ok());
        assert_eq!(cache.count(), 2);
    }

    #[test]
    fn test_dedup_cache_contains() {
        let mut cache = DeduplicationCache::default();

        assert!(!cache.contains("MSG-001"));

        cache.check_and_insert("MSG-001").unwrap();
        assert!(cache.contains("MSG-001"));

        cache.check_and_insert("MSG-002").unwrap();
        assert!(cache.contains("MSG-001"));
        assert!(cache.contains("MSG-002"));
    }

    #[test]
    fn test_dedup_cache_clear() {
        let mut cache = DeduplicationCache::default();

        cache.check_and_insert("MSG-001").unwrap();
        cache.check_and_insert("MSG-002").unwrap();
        cache.check_and_insert("MSG-003").unwrap();
        assert_eq!(cache.count(), 3);

        cache.clear();
        assert_eq!(cache.count(), 0);
        assert!(!cache.contains("MSG-001"));
        assert!(!cache.contains("MSG-002"));
        assert!(!cache.contains("MSG-003"));
    }

    #[test]
    fn test_dedup_cache_multiple_inserts_same_id() {
        let mut cache = DeduplicationCache::default();

        // First insert succeeds
        assert!(cache.check_and_insert("MSG-001").is_ok());

        // Subsequent inserts fail
        assert!(cache.check_and_insert("MSG-001").is_err());
        assert!(cache.check_and_insert("MSG-001").is_err());
        assert!(cache.check_and_insert("MSG-001").is_err());

        // Count remains 1
        assert_eq!(cache.count(), 1);
    }

    #[test]
    fn test_dedup_cache_empty_string_id() {
        let mut cache = DeduplicationCache::default();

        // Empty string should be treated as a valid ID
        assert!(cache.check_and_insert("").is_ok());
        assert!(cache.contains(""));

        // Duplicate empty string rejected
        assert!(cache.check_and_insert("").is_err());
    }

    #[test]
    fn test_dedup_cache_long_message_id() {
        let mut cache = DeduplicationCache::default();

        // Very long message ID (e.g., UUID)
        let long_id = "12345678-1234-1234-1234-123456789012";
        assert!(cache.check_and_insert(long_id).is_ok());
        assert!(cache.contains(long_id));

        // Duplicate rejected
        assert!(cache.check_and_insert(long_id).is_err());
    }
}
