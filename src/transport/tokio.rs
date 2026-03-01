//! Tokio-based transport implementation for standard environments
//!
//! This module provides a Tokio-specific implementation of the `Transport` trait
//! using `tokio::net::TcpStream` for async TCP networking in server environments.
//!
//! # Features
//!
//! - **Async I/O**: Uses Tokio's async runtime for non-blocking operations
//! - **Timeout support**: Configurable timeouts for connect, read, and write operations
//! - **std feature-gated**: Only available when `std` feature is enabled
//!
//! # Usage
//!
//! ## Client-side connection
//!
//! ```rust,no_run
//! use nexo_retailer_protocol::transport::TokioTransport;
//! use core::time::Duration;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Connect to a remote server
//! let transport = TokioTransport::connect("127.0.0.1:8080", Duration::from_secs(10)).await?;
//!
//! // Verify connection
//! assert!(transport.is_connected());
//! # Ok(())
//! # }
//! ```
//!
//! ## Server-side accepted connection
//!
//! ```rust,no_run
//! use nexo_retailer_protocol::transport::TokioTransport;
//! use tokio::net::TcpListener;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let listener = TcpListener::bind("0.0.0.0:8080").await?;
//!
//! loop {
//!     let (stream, _) = listener.accept().await?;
//!     let transport = TokioTransport::new(stream)
//!         .with_timeouts(
//!             Duration::from_secs(30),
//!             Duration::from_secs(10),
//!         );
//!     // Use transport for communication
//! }
//! # }
//! ```

#![cfg(feature = "std")]

use core::time::Duration;

use crate::error::NexoError;
use crate::transport::Transport;

// Import Tokio I/O traits for read/write methods
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Timeout configuration for transport operations
///
/// Provides a unified timeout configuration structure that can be used across
/// both Tokio and Embassy implementations, with runtime-specific wrapper methods.
///
/// # Default Values
///
/// - Connect timeout: 10 seconds
/// - Read timeout: 30 seconds
/// - Write timeout: 10 seconds
///
/// # Example
///
/// ```rust,no_run
/// use nexo_retailer_protocol::transport::tokio::TimeoutConfig;
/// use core::time::Duration;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = TimeoutConfig::new()
///     .with_connect(Duration::from_secs(5))
///     .with_read(Duration::from_secs(20))
///     .with_write(Duration::from_secs(5));
///
/// // Use with timeout wrapper methods
/// let result = config.with_read_timeout(async {
///     // Some async operation
///     Ok::<(), Box<dyn std::error::Error>>(())
/// }).await;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct TimeoutConfig {
    /// Timeout for connection operations
    pub connect_timeout: Duration,

    /// Timeout for read operations
    pub read_timeout: Duration,

    /// Timeout for write operations
    pub write_timeout: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            read_timeout: Duration::from_secs(30),
            write_timeout: Duration::from_secs(10),
        }
    }
}

impl TimeoutConfig {
    /// Create a new TimeoutConfig with default values
    ///
    /// # Default Values
    ///
    /// - Connect timeout: 10 seconds
    /// - Read timeout: 30 seconds
    /// - Write timeout: 10 seconds
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexo_retailer_protocol::transport::tokio::TimeoutConfig;
    ///
    /// let config = TimeoutConfig::new();
    /// assert_eq!(config.connect_timeout.as_secs(), 10);
    /// assert_eq!(config.read_timeout.as_secs(), 30);
    /// assert_eq!(config.write_timeout.as_secs(), 10);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the connect timeout
    ///
    /// # Arguments
    ///
    /// * `d` - Duration to wait for connection
    ///
    /// # Returns
    ///
    /// Self for builder pattern chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexo_retailer_protocol::transport::tokio::TimeoutConfig;
    /// use core::time::Duration;
    ///
    /// let config = TimeoutConfig::new().with_connect(Duration::from_secs(5));
    /// assert_eq!(config.connect_timeout.as_secs(), 5);
    /// ```
    pub fn with_connect(mut self, d: Duration) -> Self {
        self.connect_timeout = d;
        self
    }

    /// Set the read timeout
    ///
    /// # Arguments
    ///
    /// * `d` - Duration to wait for read operations
    ///
    /// # Returns
    ///
    /// Self for builder pattern chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexo_retailer_protocol::transport::tokio::TimeoutConfig;
    /// use core::time::Duration;
    ///
    /// let config = TimeoutConfig::new().with_read(Duration::from_secs(20));
    /// assert_eq!(config.read_timeout.as_secs(), 20);
    /// ```
    pub fn with_read(mut self, d: Duration) -> Self {
        self.read_timeout = d;
        self
    }

    /// Set the write timeout
    ///
    /// # Arguments
    ///
    /// * `d` - Duration to wait for write operations
    ///
    /// # Returns
    ///
    /// Self for builder pattern chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexo_retailer_protocol::transport::tokio::TimeoutConfig;
    /// use core::time::Duration;
    ///
    /// let config = TimeoutConfig::new().with_write(Duration::from_secs(5));
    /// assert_eq!(config.write_timeout.as_secs(), 5);
    /// ```
    pub fn with_write(mut self, d: Duration) -> Self {
        self.write_timeout = d;
        self
    }

    /// Wrap an async operation with a read timeout
    ///
    /// # Arguments
    ///
    /// * `f` - Async function to execute with timeout
    ///
    /// # Returns
    ///
    /// The result of the async operation, or `NexoError::Timeout` if it exceeds the read timeout
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use nexo_retailer_protocol::transport::tokio::TimeoutConfig;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = TimeoutConfig::new();
    ///
    /// let result = config.with_read_timeout(async {
    ///     // Some async read operation
    ///     Ok::<usize, Box<dyn std::error::Error>>(42)
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn with_read_timeout<F, T>(&self, f: F) -> Result<T, NexoError>
    where
        F: core::future::Future<Output = Result<T, NexoError>>,
    {
        tokio::time::timeout(self.read_timeout, f)
            .await
            .map_err(|_| NexoError::Timeout)?
    }

    /// Wrap an async operation with a write timeout
    ///
    /// # Arguments
    ///
    /// * `f` - Async function to execute with timeout
    ///
    /// # Returns
    ///
    /// The result of the async operation, or `NexoError::Timeout` if it exceeds the write timeout
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use nexo_retailer_protocol::transport::tokio::TimeoutConfig;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = TimeoutConfig::new();
    ///
    /// let result = config.with_write_timeout(async {
    ///     // Some async write operation
    ///     Ok::<usize, Box<dyn std::error::Error>>(42)
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn with_write_timeout<F, T>(&self, f: F) -> Result<T, NexoError>
    where
        F: core::future::Future<Output = Result<T, NexoError>>,
    {
        tokio::time::timeout(self.write_timeout, f)
            .await
            .map_err(|_| NexoError::Timeout)?
    }

    /// Wrap an async operation with a connect timeout
    ///
    /// # Arguments
    ///
    /// * `f` - Async function to execute with timeout
    ///
    /// # Returns
    ///
    /// The result of the async operation, or `NexoError::Timeout` if it exceeds the connect timeout
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use nexo_retailer_protocol::transport::tokio::TimeoutConfig;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = TimeoutConfig::new();
    ///
    /// let result = config.with_connect_timeout(async {
    ///     // Some async connect operation
    ///     Ok::<(), Box<dyn std::error::Error>>(())
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn with_connect_timeout<F, T>(&self, f: F) -> Result<T, NexoError>
    where
        F: core::future::Future<Output = Result<T, NexoError>>,
    {
        tokio::time::timeout(self.connect_timeout, f)
            .await
            .map_err(|_| NexoError::Timeout)?
    }
}

/// Tokio-based transport implementation
///
/// This struct wraps `tokio::net::TcpStream` and implements the `Transport` trait,
/// providing async TCP networking with timeout support for standard environments.
///
/// # Fields
///
/// * `stream` - The underlying Tokio TCP stream
/// * `read_timeout` - Maximum duration to wait for read operations
/// * `write_timeout` - Maximum duration to wait for write operations
pub struct TokioTransport {
    /// Underlying Tokio TCP stream
    stream: tokio::net::TcpStream,

    /// Timeout for read operations
    read_timeout: Duration,

    /// Timeout for write operations
    write_timeout: Duration,
}

impl TokioTransport {
    /// Connect to a remote address with a timeout
    ///
    /// # Arguments
    ///
    /// * `addr` - Remote address in format "host:port" (e.g., "192.168.1.100:8080")
    /// * `timeout` - Maximum duration to wait for connection
    ///
    /// # Returns
    ///
    /// A connected `TokioTransport` instance
    ///
    /// # Errors
    ///
    /// Returns `NexoError` if:
    /// - The address cannot be parsed
    /// - The connection cannot be established
    /// - The connection attempt times out
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use nexo_retailer_protocol::transport::TokioTransport;
    /// use core::time::Duration;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let transport = TokioTransport::connect("127.0.0.1:8080", Duration::from_secs(5)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(addr: &str, timeout: Duration) -> Result<Self, NexoError> {
        // Parse the address
        let socket_addr = addr.parse::<std::net::SocketAddr>().map_err(|_| {
            NexoError::Connection {
                details: "invalid address format",
            }
        })?;

        // Use timeout wrapper for connection
        let stream = tokio::time::timeout(timeout, tokio::net::TcpStream::connect(socket_addr))
            .await
            .map_err(|_| NexoError::Timeout)?
            .map_err(|e| NexoError::Connection {
                details: Box::leak(e.to_string().into_boxed_str()),
            })?;

        Ok(Self {
            stream,
            read_timeout: Duration::from_secs(30),
            write_timeout: Duration::from_secs(10),
        })
    }

    /// Create a new TokioTransport from an existing TcpStream
    ///
    /// This constructor is useful for server-side usage where you have an
    /// accepted connection from a `TcpListener`.
    ///
    /// # Arguments
    ///
    /// * `stream` - An already-connected Tokio TCP stream
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use nexo_retailer_protocol::transport::TokioTransport;
    /// use tokio::net::TcpListener;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let listener = TcpListener::bind("0.0.0.0:8080").await?;
    /// let (stream, _) = listener.accept().await?;
    /// let transport = TokioTransport::new(stream);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(stream: tokio::net::TcpStream) -> Self {
        Self {
            stream,
            read_timeout: Duration::from_secs(30),
            write_timeout: Duration::from_secs(10),
        }
    }

    /// Set custom timeouts for read and write operations
    ///
    /// This method uses the builder pattern to allow fluent configuration.
    ///
    /// # Arguments
    ///
    /// * `read` - Timeout for read operations
    /// * `write` - Timeout for write operations
    ///
    /// # Returns
    ///
    /// Self with updated timeouts
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use nexo_retailer_protocol::transport::TokioTransport;
    /// use core::time::Duration;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let transport = TokioTransport::connect("127.0.0.1:8080", Duration::from_secs(5))
    ///     .await?
    ///     .with_timeouts(Duration::from_secs(30), Duration::from_secs(10));
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_timeouts(mut self, read: Duration, write: Duration) -> Self {
        self.read_timeout = read;
        self.write_timeout = write;
        self
    }
}

impl Transport for TokioTransport {
    type Error = NexoError;

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        tokio::time::timeout(self.read_timeout, self.stream.read(buf))
            .await
            .map_err(|_| NexoError::Timeout)?
            .map_err(|e| {
                NexoError::Connection {
                    details: Box::leak(e.to_string().into_boxed_str()),
                }
                .into()
            })
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        tokio::time::timeout(self.write_timeout, self.stream.write(buf))
            .await
            .map_err(|_| NexoError::Timeout)?
            .map_err(|e| {
                NexoError::Connection {
                    details: Box::leak(e.to_string().into_boxed_str()),
                }
                .into()
            })
    }

    async fn connect(&mut self, addr: &str) -> Result<(), Self::Error> {
        let socket_addr = addr.parse::<std::net::SocketAddr>().map_err(|_| {
            NexoError::Connection {
                details: "invalid address format",
            }
        })?;

        self.stream = tokio::net::TcpStream::connect(socket_addr).await.map_err(|e| {
            NexoError::Connection {
                details: Box::leak(e.to_string().into_boxed_str()),
            }
        })?;

        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.stream.peer_addr().is_ok()
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    // Import ToString for no_std tests with alloc
    #[cfg(feature = "alloc")]
    use prost::alloc::string::ToString;

    // Helper to get a random available port
    fn get_available_port() -> u16 {
        // Use port 0 to let the OS assign an available port
        0
    }

    #[tokio::test]
    async fn test_tokio_transport_connect_to_echo_server() {
        // Start a simple echo server
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();

        // Spawn echo server task
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            loop {
                let n = socket.read(&mut buf).await.unwrap();
                if n == 0 {
                    break;
                }
                socket.write_all(&buf[..n]).await.unwrap();
            }
        });

        // Connect client
        let mut transport =
            TokioTransport::connect(&local_addr.to_string(), Duration::from_secs(5))
                .await
                .unwrap();

        // Verify connection
        assert!(transport.is_connected());

        // Send and receive data
        let test_data = b"Hello, Nexo!";
        transport.write(test_data).await.unwrap();

        let mut recv_buf = [0u8; 1024];
        let n = transport.read(&mut recv_buf).await.unwrap();
        assert_eq!(&recv_buf[..n], test_data);
    }

    #[tokio::test]
    async fn test_tokio_transport_read_timeout() {
        // Start a server that never sends data
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (_socket, _) = listener.accept().await.unwrap();
            // Don't send anything - just hold the connection
            tokio::time::sleep(Duration::from_secs(10)).await;
        });

        // Connect with very short read timeout
        let mut transport = TokioTransport::connect(&local_addr.to_string(), Duration::from_secs(5))
            .await
            .unwrap()
            .with_timeouts(Duration::from_millis(100), Duration::from_secs(10));

        // Attempt to read should timeout
        let mut buf = [0u8; 1024];
        let result = transport.read(&mut buf).await;
        assert!(matches!(result, Err(NexoError::Timeout)));
    }

    #[tokio::test]
    async fn test_tokio_transport_write_timeout() {
        // Start a server that doesn't read from the socket
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (_socket, _) = listener.accept().await.unwrap();
            // Don't read - just hold the connection to fill buffer
            tokio::time::sleep(Duration::from_secs(10)).await;
        });

        // Connect with short write timeout
        let mut transport = TokioTransport::connect(&local_addr.to_string(), Duration::from_secs(5))
            .await
            .unwrap()
            .with_timeouts(Duration::from_secs(30), Duration::from_millis(100));

        // Write huge amount of data to fill buffer and timeout
        let huge_data = vec![0u8; 1024 * 1024 * 100]; // 100 MB
        let result = transport.write(&huge_data).await;

        // Should either succeed (wrote some bytes) or timeout
        // We're mainly checking that it doesn't hang forever
        match result {
            Ok(_) | Err(NexoError::Timeout) | Err(NexoError::Connection { .. }) => {
                // All acceptable outcomes
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_tokio_transport_partial_read() {
        // Start a server that sends data in chunks
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            // Send in two chunks to test partial read handling
            socket.write_all(b"PART1").await.unwrap();
            tokio::time::sleep(Duration::from_millis(50)).await;
            socket.write_all(b"PART2").await.unwrap();
        });

        // Connect and read
        let mut transport =
            TokioTransport::connect(&local_addr.to_string(), Duration::from_secs(5))
                .await
                .unwrap();

        let mut buf = [0u8; 1024];
        let mut total_read = 0;

        // Read in loop to get all data
        loop {
            let n = transport.read(&mut buf[total_read..]).await.unwrap();
            if n == 0 {
                break;
            }
            total_read += n;
            if total_read >= 10 {
                break;
            }
        }

        assert_eq!(total_read, 10);
        assert_eq!(&buf[..total_read], b"PART1PART2");
    }

    #[tokio::test]
    async fn test_tokio_transport_is_connected() {
        // Test with active connection
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (_socket, _) = listener.accept().await.unwrap();
            tokio::time::sleep(Duration::from_secs(5)).await;
        });

        let transport =
            TokioTransport::connect(&local_addr.to_string(), Duration::from_secs(5))
                .await
                .unwrap();

        // Active connection should report as connected
        assert!(transport.is_connected());

        // Test that we can create a transport and check its state
        let listener2 = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr2 = listener2.local_addr().unwrap();

        tokio::spawn(async move {
            let (_socket, _) = listener2.accept().await.unwrap();
            tokio::time::sleep(Duration::from_secs(1)).await;
        });

        let transport2 =
            TokioTransport::connect(&local_addr2.to_string(), Duration::from_secs(5))
                .await
                .unwrap();

        // Should be connected after successful connect
        assert!(transport2.is_connected());
    }
}
