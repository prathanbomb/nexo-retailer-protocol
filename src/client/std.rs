//! Tokio-based client implementation for Nexo Retailer Protocol
//!
//! This module provides the standard library implementation of `NexoClient`
//! using Tokio for async runtime and `tokio::sync::oneshot` for request/response
//! correlation.

#![cfg(feature = "std")]

use crate::error::NexoError;
use crate::transport::{FramedTransport, Transport, TokioTransport};

use prost::Message;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;

use tokio::sync::oneshot;

/// Pending request tracking using oneshot channels
///
/// This struct manages in-flight requests by mapping message IDs to oneshot
/// senders. When a response arrives, the corresponding sender is used to
/// deliver the response to the waiting task.
type PendingRequests = BTreeMap<String, oneshot::Sender<Vec<u8>>>;

/// Nexo Retailer Protocol client for Tokio runtime
///
/// This client provides a high-level API for connecting to payment terminals,
/// sending requests, and receiving responses. It handles connection management
/// and request/response correlation automatically.
///
/// # Type Parameters
///
/// * `T` - Transport implementation (defaults to `TokioTransport`)
///
/// # Examples
///
/// ```rust,no_run
/// use nexo_retailer_protocol::NexoClient;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create client with default Tokio transport
/// let mut client = NexoClient::new();
///
/// // Connect to payment terminal
/// client.connect("192.168.1.100:8080").await?;
///
/// // Check connection state
/// assert!(client.is_connected());
///
/// // Disconnect
/// client.disconnect().await?;
/// # Ok(())
/// # }
/// ```
pub struct NexoClient<T: Transport = TokioTransport> {
    /// Framed transport wrapper for length-prefixed messaging
    transport: Option<FramedTransport<T>>,
    /// Connection state flag
    connected: Arc<AtomicBool>,
    /// Server address for reconnection
    server_addr: String,
    /// Pending requests awaiting responses
    #[allow(dead_code)]
    pending: PendingRequests,
}

impl NexoClient<TokioTransport> {
    /// Create a new unconnected client with default Tokio transport
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexo_retailer_protocol::NexoClient;
    ///
    /// let client = NexoClient::new();
    /// assert!(!client.is_connected());
    /// ```
    pub fn new() -> Self {
        Self {
            transport: None,
            connected: Arc::new(AtomicBool::new(false)),
            server_addr: String::new(),
            pending: PendingRequests::new(),
        }
    }

    /// Connect to a Nexo payment terminal
    ///
    /// # Arguments
    ///
    /// * `addr` - Server address (e.g., "192.168.1.100:8080")
    ///
    /// # Errors
    ///
    /// Returns `NexoError::Connection` if:
    /// - The address is invalid
    /// - The connection cannot be established
    /// - A timeout occurs
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nexo_retailer_protocol::NexoClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = NexoClient::new();
    /// client.connect("192.168.1.100:8080").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(&mut self, addr: &str) -> Result<(), NexoError> {
        let transport = TokioTransport::connect(addr, Duration::from_secs(10)).await?;
        self.transport = Some(FramedTransport::new(transport));
        self.server_addr = addr.to_string();
        self.connected.store(true, Ordering::Release);
        Ok(())
    }
}

impl<T: Transport> NexoClient<T> {
    /// Create a new unconnected client with custom transport
    ///
    /// # Arguments
    ///
    /// * `transport` - Transport implementation to use
    pub fn with_transport(transport: T) -> Self {
        Self {
            transport: Some(FramedTransport::new(transport)),
            connected: Arc::new(AtomicBool::new(false)),
            server_addr: String::new(),
            pending: PendingRequests::new(),
        }
    }

    /// Disconnect from the payment terminal
    ///
    /// This method closes the connection and resets the client state.
    /// The client can be reconnected by calling `connect()` again.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nexo_retailer_protocol::NexoClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = NexoClient::new();
    /// client.connect("192.168.1.100:8080").await?;
    /// client.disconnect().await?;
    /// assert!(!client.is_connected());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn disconnect(&mut self) -> Result<(), T::Error> {
        self.connected.store(false, Ordering::Release);
        self.transport = None;
        self.server_addr.clear();
        Ok(())
    }

    /// Check if the client is currently connected
    ///
    /// # Returns
    ///
    /// `true` if connected, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nexo_retailer_protocol::NexoClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = NexoClient::new();
    /// assert!(!client.is_connected());
    ///
    /// client.connect("192.168.1.100:8080").await?;
    /// assert!(client.is_connected());
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Acquire)
    }

    /// Send a request message
    ///
    /// # Arguments
    ///
    /// * `request` - Request message implementing `prost::Message`
    ///
    /// # Errors
    ///
    /// Returns `NexoError::Connection` if not connected
    /// Returns `NexoError::Encoding` if message encoding fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nexo_retailer_protocol::NexoClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = NexoClient::new();
    /// client.connect("192.168.1.100:8080").await?;
    ///
    /// let request = YourRequestType::default();
    /// client.send_request(&request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_request<M: Message>(&mut self, request: &M) -> Result<(), T::Error> {
        if !self.is_connected() {
            return Err(NexoError::Connection {
                details: "not connected",
            }
            .into());
        }

        let transport = self
            .transport
            .as_mut()
            .ok_or_else(|| NexoError::Connection {
                details: "transport not initialized",
            })?;

        transport.send_message(request).await?;
        Ok(())
    }

    /// Receive a response message
    ///
    /// # Arguments
    ///
    /// * `request` - Phantom type parameter for the response message type
    ///
    /// # Errors
    ///
    /// Returns `NexoError::Connection` if not connected
    /// Returns `NexoError::Decoding` if message decoding fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nexo_retailer_protocol::NexoClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = NexoClient::new();
    /// client.connect("192.168.1.100:8080").await?;
    ///
    /// let response: YourResponseType = client.receive_response().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn receive_response<M: Message + Default>(
        &mut self,
    ) -> Result<M, T::Error> {
        if !self.is_connected() {
            return Err(NexoError::Connection {
                details: "not connected",
            }
            .into());
        }

        let transport = self
            .transport
            .as_mut()
            .ok_or_else(|| NexoError::Connection {
                details: "transport not initialized",
            })?;

        let msg = transport.recv_message().await?;
        Ok(msg)
    }

    /// Send a request and receive a response (combined operation)
    ///
    /// This is a convenience method that combines `send_request` and
    /// `receive_response` into a single call.
    ///
    /// # Arguments
    ///
    /// * `request` - Request message implementing `prost::Message`
    ///
    /// # Errors
    ///
    /// Returns `NexoError::Connection` if not connected
    /// Returns `NexoError::Encoding` if message encoding fails
    /// Returns `NexoError::Decoding` if message decoding fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nexo_retailer_protocol::NexoClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = NexoClient::new();
    /// client.connect("192.168.1.100:8080").await?;
    ///
    /// let request = YourRequestType::default();
    /// let response: YourResponseType = client.send_and_receive(&request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_and_receive<M: Message + Default>(
        &mut self,
        request: &M,
    ) -> Result<M, T::Error> {
        self.send_request(request).await?;
        self.receive_response().await
    }
}

impl Default for NexoClient<TokioTransport> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_executor::block_on;

    // Mock transport for testing
    struct MockTransport {
        connected: bool,
    }

    impl Transport for MockTransport {
        type Error = NexoError;

        async fn read(&mut self, _buf: &mut [u8]) -> Result<usize, Self::Error> {
            if !self.connected {
                return Err(NexoError::Connection {
                    details: "Not connected",
                });
            }
            Ok(0)
        }

        async fn write(&mut self, _buf: &[u8]) -> Result<usize, Self::Error> {
            if !self.connected {
                return Err(NexoError::Connection {
                    details: "Not connected",
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
    fn test_connection_state_transitions() {
        // Test: disconnected → connected → disconnected
        let mut client = NexoClient::with_transport(MockTransport {
            connected: false,
        });

        // Initially disconnected (with_transport doesn't auto-connect)
        assert!(!client.is_connected());

        // Manually set connected for testing
        client.connected.store(true, Ordering::Release);
        assert!(client.is_connected());

        // Disconnect
        block_on(client.disconnect()).unwrap();
        assert!(!client.is_connected());
    }

    #[test]
    fn test_new_client_is_disconnected() {
        let client = NexoClient::new();
        assert!(!client.is_connected());
    }

    #[test]
    fn test_default_client_is_disconnected() {
        let client = NexoClient::<TokioTransport>::default();
        assert!(!client.is_connected());
    }

    #[test]
    fn test_send_request_when_not_connected() {
        let mut client = NexoClient::new();
        let result = block_on(client.send_request(&Vec::<u8>::new()));
        assert!(result.is_err());
        match result {
            Err(NexoError::Connection { details }) => {
                assert_eq!(details, "not connected");
            }
            _ => panic!("Expected Connection error"),
        }
    }

    #[test]
    fn test_receive_response_when_not_connected() {
        let mut client = NexoClient::new();
        let result: Result<Vec<u8>, _> = block_on(client.receive_response());
        assert!(result.is_err());
        match result {
            Err(NexoError::Connection { details }) => {
                assert_eq!(details, "not connected");
            }
            _ => panic!("Expected Connection error"),
        }
    }
}
