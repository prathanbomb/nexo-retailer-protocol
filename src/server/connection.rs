//! Per-connection state tracking for Nexo server
//!
//! This module defines the `ConnectionState` struct which tracks per-connection
//! data for each client connected to the Nexo server. This state is extended
//! in later plans for deduplication and heartbeat tracking.

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use crate::server::dedup::DeduplicationCache;

/// Default TTL for deduplication cache (5 minutes)
const DEFAULT_DEDUP_TTL: Duration = Duration::from_secs(5 * 60);

/// Per-connection state tracking
///
/// This struct maintains state information for each connected client,
/// including the client's address, connection timestamp, message count,
/// and deduplication cache for replay attack prevention.
///
/// # Fields
///
/// * `addr` - The socket address of the connected client
/// * `connected_at` - Timestamp when the connection was established
/// * `message_count` - Number of messages received from this client
/// * `seen_message_ids` - Deduplication cache for tracking seen message IDs
///
/// # Examples
///
/// ```
/// use nexo_retailer_protocol::server::ConnectionState;
/// use std::net::SocketAddr;
///
/// let addr = "127.0.0.1:12345".parse().unwrap();
/// let state = ConnectionState::new(addr);
///
/// assert_eq!(state.addr(), addr);
/// assert!(state.connected_at() <= std::time::Instant::now());
/// assert_eq!(state.message_count(), 0);
/// ```
#[derive(Debug, Clone)]
pub struct ConnectionState {
    /// Socket address of the connected client
    addr: SocketAddr,
    /// Timestamp when the connection was established
    connected_at: Instant,
    /// Number of messages received from this client
    message_count: u64,
    /// Deduplication cache for tracking seen message IDs
    seen_message_ids: DeduplicationCache,
}

impl ConnectionState {
    /// Create a new connection state
    ///
    /// # Arguments
    ///
    /// * `addr` - The socket address of the connected client
    ///
    /// # Examples
    ///
    /// ```
    /// use nexo_retailer_protocol::server::ConnectionState;
    /// use std::net::SocketAddr;
    ///
    /// let addr = "127.0.0.1:8080".parse().unwrap();
    /// let state = ConnectionState::new(addr);
    /// ```
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            connected_at: Instant::now(),
            message_count: 0,
            seen_message_ids: DeduplicationCache::default(),
        }
    }

    /// Create a new connection state with custom deduplication TTL
    ///
    /// # Arguments
    ///
    /// * `addr` - The socket address of the connected client
    /// * `dedup_ttl` - Time-to-live for deduplication cache entries
    ///
    /// # Examples
    ///
    /// ```
    /// use nexo_retailer_protocol::server::ConnectionState;
    /// use std::net::SocketAddr;
    /// use std::time::Duration;
    ///
    /// let addr = "127.0.0.1:8080".parse().unwrap();
    /// let ttl = Duration::from_secs(60);
    /// let state = ConnectionState::with_ttl(addr, ttl);
    /// ```
    pub fn with_ttl(addr: SocketAddr, dedup_ttl: Duration) -> Self {
        Self {
            addr,
            connected_at: Instant::now(),
            message_count: 0,
            seen_message_ids: DeduplicationCache::new(dedup_ttl),
        }
    }

    /// Get the socket address of the connected client
    ///
    /// # Returns
    ///
    /// The socket address
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Get the timestamp when the connection was established
    ///
    /// # Returns
    ///
    /// The connection timestamp
    pub fn connected_at(&self) -> Instant {
        self.connected_at
    }

    /// Get the number of messages received from this client
    ///
    /// # Returns
    ///
    /// The message count
    pub fn message_count(&self) -> u64 {
        self.message_count
    }

    /// Increment the message count (called when a message is received)
    pub(crate) fn increment_message_count(&mut self) {
        self.message_count += 1;
    }

    /// Get a mutable reference to the deduplication cache
    ///
    /// This allows the server to check for duplicate message IDs before
    /// processing incoming messages.
    ///
    /// # Returns
    ///
    /// A mutable reference to the deduplication cache
    ///
    /// # Examples
    ///
    /// ```
    /// use nexo_retailer_protocol::server::ConnectionState;
    /// use std::net::SocketAddr;
    ///
    /// let addr = "127.0.0.1:8080".parse().unwrap();
    /// let mut state = ConnectionState::new(addr);
    ///
    /// // Check for duplicate message ID
    /// let result = state.dedup_cache().check_and_insert_static("MSG-001");
    /// assert!(result.is_ok());
    /// ```
    #[cfg(feature = "alloc")]
    pub fn dedup_cache(&mut self) -> &mut DeduplicationCache {
        &mut self.seen_message_ids
    }

    /// Get a mutable reference to the deduplication cache (no_std version)
    ///
    /// This no_std-compatible version uses static strings for error messages.
    #[cfg(not(feature = "alloc"))]
    pub fn dedup_cache(&mut self) -> &mut DeduplicationCache {
        &mut self.seen_message_ids
    }

    /// Get the duration since the connection was established
    ///
    /// # Returns
    ///
    /// The duration since connection
    pub fn connection_duration(&self) -> core::time::Duration {
        self.connected_at.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_state_new() {
        let addr = "127.0.0.1:8080".parse().unwrap();
        let state = ConnectionState::new(addr);

        assert_eq!(state.addr(), addr);
        assert_eq!(state.message_count(), 0);
        assert!(state.connection_duration().as_secs() < 1); // Should be very recent
    }

    #[test]
    fn test_connection_state_increment_message_count() {
        let addr = "127.0.0.1:8080".parse().unwrap();
        let mut state = ConnectionState::new(addr);

        assert_eq!(state.message_count(), 0);

        state.increment_message_count();
        assert_eq!(state.message_count(), 1);

        state.increment_message_count();
        assert_eq!(state.message_count(), 2);
    }

    #[test]
    fn test_connection_state_duration_increases() {
        let addr = "127.0.0.1:8080".parse().unwrap();
        let state = ConnectionState::new(addr);

        let duration1 = state.connection_duration();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let duration2 = state.connection_duration();

        assert!(duration2 > duration1);
    }

    #[test]
    fn test_connection_state_field_accessors() {
        let addr = "192.168.1.100:12345".parse().unwrap();
        let state = ConnectionState::new(addr);

        // Test addr accessor
        assert_eq!(state.addr(), addr);

        // Test connected_at accessor
        let now = std::time::Instant::now();
        let connected_at = state.connected_at();
        assert!(connected_at <= now);

        // Test message_count accessor
        assert_eq!(state.message_count(), 0);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_connection_state_new_has_empty_dedup_cache() {
        let addr = "127.0.0.1:8080".parse().unwrap();
        let mut state = ConnectionState::new(addr);

        // New connection should have empty dedup cache
        assert_eq!(state.dedup_cache().count(), 0);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_connection_state_dedup_cache_insertion_and_retrieval() {
        let addr = "127.0.0.1:8080".parse().unwrap();
        let mut state = ConnectionState::new(addr);

        // Insert message ID
        assert!(state.dedup_cache().check_and_insert("MSG-001").is_ok());
        assert_eq!(state.dedup_cache().count(), 1);
        assert!(state.dedup_cache().contains("MSG-001"));

        // Duplicate should be rejected
        assert!(state.dedup_cache().check_and_insert("MSG-001").is_err());
        assert_eq!(state.dedup_cache().count(), 1);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_connection_state_with_custom_ttl() {
        let addr = "127.0.0.1:8080".parse().unwrap();
        let ttl = Duration::from_secs(60);
        let mut state = ConnectionState::with_ttl(addr, ttl);

        // Verify custom TTL is set
        assert_eq!(state.dedup_cache().ttl(), ttl);

        // Insert message ID
        assert!(state.dedup_cache().check_and_insert("MSG-001").is_ok());
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_connection_state_expired_message_ids_accepted() {
        let addr = "127.0.0.1:8080".parse().unwrap();
        let ttl = Duration::from_millis(100);
        let mut state = ConnectionState::with_ttl(addr, ttl);

        // Insert message ID
        assert!(state.dedup_cache().check_and_insert("MSG-001").is_ok());
        assert_eq!(state.dedup_cache().count(), 1);

        // Wait for expiry
        std::thread::sleep(Duration::from_millis(150));

        // Insert should succeed after expiry (cleanup happens on insert)
        assert!(state.dedup_cache().check_and_insert("MSG-001").is_ok());
        assert_eq!(state.dedup_cache().count(), 1);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_connection_state_different_message_ids_accepted() {
        let addr = "127.0.0.1:8080".parse().unwrap();
        let mut state = ConnectionState::new(addr);

        // Different message IDs should all be accepted
        assert!(state.dedup_cache().check_and_insert("MSG-001").is_ok());
        assert!(state.dedup_cache().check_and_insert("MSG-002").is_ok());
        assert!(state.dedup_cache().check_and_insert("MSG-003").is_ok());

        assert_eq!(state.dedup_cache().count(), 3);
    }

    #[test]
    fn test_connection_state_with_ttl_creates_valid_state() {
        let addr = "127.0.0.1:8080".parse().unwrap();
        let ttl = Duration::from_secs(30);
        let state = ConnectionState::with_ttl(addr, ttl);

        assert_eq!(state.addr(), addr);
        assert_eq!(state.message_count(), 0);
        assert!(state.connection_duration().as_secs() < 1);
    }
}
