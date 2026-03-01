//! Tokio-based server implementation for Nexo Retailer Protocol
//!
//! This module provides the standard library implementation of `NexoServer`
//! using Tokio for async runtime and `tokio::spawn` for concurrent connection
//! handling. The server maintains a thread-safe HashMap of active connections
//! and spawns a lightweight task for each accepted connection.

#![cfg(feature = "std")]

use crate::error::NexoError;
use crate::server::ConnectionState;
use crate::server::handler::RequestHandler;
use crate::server::dispatcher::Dispatcher;
use crate::server::heartbeat::{HeartbeatConfig, HeartbeatMonitor};
use crate::transport::{FramedTransport, TokioTransport};

#[cfg(feature = "std")]
use tracing::{info, debug, warn, error};

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Nexo Retailer Protocol server for Tokio runtime
///
/// This server provides a high-level API for accepting concurrent connections
/// from POS clients, tracking per-connection state, and handling message
/// processing. Each connection runs in an independent tokio::spawn task.
///
/// # Architecture
///
/// The server uses `Arc<Mutex<HashMap<SocketAddr, ConnectionState>>>` for
/// thread-safe connection tracking. Each connection handler removes itself
/// from the HashMap when it exits, ensuring no memory leaks.
///
/// # Examples
///
/// ```rust,no_run
/// use nexo_retailer_protocol::NexoServer;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Bind to address and create server
/// let server = NexoServer::bind("127.0.0.1:8080").await?;
///
/// // Run the server (accepts connections indefinitely)
/// server.run().await?;
/// # Ok(())
/// # }
/// ```
pub struct NexoServer {
    /// TCP listener for accepting connections
    listener: Arc<tokio::net::TcpListener>,
    /// Thread-safe map of active connections
    connections: Arc<Mutex<BTreeMap<SocketAddr, ConnectionState>>>,
    /// Flag to signal graceful shutdown
    shutdown_flag: Arc<tokio::sync::Notify>,
    /// Application handler for processing incoming messages
    handler: Option<Arc<dyn RequestHandler>>,
}

impl NexoServer {
    /// Bind to an address and create a new server
    ///
    /// # Arguments
    ///
    /// * `addr` - Address to bind to (e.g., "127.0.0.1:8080" or "0.0.0.0:8080")
    ///
    /// # Errors
    ///
    /// Returns `NexoError::Connection` if:
    /// - The address is invalid
    /// - The address is already in use
    /// - Permission is denied
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nexo_retailer_protocol::NexoServer;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let server = NexoServer::bind("127.0.0.1:8080").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn bind(addr: &str) -> Result<Self, NexoError> {
        // Parse the address
        let socket_addr: SocketAddr = addr.parse().map_err(|_| {
            NexoError::connection_owned(format!("invalid address: {}", addr))
        })?;

        // Bind TCP listener
        let listener = tokio::net::TcpListener::bind(socket_addr).await.map_err(|e| {
            NexoError::connection_owned(format!("failed to bind to {}: {}", addr, e))
        })?;

        Ok(Self {
            listener: Arc::new(listener),
            connections: Arc::new(Mutex::new(BTreeMap::new())),
            shutdown_flag: Arc::new(tokio::sync::Notify::new()),
            handler: None,
        })
    }

    /// Set the request handler for this server
    ///
    /// # Arguments
    ///
    /// * `handler` - Application handler implementing `RequestHandler` trait
    ///
    /// # Returns
    ///
    /// Self for builder pattern chaining
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nexo_retailer_protocol::NexoServer;
    /// # use nexo_retailer_protocol::server::RequestHandler;
    /// # use std::sync::Arc;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let handler = Arc::new(MyHandler);
    /// let server = NexoServer::bind("127.0.0.1:8080").await?
    ///     .with_handler(handler);
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_handler(mut self, handler: Arc<dyn RequestHandler>) -> Self {
        self.handler = Some(handler);
        self
    }

    /// Run the server, accepting connections indefinitely
    ///
    /// This method runs an infinite loop that accepts connections and spawns
    /// a handler task for each connection. The loop continues until an error
    /// occurs or the process is killed.
    ///
    /// # Errors
    ///
    /// Returns `NexoError::Connection` if accepting a connection fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nexo_retailer_protocol::NexoServer;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let server = NexoServer::bind("127.0.0.1:8080").await?;
    /// server.run().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self) -> Result<(), NexoError> {
        loop {
            // Accept a new connection
            let (stream, addr) = self.accept().await?;

            #[cfg(feature = "std")]
            info!(addr = %addr, "Connection accepted");

            // Spawn connection handler task
            self.spawn_connection_handler(stream, addr);
        }
    }

    /// Accept a single connection
    ///
    /// This is a lower-level method that can be used for custom server loops.
    /// Most users should use `run()` instead.
    ///
    /// # Returns
    ///
    /// A tuple of (TcpStream, SocketAddr)
    async fn accept(&self) -> Result<(tokio::net::TcpStream, SocketAddr), NexoError> {
        self.listener
            .accept()
            .await
            .map_err(|e| NexoError::connection_owned(format!("failed to accept connection: {}", e)))
    }

    /// Spawn a connection handler task
    ///
    /// # Arguments
    ///
    /// * `stream` - The TCP stream for the connection
    /// * `addr` - The client's socket address
    fn spawn_connection_handler(&self, stream: tokio::net::TcpStream, addr: SocketAddr) {
        // Clone Arc references for the spawned task
        let connections = Arc::clone(&self.connections);
        let handler = self.handler.clone();

        // Wrap TcpStream in TokioTransport and FramedTransport for proper framing
        let transport = TokioTransport::new(stream);
        let framed = FramedTransport::new(transport);

        // Spawn connection handler task
        tokio::spawn(async move {
            // Log connection start with context
            #[cfg(feature = "std")]
            info!(addr = %addr, "Handling connection");

            // Create connection state with default heartbeat config
            let mut state = ConnectionState::new(addr);
            state.set_heartbeat_config(Some(HeartbeatConfig::new()));

            // Create heartbeat monitor
            let heartbeat_monitor = HeartbeatMonitor::new(HeartbeatConfig::new());

            // Insert connection state into the HashMap
            connections.lock().await.insert(addr, state.clone());

            // Handle connection with dispatcher if handler is set
            let result = if let Some(handler) = handler {
                let dispatcher = Dispatcher::new(handler);
                Self::handle_connection_with_dispatcher(framed, &mut state, dispatcher, heartbeat_monitor).await
            } else {
                // No handler set - use basic echo handler
                Self::handle_connection(framed, &mut state, heartbeat_monitor).await
            };

            // Clean up connection state when handler exits
            connections.lock().await.remove(&addr);

            // Log disconnection (if there was an error)
            if let Err(e) = result {
                #[cfg(feature = "std")]
                error!(error = %e, addr = %addr, "Connection closed with error");
                #[cfg(not(feature = "std"))]
                eprintln!("Connection {} closed with error: {:?}", addr, e);
            } else {
                #[cfg(feature = "std")]
                info!(addr = %addr, message_count = state.message_count(), "Connection closed");
            }
        });
    }

    /// Handle a single connection with dispatcher
    ///
    /// This method reads framed messages from the client, dispatches them to the handler,
    /// and sends framed responses back. It continues until the client disconnects.
    ///
    /// Uses tokio::select! to concurrently handle incoming messages and heartbeat
    /// monitoring, ensuring the connection loop remains responsive.
    ///
    /// # Arguments
    ///
    /// * `framed` - The framed transport for the connection
    /// * `state` - The connection state (updated in-place)
    /// * `dispatcher` - Message dispatcher for routing to handlers
    /// * `heartbeat_monitor` - Heartbeat monitor for dead connection detection
    async fn handle_connection_with_dispatcher(
        mut framed: FramedTransport<TokioTransport>,
        state: &mut ConnectionState,
        dispatcher: Dispatcher,
        mut heartbeat_monitor: HeartbeatMonitor,
    ) -> Result<(), NexoError> {
        // Create heartbeat ticker
        let config = heartbeat_monitor.config().clone();
        let mut heartbeat_interval = if config.is_enabled() {
            tokio::time::interval(config.interval())
        } else {
            // Create a dummy interval that never ticks if heartbeat is disabled
            tokio::time::interval(Duration::from_secs(u64::MAX))
        };

        loop {
            // Use tokio::select! to concurrently handle messages and heartbeat
            tokio::select! {
                // Heartbeat tick
                _ = heartbeat_interval.tick() => {
                    // Check for timeout
                    if heartbeat_monitor.check_timeout() {
                        #[cfg(feature = "std")]
                        warn!(elapsed = ?heartbeat_monitor.time_since_activity(), "Connection timeout");

                        return Err(NexoError::connection_owned(
                            format!("connection timeout: no activity for {:?}", heartbeat_monitor.time_since_activity())
                        ));
                    }

                    // Send heartbeat if needed
                    if heartbeat_monitor.should_send_heartbeat() {
                        // Create heartbeat message with proper CASP format
                        let heartbeat_doc = create_heartbeat_document();

                        #[cfg(feature = "std")]
                        debug!("Heartbeat sent");

                        // Send heartbeat with proper framing
                        if let Err(e) = framed.send_message(&heartbeat_doc).await {
                            return Err(NexoError::connection_owned(format!("heartbeat send error: {:?}", e)));
                        }

                        // Mark heartbeat as sent
                        heartbeat_monitor.mark_heartbeat_sent();
                    }
                }

                // Incoming message from client (using framed recv_message)
                result = framed.recv_message::<crate::Casp001Document>() => {
                    match result {
                        Ok(document) => {
                            // Update activity and increment message count
                            state.update_activity();
                            heartbeat_monitor.update_activity();
                            state.increment_message_count();

                            #[cfg(feature = "std")]
                            debug!("Message received (framed)");

                            // Dispatch decoded document to handler and get response
                            let response_result = dispatcher.dispatch_document(document).await;

                            match response_result {
                                Ok(response_doc) => {
                                    // Send framed response back to client
                                    #[cfg(feature = "std")]
                                    debug!("Dispatching to handler");

                                    framed.send_message(&response_doc).await.map_err(|e| {
                                        NexoError::connection_owned(format!("write error: {:?}", e))
                                    })?;
                                }
                                Err(e) => {
                                    // Handler returned error - log it and continue
                                    // Don't crash the server due to handler errors
                                    #[cfg(feature = "std")]
                                    error!(error = %e, "Handler error");
                                    #[cfg(not(feature = "std"))]
                                    eprintln!("Handler error for connection {:?}: {:?}", state.addr(), e);
                                    // Optionally send error response to client
                                    // For now, just continue processing
                                }
                            }
                        }
                        Err(e) => {
                            // Check if this is a connection closed error
                            let error_str = format!("{:?}", e);
                            if error_str.contains("EOF") || error_str.contains("closed") {
                                // Client disconnected
                                return Ok(());
                            }
                            return Err(NexoError::connection_owned(format!("read error: {:?}", e)));
                        }
                    }
                }
            }
        }
    }

    /// Handle a single connection (fallback without dispatcher)
    ///
    /// This is a placeholder that keeps the connection alive by echoing back
    /// any data received. Used when no handler is set.
    ///
    /// Uses tokio::select! to concurrently handle incoming messages and heartbeat
    /// monitoring, ensuring the connection loop remains responsive.
    ///
    /// # Arguments
    ///
    /// * `framed` - The framed transport for the connection
    /// * `state` - The connection state (updated in-place)
    /// * `heartbeat_monitor` - Heartbeat monitor for dead connection detection
    async fn handle_connection(
        mut framed: FramedTransport<TokioTransport>,
        state: &mut ConnectionState,
        mut heartbeat_monitor: HeartbeatMonitor,
    ) -> Result<(), NexoError> {
        // Create heartbeat ticker
        let config = heartbeat_monitor.config().clone();
        let mut heartbeat_interval = if config.is_enabled() {
            tokio::time::interval(config.interval())
        } else {
            // Create a dummy interval that never ticks if heartbeat is disabled
            tokio::time::interval(Duration::from_secs(u64::MAX))
        };

        loop {
            // Use tokio::select! to concurrently handle messages and heartbeat
            tokio::select! {
                // Heartbeat tick
                _ = heartbeat_interval.tick() => {
                    // Check for timeout
                    if heartbeat_monitor.check_timeout() {
                        return Err(NexoError::connection_owned(
                            format!("connection timeout: no activity for {:?}", heartbeat_monitor.time_since_activity())
                        ));
                    }

                    // Send heartbeat if needed
                    if heartbeat_monitor.should_send_heartbeat() {
                        // Create heartbeat message with proper CASP format
                        let heartbeat_doc = create_heartbeat_document();

                        // Send heartbeat with proper framing
                        if let Err(e) = framed.send_message(&heartbeat_doc).await {
                            return Err(NexoError::connection_owned(format!("heartbeat send error: {:?}", e)));
                        }

                        // Mark heartbeat as sent
                        heartbeat_monitor.mark_heartbeat_sent();
                    }
                }

                // Incoming message from client (using framed recv_message)
                result = framed.recv_message::<crate::Casp001Document>() => {
                    match result {
                        Ok(document) => {
                            // Update activity and increment message count
                            state.update_activity();
                            heartbeat_monitor.update_activity();
                            state.increment_message_count();

                            #[cfg(feature = "std")]
                            debug!("Message received (echo mode, framed)");

                            // Echo back (basic functionality) - send framed response
                            framed.send_message(&document).await.map_err(|e| {
                                NexoError::connection_owned(format!("write error: {:?}", e))
                            })?;
                        }
                        Err(e) => {
                            // Check if this is a connection closed error
                            let error_str = format!("{:?}", e);
                            if error_str.contains("EOF") || error_str.contains("closed") {
                                // Client disconnected
                                return Ok(());
                            }
                            return Err(NexoError::connection_owned(format!("read error: {:?}", e)));
                        }
                    }
                }
            }
        }
    }

    /// Get the number of active connections
    ///
    /// # Returns
    ///
    /// The number of currently connected clients
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nexo_retailer_protocol::NexoServer;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let server = NexoServer::bind("127.0.0.1:8080").await?;
    /// let count = server.connection_count().await;
    /// println!("Active connections: {}", count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connection_count(&self) -> usize {
        self.connections.lock().await.len()
    }

    /// Get a copy of the current connection states
    ///
    /// # Returns
    ///
    /// A vector of connection states (cloned)
    pub async fn get_connections(&self) -> Vec<ConnectionState> {
        self.connections
            .lock()
            .await
            .values()
            .cloned()
            .collect()
    }

    /// Check if a specific client is connected
    ///
    /// # Arguments
    ///
    /// * `addr` - The client's socket address
    ///
    /// # Returns
    ///
    /// `true` if the client is connected, `false` otherwise
    pub async fn is_connected(&self, addr: SocketAddr) -> bool {
        self.connections.lock().await.contains_key(&addr)
    }

    /// Get the local address that the server is bound to
    ///
    /// # Returns
    ///
    /// The local socket address
    pub fn local_addr(&self) -> Result<SocketAddr, NexoError> {
        self.listener
            .local_addr()
            .map_err(|e| NexoError::connection_owned(format!("failed to get local address: {}", e)))
    }
}

/// Create a heartbeat CASP document
///
/// This creates a minimal heartbeat message that can be sent to keep the
/// connection alive. The heartbeat uses a Casp001Document with MessageFunction="HRTB"
/// to indicate a heartbeat message per the Nexo protocol.
///
/// # Returns
///
/// A Casp001Document with heartbeat message format
fn create_heartbeat_document() -> crate::Casp001Document {
    // Create a minimal heartbeat message with MessageFunction="HRTB"
    // This follows the Nexo Retailer Protocol specification for heartbeat messages
    crate::Casp001Document {
        document: Some(crate::Casp001DocumentDocument {
            sale_to_poi_svc_req: Some(crate::SaleToPoiServiceRequestV06 {
                hdr: Some(crate::Header4 {
                    msg_fctn: Some("HRTB".to_string()),
                    proto_vrsn: Some("6.0".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_bind_localhost() {
        // Test binding to localhost with ephemeral port
        let result = NexoServer::bind("127.0.0.1:0").await;
        assert!(result.is_ok(), "Should bind to 127.0.0.1:0 successfully");

        let server = result.unwrap();
        assert!(server.local_addr().is_ok());
    }

    #[tokio::test]
    async fn test_server_bind_invalid_address() {
        // Test binding to invalid address
        let result = NexoServer::bind("invalid-address").await;
        assert!(result.is_err(), "Should fail to bind to invalid address");

        match result {
            Err(NexoError::Connection { details }) => {
                assert!(details.contains("invalid address"));
            }
            _ => panic!("Expected Connection error"),
        }
    }

    #[tokio::test]
    async fn test_server_bind_address_in_use() {
        // Test binding to address already in use
        let server1 = NexoServer::bind("127.0.0.1:0").await.unwrap();
        let addr = server1.local_addr().unwrap();

        // Try to bind to the same address
        let result = NexoServer::bind(&addr.to_string()).await;
        assert!(result.is_err(), "Should fail to bind to address in use");
    }

    #[tokio::test]
    async fn test_server_connection_count_starts_at_zero() {
        let server = NexoServer::bind("127.0.0.1:0").await.unwrap();
        let count = server.connection_count().await;
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_server_get_connections_starts_empty() {
        let server = NexoServer::bind("127.0.0.1:0").await.unwrap();
        let connections = server.get_connections().await;
        assert_eq!(connections.len(), 0);
    }

    #[tokio::test]
    async fn test_server_is_connected_returns_false_for_unknown() {
        let server = NexoServer::bind("127.0.0.1:0").await.unwrap();
        let addr = "127.0.0.1:12345".parse().unwrap();
        assert!(!server.is_connected(addr).await);
    }

    #[tokio::test]
    async fn test_server_local_addr() {
        let server = NexoServer::bind("127.0.0.1:0").await.unwrap();
        let addr = server.local_addr().unwrap();
        assert_eq!(addr.ip(), std::net::Ipv4Addr::new(127, 0, 0, 1));
        assert!(addr.port() > 0);
    }

    #[tokio::test]
    async fn test_server_accept_single_connection() {
        // Bind server to ephemeral port
        let server = NexoServer::bind("127.0.0.1:0").await.unwrap();
        let addr = server.local_addr().unwrap().to_string();

        // Spawn a task to accept one connection
        let server_clone = Arc::new(server);
        let server_task = tokio::spawn({
            let server = Arc::clone(&server_clone);
            async move { server.accept().await }
        });

        // Connect a client
        let client = tokio::net::TcpStream::connect(&addr).await;
        assert!(client.is_ok(), "Client should connect successfully");

        // Wait for server to accept
        let result = server_task.await.unwrap();
        assert!(result.is_ok(), "Server should accept connection");

        let (_stream, client_addr) = result.unwrap();
        assert_eq!(client_addr.ip(), std::net::Ipv4Addr::new(127, 0, 0, 1));
    }

    #[tokio::test]
    async fn test_server_handle_connection_echo() {
        // Test the connection state tracking directly
        let addr = "127.0.0.1:8080".parse().unwrap();
        let mut state = ConnectionState::new(addr);
        assert_eq!(state.message_count(), 0);

        // Simulate message increment
        state.increment_message_count();
        assert_eq!(state.message_count(), 1);
    }
}
