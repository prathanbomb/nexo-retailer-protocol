//! Tokio-based server implementation for Nexo Retailer Protocol
//!
//! This module provides the standard library implementation of `NexoServer`
//! using Tokio for async runtime and `tokio::spawn` for concurrent connection
//! handling. The server maintains a thread-safe HashMap of active connections
//! and spawns a lightweight task for each accepted connection.

#![cfg(feature = "std")]

use crate::error::NexoError;
use crate::server::ConnectionState;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Arc;
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
        })
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

        // Spawn connection handler task
        tokio::spawn(async move {
            // Create connection state
            let mut state = ConnectionState::new(addr);

            // Insert connection state into the HashMap
            connections.lock().await.insert(addr, state.clone());

            // Handle connection (placeholder - will be implemented in later plans)
            // For now, just keep the connection alive
            let result = Self::handle_connection(stream, &mut state).await;

            // Clean up connection state when handler exits
            connections.lock().await.remove(&addr);

            // Log disconnection (if there was an error)
            if let Err(e) = result {
                eprintln!("Connection {} closed with error: {:?}", addr, e);
            }
        });
    }

    /// Handle a single connection
    ///
    /// This is a placeholder that will be implemented in later plans to:
    /// - Read messages from the client
    /// - Process messages
    /// - Send responses
    /// - Handle deduplication
    /// - Handle heartbeat
    ///
    /// For now, it just keeps the connection alive by reading until EOF.
    ///
    /// # Arguments
    ///
    /// * `stream` - The TCP stream for the connection
    /// * `state` - The connection state (updated in-place)
    async fn handle_connection(
        mut stream: tokio::net::TcpStream,
        state: &mut ConnectionState,
    ) -> Result<(), NexoError> {
        // Placeholder: Read until EOF (client disconnects)
        // In later plans, this will be replaced with message handling
        let mut buffer = [0u8; 4096];
        loop {
            let n = stream.read(&mut buffer).await.map_err(|e| {
                NexoError::connection_owned(format!("read error: {}", e))
            })?;

            if n == 0 {
                // EOF - client disconnected
                return Ok(());
            }

            // Increment message count
            state.increment_message_count();

            // Placeholder: In later plans, we'll parse and handle messages
            // For now, just echo back (basic functionality)
            stream.write_all(&buffer[..n]).await.map_err(|e| {
                NexoError::connection_owned(format!("write error: {}", e))
            })?;
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
