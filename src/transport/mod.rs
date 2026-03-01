//! Transport layer for Nexo Retailer Protocol
//!
//! This module provides a runtime-agnostic transport trait that can be implemented
//! for both Tokio (std) and Embassy (no_std) async runtimes, enabling protocol
//! message transmission over TCP connections.

#![cfg_attr(not(feature = "std"), no_std)]

use core::time::Duration;

use crate::error::NexoError;

pub mod framing;

// Re-export framing types at module level
pub use framing::{FramedTransport, LENGTH_PREFIX_SIZE, MAX_FRAME_SIZE};

/// Runtime-agnostic transport trait for async I/O operations
///
/// This trait defines the interface for both Tokio and Embassy implementations,
/// allowing the protocol to work in both standard (std) and bare-metal (no_std)
/// environments.
///
/// # Associated Types
///
/// * `Error` - Runtime-specific error type that must convert from NexoError
///
/// # Example
///
/// ```rust,ignore
/// use nexo_retailer_protocol::transport::Transport;
///
/// struct MyTransport;
///
/// impl Transport for MyTransport {
///     type Error = NexoError;
///
///     async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
///         // Read bytes into buffer
///         Ok(bytes_read)
///     }
///
///     async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
///         // Write bytes from buffer
///         Ok(bytes_written)
///     }
///
///     async fn connect(&mut self, addr: &str) -> Result<(), Self::Error> {
///         // Connect to remote address
///         Ok(())
///     }
///
///     fn is_connected(&self) -> bool {
///         true
///     }
/// }
/// ```
pub trait Transport {
    /// Associated error type for runtime-specific errors
    ///
    /// Must implement core::error::Error and be convertible from NexoError
    type Error: core::error::Error + From<NexoError>;

    /// Read bytes from the transport into the provided buffer
    ///
    /// # Arguments
    ///
    /// * `buf` - Buffer to read bytes into
    ///
    /// # Returns
    ///
    /// Number of bytes read (0 indicates EOF for stream-based transports)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The transport is not connected
    /// - An I/O error occurs during read
    /// - The connection is closed unexpectedly
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;

    /// Write bytes from the provided buffer to the transport
    ///
    /// # Arguments
    ///
    /// * `buf` - Buffer containing bytes to write
    ///
    /// # Returns
    ///
    /// Number of bytes written
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The transport is not connected
    /// - An I/O error occurs during write
    /// - The connection is closed
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error>;

    /// Connect to a remote address
    ///
    /// # Arguments
    ///
    /// * `addr` - Remote address as a string (e.g., "192.168.1.100:8080")
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The address is invalid
    /// - The connection cannot be established
    /// - A timeout occurs
    async fn connect(&mut self, addr: &str) -> Result<(), Self::Error>;

    /// Check if the transport is currently connected
    ///
    /// This is a synchronous method that returns the current connection state
    /// without blocking.
    ///
    /// # Returns
    ///
    /// `true` if connected, `false` otherwise
    fn is_connected(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock transport for testing
    struct MockTransport {
        connected: bool,
    }

    impl Transport for MockTransport {
        type Error = NexoError;

        async fn read(&mut self, _buf: &mut [u8]) -> Result<usize, Self::Error> {
            if !self.connected {
                return Err(NexoError::Connection {
                    details: "Not connected".to_string(),
                });
            }
            Ok(0)
        }

        async fn write(&mut self, _buf: &[u8]) -> Result<usize, Self::Error> {
            if !self.connected {
                return Err(NexoError::Connection {
                    details: "Not connected".to_string(),
                });
            }
            Ok(0)
        }

        async fn connect(&mut self, _addr: &str) -> Result<(), Self::Error> {
            self.connected = true;
            Ok(())
        }

        fn is_connected(&self) -> bool {
            self.connected
        }
    }

    #[test]
    fn test_transport_trait_defined() {
        // This test verifies that the Transport trait is properly defined
        // and can be implemented by concrete types
        let transport = MockTransport { connected: false };
        assert!(!transport.is_connected());
    }
}
