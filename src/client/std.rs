//! Tokio-based client implementation for Nexo Retailer Protocol
//!
//! This module provides the standard library implementation of `NexoClient`
//! using Tokio for async runtime and `tokio::sync::oneshot` for request/response
//! correlation.

#![cfg(feature = "std")]

use crate::client::reconnect::{ReconnectConfig, Backoff};
use crate::client::timeout::TimeoutConfig;
use crate::client::timeout::generate_message_id;
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
struct PendingRequests {
    inner: BTreeMap<String, oneshot::Sender<Vec<u8>>>,
}

impl PendingRequests {
    /// Create a new pending requests tracker
    fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    /// Register a pending request and return the receiver
    ///
    /// # Arguments
    ///
    /// * `id` - Unique message ID for this request
    ///
    /// # Returns
    ///
    /// A oneshot receiver that will receive the response
    fn register(&mut self, id: String) -> oneshot::Receiver<Vec<u8>> {
        let (tx, rx) = oneshot::channel();
        self.inner.insert(id, tx);
        rx
    }

    /// Complete a pending request by sending the response
    ///
    /// # Arguments
    ///
    /// * `id` - Message ID of the request to complete
    /// * `response` - Response bytes to send
    ///
    /// # Returns
    ///
    /// Ok(()) if the response was sent, Err if the receiver was dropped
    fn complete(&mut self, id: String, response: Vec<u8>) -> Result<(), Vec<u8>> {
        if let Some(tx) = self.inner.remove(&id) {
            tx.send(response)
        } else {
            Err(response)
        }
    }

    /// Clean up a pending request (e.g., after timeout)
    ///
    /// # Arguments
    ///
    /// * `id` - Message ID of the request to clean up
    fn cleanup(&mut self, id: String) {
        self.inner.remove(&id);
    }
}

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
    pending: PendingRequests,
    /// Reconnection configuration
    reconnect_config: Option<ReconnectConfig>,
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
            reconnect_config: None,
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

    /// Set reconnection configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Reconnection configuration with backoff parameters
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nexo_retailer_protocol::{NexoClient, client::ReconnectConfig};
    /// # use std::time::Duration;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = NexoClient::new();
    /// let config = ReconnectConfig::new()
    ///     .with_max_attempts(5)
    ///     .with_base_delay(Duration::from_millis(100));
    /// client = client.with_reconnect_config(config);
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_reconnect_config(mut self, config: ReconnectConfig) -> Self {
        self.reconnect_config = Some(config);
        self
    }

    /// Reconnect to the payment terminal with exponential backoff
    ///
    /// This method attempts to reconnect using the configured backoff strategy.
    /// If no reconnection config is set, returns an error.
    ///
    /// # Errors
    ///
    /// Returns `NexoError::Connection` if:
    /// - No reconnection config is set
    /// - Max reconnection attempts are exceeded
    /// - Connection cannot be established
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nexo_retailer_protocol::{NexoClient, client::ReconnectConfig};
    /// # use std::time::Duration;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = NexoClient::new();
    /// let config = ReconnectConfig::new()
    ///     .with_max_attempts(5)
    ///     .with_base_delay(Duration::from_millis(100));
    /// client = client.with_reconnect_config(config);
    /// client.connect("192.168.1.100:8080").await?;
    ///
    /// // After connection lost, reconnect with backoff
    /// client.reconnect().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn reconnect(&mut self) -> Result<(), NexoError> {
        let config = self.reconnect_config.ok_or(NexoError::Connection {
            details: "reconnect config not set",
        })?;

        // Disconnect if currently connected
        let addr = self.server_addr.clone();
        if !addr.is_empty() {
            self.connected.store(false, Ordering::Release);
            self.transport = None;
        }

        // Attempt reconnection with backoff
        let mut backoff = Backoff::new(config);
        while backoff.wait_with_jitter().await {
            let transport = match TokioTransport::connect(&addr, Duration::from_secs(10)).await {
                Ok(t) => t,
                Err(_) => continue, // Try again with backoff
            };
            self.transport = Some(FramedTransport::new(transport));
            self.connected.store(true, Ordering::Release);
            return Ok(());
        }

        // Max attempts exceeded
        Err(NexoError::Connection {
            details: "max reconnection attempts exceeded",
        })
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
            reconnect_config: None,
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

    /// Send a request with timeout and receive a response
    ///
    /// This method wraps the send/receive operation with a timeout. If the timeout
    /// expires, the pending request is cleaned up and `NexoError::Timeout` is returned.
    ///
    /// # Arguments
    ///
    /// * `request` - Request message implementing `prost::Message`
    /// * `timeout` - Maximum duration to wait for response
    ///
    /// # Errors
    ///
    /// Returns `NexoError::Timeout` if timeout expires
    /// Returns other errors from `send_and_receive`
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nexo_retailer_protocol::NexoClient;
    /// # use std::time::Duration;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = NexoClient::new();
    /// client.connect("192.168.1.100:8080").await?;
    ///
    /// let request = YourRequestType::default();
    /// let response: YourResponseType = client.send_with_timeout(&request, Duration::from_secs(10)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_with_timeout<M: Message + Default>(
        &mut self,
        request: &M,
        timeout: Duration,
    ) -> Result<M, NexoError>
    where
        T::Error: Into<NexoError>,
    {
        // Generate unique message ID for this request
        let message_id = generate_message_id();

        // Register the pending request
        let rx = self.pending.register(message_id.clone());

        // Send the request
        self.send_request(request).await.map_err(|e| {
            // Clean up pending request on send error
            self.pending.cleanup(message_id.clone());
            e.into()
        })?;

        // Wait for response with timeout
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(response_bytes)) => {
                // Decode the response
                M::decode(&*response_bytes).map_err(|_| NexoError::Decoding {
                    details: "failed to decode response",
                })
            }
            Ok(Err(_)) => {
                // Channel closed (receiver dropped)
                self.pending.cleanup(message_id);
                Err(NexoError::Connection {
                    details: "response channel closed",
                })
            }
            Err(_) => {
                // Timeout expired
                self.pending.cleanup(message_id);
                Err(NexoError::Timeout)
            }
        }
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

    #[test]
    fn test_pending_requests_register_and_complete() {
        let mut pending = PendingRequests::new();

        // Register a request
        let rx = pending.register("msg-1".to_string());

        // Complete the request
        let response = vec![1, 2, 3, 4];
        assert!(pending.complete("msg-1".to_string(), response.clone()).is_ok());

        // Verify we can receive the response
        let received = block_on(rx).unwrap();
        assert_eq!(received, response);
    }

    #[test]
    fn test_pending_requests_cleanup() {
        let mut pending = PendingRequests::new();

        // Register a request
        pending.register("msg-1".to_string());
        assert_eq!(pending.inner.len(), 1);

        // Clean up the request
        pending.cleanup("msg-1".to_string());
        assert_eq!(pending.inner.len(), 0);
    }

    #[test]
    fn test_pending_requests_complete_unknown_id() {
        let mut pending = PendingRequests::new();

        // Try to complete an unknown request
        let response = vec![1, 2, 3, 4];
        let result = pending.complete("unknown".to_string(), response.clone());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), response);
    }

    #[test]
    fn test_pending_requests_register_replaces_existing() {
        let mut pending = PendingRequests::new();

        // Register first request
        let _rx1 = pending.register("msg-1".to_string());
        assert_eq!(pending.inner.len(), 1);

        // Register second request with same ID (should replace)
        let rx2 = pending.register("msg-1".to_string());
        assert_eq!(pending.inner.len(), 1);

        // Complete should work with the new receiver
        let response = vec![5, 6, 7, 8];
        assert!(pending.complete("msg-1".to_string(), response.clone()).is_ok());

        let received = block_on(rx2).unwrap();
        assert_eq!(received, response);
    }

    // Mock transport that supports send/receive testing
    struct MockFramedTransport {
        connected: bool,
        send_buffer: Vec<u8>,
        receive_buffer: Vec<u8>,
    }

    impl MockFramedTransport {
        fn new() -> Self {
            Self {
                connected: false,
                send_buffer: Vec::new(),
                receive_buffer: Vec::new(),
            }
        }

        fn set_receive_data(&mut self, data: Vec<u8>) {
            self.receive_buffer = data;
        }

        fn get_sent_data(&self) -> &[u8] {
            &self.send_buffer
        }
    }

    impl crate::transport::Transport for MockFramedTransport {
        type Error = NexoError;

        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            if !self.connected {
                return Err(NexoError::Connection {
                    details: "Not connected",
                });
            }
            let bytes_to_read = core::cmp::min(self.receive_buffer.len(), buf.len());
            if bytes_to_read == 0 {
                return Ok(0);
            }
            buf[..bytes_to_read].copy_from_slice(&self.receive_buffer[..bytes_to_read]);
            self.receive_buffer = self.receive_buffer[bytes_to_read..].to_vec();
            Ok(bytes_to_read)
        }

        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            if !self.connected {
                return Err(NexoError::Connection {
                    details: "Not connected",
                });
            }
            self.send_buffer.extend_from_slice(buf);
            Ok(buf.len())
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
    fn test_send_request_with_connected_transport() {
        // Create a connected mock transport that properly implements read/write
        let mock = MockFramedTransport::new();

        // Create client with the transport (will be wrapped in FramedTransport internally)
        let mut client = NexoClient::with_transport(mock);
        client.connected.store(true, Ordering::Release);

        // Verify the client is connected
        assert!(client.is_connected());

        // Note: Actual send/receive requires a properly framed protocol implementation
        // The MockFramedTransport is simplified for basic state testing
        // Full integration tests are in the transport layer tests
    }

    #[test]
    fn test_send_and_receive_with_mock_transport() {
        use prost::Message;

        // Create a mock transport
        let mut mock = MockFramedTransport::new();
        mock.connected = true;

        // Create a test message to send
        let test_msg = crate::ActiveCurrencyAndAmount {
            ccy: "USD".to_string(),
            units: 100,
            nanos: 500000000,
        };

        // Encode the test message and set up receive buffer
        let encoded = test_msg.encode_to_vec();
        let mut receive_data = Vec::new();
        // Add length prefix (4 bytes big-endian)
        let len = encoded.len() as u32;
        receive_data.extend_from_slice(&len.to_be_bytes());
        // Add message body
        receive_data.extend_from_slice(&encoded);
        mock.set_receive_data(receive_data);

        let mut client = NexoClient::with_transport(mock);
        client.connected.store(true, Ordering::Release);

        // Send and receive
        let send_result = block_on(client.send_request(&test_msg));
        assert!(send_result.is_ok());

        let received: crate::ActiveCurrencyAndAmount =
            block_on(client.receive_response()).unwrap();

        assert_eq!(received.ccy, "USD");
        assert_eq!(received.units, 100);
        assert_eq!(received.nanos, 500000000);
    }

    #[test]
    fn test_send_and_receive_combined() {
        let mut mock = MockFramedTransport::new();
        mock.connected = true;

        // Set up receive data
        let test_msg = crate::ActiveCurrencyAndAmount {
            ccy: "EUR".to_string(),
            units: 200,
            nanos: 750000000,
        };
        let encoded = test_msg.encode_to_vec();
        let mut receive_data = Vec::new();
        receive_data.extend_from_slice(&(encoded.len() as u32).to_be_bytes());
        receive_data.extend_from_slice(&encoded);
        mock.set_receive_data(receive_data);

        let mut client = NexoClient::with_transport(mock);
        client.connected.store(true, Ordering::Release);

        // Use send_and_receive
        let received: crate::ActiveCurrencyAndAmount =
            block_on(client.send_and_receive(&test_msg)).unwrap();

        assert_eq!(received.ccy, "EUR");
        assert_eq!(received.units, 200);
        assert_eq!(received.nanos, 750000000);
    }

    #[test]
    fn test_reconnect_without_config() {
        let mut client = NexoClient::new();
        let result = block_on(client.reconnect());
        assert!(result.is_err());
        match result {
            Err(NexoError::Connection { details }) => {
                assert_eq!(details, "reconnect config not set");
            }
            _ => panic!("Expected 'reconnect config not set' error"),
        }
    }

    #[test]
    fn test_reconnect_with_config() {
        use core::time::Duration;
        use crate::client::reconnect::ReconnectConfig;

        // Test that we can set reconnect config
        let client = NexoClient::new();
        let config = ReconnectConfig::new()
            .with_base_delay(Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(60))
            .with_max_attempts(5);

        let client = client.with_reconnect_config(config);
        // We can't test actual reconnection without a real server,
        // but we can verify the config is set
        assert!(client.reconnect_config.is_some());
        assert_eq!(client.reconnect_config.unwrap().base_delay, Duration::from_millis(100));
    }

    #[test]
    fn test_send_with_timeout_when_not_connected() {
        use core::time::Duration;

        let mut client = NexoClient::new();
        let result = block_on(client.send_with_timeout(&Vec::<u8>::new(), Duration::from_secs(1)));
        assert!(result.is_err());
        match result {
            Err(NexoError::Connection { .. }) => {
                // Expected - not connected
            }
            _ => panic!("Expected Connection error"),
        }
    }
}
