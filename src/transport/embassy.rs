//! Embassy-based transport implementation for embedded (no_std) environments
//!
//! This module provides a transport implementation using Embassy's async runtime
//! and TCP stack for bare-metal embedded devices. It uses `embassy_net::TcpSocket`
//! for network operations and `embassy_futures::select` for timeout handling.
//!
//! # Features
//!
//! - **no_std compatible**: Works in bare-metal environments
//! - **Timeout support**: Uses `embassy_futures::select` for timeout enforcement
//! - **Async I/O**: Full async support using Embassy's executor
//!
//! # Example
//!
//! ```rust,ignore
//! use nexo_retailer_protocol::transport::embassy::EmbassyTransport;
//!
//! # async fn example() -> Result<(), NexoError> {
//! let mut rx_buffer = [0u8; 4096];
//! let mut tx_buffer = [0u8; 4096];
//! let socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
//!
//! let mut transport = EmbassyTransport::new(socket, &mut rx_buffer, &mut tx_buffer);
//! transport.connect("192.168.1.100:8080").await?;
//!
//! // Read/write using FramedTransport wrapper
//! // ...
//! # Ok(())
//! # }
//! ```

#![cfg(feature = "embassy-net")]

use core::time::Duration as CoreDuration;

use crate::error::NexoError;
use crate::transport::Transport;

use embassy_futures::select::{select, Either};
use embassy_net::TcpSocket;
use embassy_time::{Duration, Timer};

/// Timeout configuration for Embassy transport operations
///
/// Provides timeout configuration using Embassy's `Duration` type and
/// async wrapper methods using `embassy_futures::select`.
///
/// # Example
///
/// ```rust,ignore
/// use nexo_retailer_protocol::transport::embassy::EmbassyTimeoutConfig;
/// use embassy_time::Duration;
///
/// # async fn example() -> Result<(), NexoError> {
/// let config = EmbassyTimeoutConfig::new()
///     .with_connect(Duration::from_secs(5))
///     .with_read(Duration::from_secs(30))
///     .with_write(Duration::from_secs(10));
///
/// // Use timeout wrapper
/// config.with_connect_timeout(async {
///     // Connection logic here
///     Ok(())
/// }).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct EmbassyTimeoutConfig {
    /// Timeout for connection operations
    pub connect_timeout: Duration,

    /// Timeout for read operations
    pub read_timeout: Duration,

    /// Timeout for write operations
    pub write_timeout: Duration,
}

impl Default for EmbassyTimeoutConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            read_timeout: Duration::from_secs(30),
            write_timeout: Duration::from_secs(10),
        }
    }
}

impl EmbassyTimeoutConfig {
    /// Create a new timeout configuration with default values
    ///
    /// Default timeouts:
    /// - Connect: 10 seconds
    /// - Read: 30 seconds
    /// - Write: 10 seconds
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let config = EmbassyTimeoutConfig::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the connect timeout
    ///
    /// # Arguments
    ///
    /// * `d` - Connection timeout duration
    ///
    /// # Returns
    ///
    /// Self for builder pattern chaining
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use embassy_time::Duration;
    ///
    /// let config = EmbassyTimeoutConfig::new()
    ///     .with_connect(Duration::from_secs(5));
    /// ```
    pub fn with_connect(mut self, d: Duration) -> Self {
        self.connect_timeout = d;
        self
    }

    /// Set the read timeout
    ///
    /// # Arguments
    ///
    /// * `d` - Read timeout duration
    ///
    /// # Returns
    ///
    /// Self for builder pattern chaining
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use embassy_time::Duration;
    ///
    /// let config = EmbassyTimeoutConfig::new()
    ///     .with_read(Duration::from_secs(60));
    /// ```
    pub fn with_read(mut self, d: Duration) -> Self {
        self.read_timeout = d;
        self
    }

    /// Set the write timeout
    ///
    /// # Arguments
    ///
    /// * `d` - Write timeout duration
    ///
    /// # Returns
    ///
    /// Self for builder pattern chaining
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use embassy_time::Duration;
    ///
    /// let config = EmbassyTimeoutConfig::new()
    ///     .with_write(Duration::from_secs(20));
    /// ```
    pub fn with_write(mut self, d: Duration) -> Self {
        self.write_timeout = d;
        self
    }

    /// Wrap a future with read timeout
    ///
    /// Uses `embassy_futures::select` to race the future against a timer.
    /// If the timer completes first, returns `NexoError::Timeout`.
    ///
    /// # Type Parameters
    ///
    /// * `F` - Future type to wrap
    /// * `T` - Result type
    ///
    /// # Arguments
    ///
    /// * `f` - Future to execute with timeout
    ///
    /// # Returns
    ///
    /// Result of the future, or `NexoError::Timeout` if timeout expires
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # async fn example() -> Result<(), NexoError> {
    /// let config = EmbassyTimeoutConfig::new();
    ///
    /// let result = config.with_read_timeout(async {
    ///     // Read operation here
    ///     Ok(bytes_read)
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn with_read_timeout<F, T>(&self, f: F) -> Result<T, NexoError>
    where
        F: core::future::Future<Output = Result<T, NexoError>>,
    {
        match select(f, Timer::after(self.read_timeout)).await {
            Either::First(result) => result,
            Either::Second(_) => Err(NexoError::Timeout),
        }
    }

    /// Wrap a future with write timeout
    ///
    /// Uses `embassy_futures::select` to race the future against a timer.
    /// If the timer completes first, returns `NexoError::Timeout`.
    ///
    /// # Type Parameters
    ///
    /// * `F` - Future type to wrap
    /// * `T` - Result type
    ///
    /// # Arguments
    ///
    /// * `f` - Future to execute with timeout
    ///
    /// # Returns
    ///
    /// Result of the future, or `NexoError::Timeout` if timeout expires
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # async fn example() -> Result<(), NexoError> {
    /// let config = EmbassyTimeoutConfig::new();
    ///
    /// let result = config.with_write_timeout(async {
    ///     // Write operation here
    ///     Ok(bytes_written)
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn with_write_timeout<F, T>(&self, f: F) -> Result<T, NexoError>
    where
        F: core::future::Future<Output = Result<T, NexoError>>,
    {
        match select(f, Timer::after(self.write_timeout)).await {
            Either::First(result) => result,
            Either::Second(_) => Err(NexoError::Timeout),
        }
    }

    /// Wrap a future with connect timeout
    ///
    /// Uses `embassy_futures::select` to race the future against a timer.
    /// If the timer completes first, returns `NexoError::Timeout`.
    ///
    /// # Type Parameters
    ///
    /// * `F` - Future type to wrap
    /// * `T` - Result type
    ///
    /// # Arguments
    ///
    /// * `f` - Future to execute with timeout
    ///
    /// # Returns
    ///
    /// Result of the future, or `NexoError::Timeout` if timeout expires
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # async fn example() -> Result<(), NexoError> {
    /// let config = EmbassyTimeoutConfig::new();
    ///
    /// let result = config.with_connect_timeout(async {
    ///     // Connect operation here
    ///     Ok(())
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn with_connect_timeout<F, T>(&self, f: F) -> Result<T, NexoError>
    where
        F: core::future::Future<Output = Result<T, NexoError>>,
    {
        match select(f, Timer::after(self.connect_timeout)).await {
            Either::First(result) => result,
            Either::Second(_) => Err(NexoError::Timeout),
        }
    }
}

/// Embassy-based transport implementation for embedded environments
///
/// This transport wraps `embassy_net::TcpSocket` and provides async read/write
/// operations with configurable timeouts using Embassy's async primitives.
///
/// # Lifetime Parameters
///
/// * `'a` - Lifetime for buffer references held by the TcpSocket
///
/// # Type Parameters
///
/// The lifetime `'a` is required because Embassy's TcpSocket holds references
/// to the rx/tx buffers provided by the caller. This design avoids allocation
/// in embedded environments.
///
/// # Example
///
/// ```rust,ignore
/// use nexo_retailer_protocol::transport::embassy::EmbassyTransport;
/// use embassy_net::TcpSocket;
/// use embassy_time::Duration;
///
/// # async fn example() -> Result<(), NexoError> {
/// let mut rx_buffer = [0u8; 4096];
/// let mut tx_buffer = [0u8; 4096];
/// let socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
///
/// let mut transport = EmbassyTransport::new(socket, &mut rx_buffer, &mut tx_buffer)
///     .with_timeouts(Duration::from_secs(30), Duration::from_secs(10));
///
/// transport.connect("192.168.1.100:8080").await?;
/// # Ok(())
/// # }
/// ```
pub struct EmbassyTransport<'a> {
    /// The underlying TCP socket from Embassy
    socket: TcpSocket<'a>,

    /// Read timeout (None = no timeout)
    read_timeout: Duration,

    /// Write timeout (None = no timeout)
    write_timeout: Duration,

    /// Connect timeout (None = no timeout)
    connect_timeout: Duration,
}

impl<'a> EmbassyTransport<'a> {
    /// Create a new EmbassyTransport from a TcpSocket
    ///
    /// # Arguments
    ///
    /// * `socket` - The Embassy TCP socket
    /// * `rx_buffer` - Receive buffer reference (for lifetime tracking)
    /// * `tx_buffer` - Transmit buffer reference (for lifetime tracking)
    ///
    /// # Returns
    ///
    /// A new EmbassyTransport instance with default timeouts:
    /// - Connect: 10 seconds
    /// - Read: 30 seconds
    /// - Write: 10 seconds
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut rx_buffer = [0u8; 4096];
    /// let mut tx_buffer = [0u8; 4096];
    /// let socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    ///
    /// let transport = EmbassyTransport::new(socket, &mut rx_buffer, &mut tx_buffer);
    /// ```
    pub fn new(
        socket: TcpSocket<'a>,
        _rx_buffer: &'a mut [u8],
        _tx_buffer: &'a mut [u8],
    ) -> Self {
        Self {
            socket,
            read_timeout: Duration::from_secs(30),
            write_timeout: Duration::from_secs(10),
            connect_timeout: Duration::from_secs(10),
        }
    }

    /// Configure custom timeouts for this transport
    ///
    /// # Arguments
    ///
    /// * `read` - Timeout for read operations
    /// * `write` - Timeout for write operations
    ///
    /// # Returns
    ///
    /// Self for builder pattern chaining
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use embassy_time::Duration;
    ///
    /// let transport = EmbassyTransport::new(socket, &mut rx_buf, &mut tx_buf)
    ///     .with_timeouts(Duration::from_secs(60), Duration::from_secs(20));
    /// ```
    pub fn with_timeouts(mut self, read: Duration, write: Duration) -> Self {
        self.read_timeout = read;
        self.write_timeout = write;
        self
    }

    /// Configure connect timeout
    ///
    /// # Arguments
    ///
    /// * `timeout` - Timeout for connect operations
    ///
    /// # Returns
    ///
    /// Self for builder pattern chaining
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use embassy_time::Duration;
    ///
    /// let transport = EmbassyTransport::new(socket, &mut rx_buf, &mut tx_buf)
    ///     .with_connect_timeout(Duration::from_secs(5));
    /// ```
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Connect to a remote address with timeout
    ///
    /// This is the internal connect implementation that uses Embassy's
    /// `embassy_futures::select` to enforce the connect timeout.
    ///
    /// # Arguments
    ///
    /// * `addr` - Remote address (e.g., "192.168.1.100:8080")
    /// * `timeout` - Connection timeout
    ///
    /// # Errors
    ///
    /// Returns `NexoError` if:
    /// - Address parsing fails
    /// - Connection cannot be established
    /// - Timeout occurs
    async fn connect_internal(
        &mut self,
        addr: &str,
        timeout: Duration,
    ) -> Result<(), NexoError> {
        // Parse the address - Embassy uses different address parsing
        // For now, we'll need to parse it manually or use Embassy's address types
        // The Embassy stack typically uses (IpAddress, Port) tuples

        // Use embassy_futures::select to race connection with timer
        // Note: TcpSocket::connect takes (IpAddress, u16) in Embassy
        // We'll need to parse the address string ourselves
        let parts: Vec<&str> = addr.split(':').collect();
        if parts.len() != 2 {
            return Err(NexoError::Connection {
                details: "invalid address format, expected 'IP:PORT'",
            });
        }

        let ip: embassy_net::IpAddress = parts[0].parse().map_err(|_| {
            NexoError::Connection {
                details: "invalid IP address format",
            }
        })?;

        let port: u16 = parts[1].parse().map_err(|_| {
            NexoError::Connection {
                details: "invalid port number",
            }
        })?;

        match select(
            self.socket.connect((ip, port)),
            Timer::after(timeout),
        )
        .await
        {
            Either::First(result) => result.map_err(|_| NexoError::Connection {
                details: "connection failed",
            }),
            Either::Second(_) => Err(NexoError::Timeout),
        }
    }
}

impl<'a> Transport for EmbassyTransport<'a> {
    type Error = NexoError;

    /// Read bytes from the TCP socket with timeout
    ///
    /// Uses `embassy_futures::select` to enforce read timeout. If the timeout
    /// expires before data is available, returns `NexoError::Timeout`.
    ///
    /// # Arguments
    ///
    /// * `buf` - Buffer to read bytes into
    ///
    /// # Returns
    ///
    /// Number of bytes read (0 indicates EOF)
    ///
    /// # Errors
    ///
    /// Returns `NexoError` if:
    /// - Not connected
    /// - Read timeout occurs
    /// - Socket error occurs
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        // Check if connected
        if !self.socket.is_open() {
            return Err(NexoError::Connection {
                details: "not connected",
            });
        }

        // Use embassy_futures::select to race read with timer
        match select(self.socket.read(buf), Timer::after(self.read_timeout)).await {
            Either::First(result) => result.map_err(|_| NexoError::Connection {
                details: "read failed",
            }),
            Either::Second(_) => Err(NexoError::Timeout),
        }
    }

    /// Write bytes to the TCP socket with timeout
    ///
    /// Uses `embassy_futures::select` to enforce write timeout. If the timeout
    /// expires before all bytes are written, returns `NexoError::Timeout`.
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
    /// Returns `NexoError` if:
    /// - Not connected
    /// - Write timeout occurs
    /// - Socket error occurs
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        // Check if connected
        if !self.socket.is_open() {
            return Err(NexoError::Connection {
                details: "not connected",
            });
        }

        // Use embassy_futures::select to race write with timer
        match select(self.socket.write(buf), Timer::after(self.write_timeout)).await {
            Either::First(result) => result.map_err(|_| NexoError::Connection {
                details: "write failed",
            }),
            Either::Second(_) => Err(NexoError::Timeout),
        }
    }

    /// Connect to a remote address
    ///
    /// Uses the configured connect timeout (default 10 seconds).
    ///
    /// # Arguments
    ///
    /// * `addr` - Remote address (e.g., "192.168.1.100:8080")
    ///
    /// # Errors
    ///
    /// Returns `NexoError` if:
    /// - Address is invalid
    /// - Connection fails
    /// - Timeout occurs
    async fn connect(&mut self, addr: &str) -> Result<(), Self::Error> {
        self.connect_internal(addr, self.connect_timeout).await
    }

    /// Check if the transport is currently connected
    ///
    /// This is a synchronous method that returns the current connection state
    /// without blocking.
    ///
    /// # Returns
    ///
    /// `true` if connected, `false` otherwise
    fn is_connected(&self) -> bool {
        self.socket.is_open()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full Embassy transport tests require:
    // - Embassy executor setup
    // - Network stack mock or real hardware
    // These tests are added in Plan 03-05 (Embassy Transport Export and Tests)
}
