//! Per-connection state tracking for Nexo server
//!
//! This module defines the `ConnectionState` struct which tracks per-connection
//! data for each client connected to the Nexo server. This state is extended
//! in later plans for deduplication and heartbeat tracking.

use std::net::SocketAddr;
use std::time::Instant;

/// Per-connection state tracking
///
/// This struct maintains state information for each connected client,
/// including the client's address, connection timestamp, and message count.
/// Additional fields for deduplication and heartbeat tracking will be added
/// in subsequent plans.
///
/// # Fields
///
/// * `addr` - The socket address of the connected client
/// * `connected_at` - Timestamp when the connection was established
/// * `message_count` - Number of messages received from this client
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
}
