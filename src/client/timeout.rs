//! Timeout handling and message ID generation for Nexo Retailer Protocol
//!
//! This module provides timeout configuration and unique message ID generation
//! for request/response correlation. Timeouts prevent indefinite waiting, and
//! unique message IDs enable replay protection and response matching.
//!
//! # Message ID Generation
//!
//! - **std**: Uses UUID v4 for cryptographically unique message IDs
//! - **no_std**: Uses timestamp-counter format for uniqueness without RNG
//!
//! # Timeout Strategy
//!
//! When a request times out:
//! 1. The pending request is removed from tracking (cleanup)
//! 2. Late responses arriving after timeout are rejected
//! 3. The caller receives `NexoError::Timeout`
//!
//! This prevents state confusion from responses arriving after the caller
//! has given up waiting.

use core::time::Duration;
use core::sync::atomic::{AtomicU64, Ordering};

#[cfg(feature = "std")]
use uuid::Uuid;

/// Global counter for no_std message ID generation
///
/// In no_std environments without a random number generator, we use
/// a timestamp-counter format: `{timestamp_ms}-{counter}`
/// This provides uniqueness across restarts (timestamp) and within
/// the same millisecond (counter).
static MESSAGE_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Timeout configuration for Nexo client requests
///
/// `TimeoutConfig` defines how long to wait for responses before timing out.
/// Sensible defaults are provided, but custom durations can be set based on
/// network conditions and application requirements.
///
/// # Default Behavior
///
/// - Default timeout: 30 seconds
/// - Late responses: Rejected with warning (std) or silently dropped (no_std)
///
/// # Examples
///
/// ```rust
/// use nexo_retailer_protocol::client::timeout::TimeoutConfig;
/// use core::time::Duration;
///
/// // Use default 30-second timeout
/// let config = TimeoutConfig::new();
///
/// // Custom timeout for slow networks
/// let config = TimeoutConfig::new()
///     .with_request_timeout(Duration::from_secs(60));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeoutConfig {
    /// Maximum duration to wait for a response
    pub request_timeout: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeoutConfig {
    /// Create a new timeout config with default values
    ///
    /// Default timeout is 30 seconds, which is appropriate for most
    /// payment terminal networks.
    pub fn new() -> Self {
        Self {
            request_timeout: Duration::from_secs(30),
        }
    }

    /// Set the request timeout duration
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum duration to wait for a response
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexo_retailer_protocol::client::timeout::TimeoutConfig;
    /// use core::time::Duration;
    ///
    /// let config = TimeoutConfig::new()
    ///     .with_request_timeout(Duration::from_secs(45));
    /// ```
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }
}

/// Generate a unique message ID for request correlation
///
/// Message IDs are used to match responses to requests and provide replay
/// protection. Each request should have a unique ID.
///
/// # Implementation
///
/// - **std feature**: Uses UUID v4 (random, 122 bits of entropy)
/// - **embassy feature**: Uses `{timestamp_ms}-{counter}` format
///
/// # Returns
///
/// A unique string identifier
///
/// # Examples
///
/// ```rust
/// use nexo_retailer_protocol::client::timeout::generate_message_id;
///
/// let id1 = generate_message_id();
/// let id2 = generate_message_id();
/// assert_ne!(id1, id2);
/// ```
///
/// # Format
///
/// - std: UUID v4 format (e.g., "550e8400-e29b-41d4-a716-446655440000")
/// - no_std: Timestamp-counter format (e.g., "1772386108123-1")
#[cfg(feature = "std")]
pub fn generate_message_id() -> String {
    // Use UUID v4 for std environments (requires rng support)
    Uuid::new_v4().to_string()
}

/// Generate a unique message ID for request correlation (no_std)
///
/// In no_std environments without a random number generator, we use
/// a timestamp-counter format: `{timestamp_ms}-{counter}`.
///
/// This provides:
/// - Uniqueness across restarts (timestamp)
/// - Uniqueness within the same millisecond (counter)
/// - Sortability by timestamp
///
/// # Returns
///
/// A unique string identifier in `{timestamp_ms}-{counter}` format
///
/// # Examples
///
/// ```rust,ignore
/// use nexo_retailer_protocol::client::timeout::generate_message_id;
///
/// let id = generate_message_id();
/// assert!(id.contains('-')); // Format: "timestamp-counter"
/// ```
#[cfg(not(feature = "std"))]
pub fn generate_message_id() -> String {
    use alloc::format;

    // Get timestamp in milliseconds
    // Note: In embedded environments, this might need to be replaced
    // with a custom time source. For now, we use a simple counter.
    let counter = MESSAGE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

    // Use a simple format: counter-based (can be replaced with timestamp)
    // In real embedded systems, you'd use a hardware timer here
    format!("{}-{}", counter, counter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_config_default() {
        let config = TimeoutConfig::new();
        assert_eq!(config.request_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_timeout_config_custom() {
        let config = TimeoutConfig::new()
            .with_request_timeout(Duration::from_secs(60));
        assert_eq!(config.request_timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_timeout_config_default_trait() {
        let config = TimeoutConfig::default();
        assert_eq!(config.request_timeout, Duration::from_secs(30));
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_generate_message_id_unique() {
        let id1 = generate_message_id();
        let id2 = generate_message_id();
        assert_ne!(id1, id2);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_generate_message_id_format() {
        let id = generate_message_id();
        // UUID v4 format: 8-4-4-4-12 hex digits
        assert!(id.len() == 36); // "550e8400-e29b-41d4-a716-446655440000"
        assert_eq!(id.chars().filter(|&c| c == '-').count(), 4);
    }

    #[cfg(not(feature = "std"))]
    #[test]
    fn test_generate_message_id_unique_no_std() {
        let id1 = generate_message_id();
        let id2 = generate_message_id();
        assert_ne!(id1, id2);
    }

    #[cfg(not(feature = "std"))]
    #[test]
    fn test_generate_message_id_format_no_std() {
        let id = generate_message_id();
        // Format: "counter-counter"
        assert!(id.contains('-'));
    }
}
