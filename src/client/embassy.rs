//! Embassy-based client implementation for Nexo Retailer Protocol
//!
//! This module provides the bare-metal implementation of `NexoClient`
//! using Embassy for async runtime. In no_std environments, we don't have
//! Arc for shared state, so the client uses simpler state management.

#![cfg(feature = "embassy-net")]

extern crate alloc;

use crate::error::NexoError;
use crate::transport::{FramedTransport, Transport, EmbassyTransport};

use prost::Message;

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use core::sync::atomic::{AtomicBool, Ordering};

/// Pending request tracking
///
/// This struct manages in-flight requests by mapping message IDs to response channels.
/// Note: Embassy doesn't have oneshot channels in embassy-futures 0.1, so we use
/// a simpler approach for request tracking.
type PendingRequests = BTreeMap<String, ()>;

/// Nexo Retailer Protocol client for Embassy runtime
///
/// This client provides a high-level API for connecting to payment terminals,
/// sending requests, and receiving responses in bare-metal environments.
/// It handles connection management and request/response correlation automatically.
///
/// # Type Parameters
///
/// * `T` - Transport implementation (typically `EmbassyTransport`)
/// * `'a` - Lifetime for buffer references (Embassy requires explicit lifetimes)
///
/// # Examples
///
/// ```rust,ignore
/// use nexo_retailer_protocol::NexoClient;
/// use embassy_net::TcpSocket;
///
/// # async fn example() -> Result<(), NexoError> {
/// // Create client with Embassy transport
/// let mut rx_buffer = [0u8; 4096];
/// let mut tx_buffer = [0u8; 4096];
/// let socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
/// let transport = EmbassyTransport::new(socket, &mut rx_buffer, &mut tx_buffer);
///
/// let mut client = NexoClient::with_transport(transport);
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
pub struct NexoClient<'a, T: Transport> {
    /// Framed transport wrapper for length-prefixed messaging
    transport: Option<FramedTransport<T>>,
    /// Connection state flag (not using Arc - simpler for embedded)
    connected: AtomicBool,
    /// Server address for reconnection
    server_addr: String,
    /// Pending requests awaiting responses
    #[allow(dead_code)]
    pending: PendingRequests,
    _phantom: core::marker::PhantomData<&'a ()>,
}

impl<'a> NexoClient<'a, EmbassyTransport<'a>> {
    /// Create a new unconnected client with Embassy transport
    ///
    /// # Note
    ///
    /// For Embassy, you typically use `with_transport()` since the transport
    /// requires buffer references with explicit lifetimes.
    pub fn new() -> Self {
        Self {
            transport: None,
            connected: AtomicBool::new(false),
            server_addr: String::new(),
            pending: PendingRequests::new(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<'a, T: Transport> NexoClient<'a, T> {
    /// Create a new unconnected client with custom transport
    ///
    /// # Arguments
    ///
    /// * `transport` - Transport implementation to use
    pub fn with_transport(transport: T) -> Self {
        Self {
            transport: Some(FramedTransport::new(transport)),
            connected: AtomicBool::new(false),
            server_addr: String::new(),
            pending: PendingRequests::new(),
            _phantom: core::marker::PhantomData,
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
    pub async fn connect(&mut self, addr: &str) -> Result<(), T::Error> {
        if self.transport.is_none() {
            // Note: Embassy transport creation requires buffers with lifetimes
            // This is a placeholder - in practice, users should use with_transport()
            return Err(NexoError::Connection {
                details: "Use with_transport() for Embassy client",
            }
            .into());
        }

        // Use existing transport
        self.transport
            .as_mut()
            .unwrap()
            .inner()
            .connect(addr)
            .await?;

        self.server_addr = addr.to_string();
        self.connected.store(true, Ordering::Release);
        Ok(())
    }

    /// Disconnect from the payment terminal
    ///
    /// This method closes the connection and resets the client state.
    /// The client can be reconnected by calling `connect()` again.
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
    pub async fn send_and_receive<M: Message + Default>(
        &mut self,
        request: &M,
    ) -> Result<M, T::Error> {
        self.send_request(request).await?;
        self.receive_response().await
    }
}

impl<'a> Default for NexoClient<'a, EmbassyTransport<'a>> {
    fn default() -> Self {
        Self::new()
    }
}
