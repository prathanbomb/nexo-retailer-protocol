//! Server API for Nexo Retailer Protocol
//!
//! This module provides a high-level server API for payment terminals (POI)
//! that wraps the transport layer with concurrent connection handling and
//! per-connection state tracking. The server is designed to support both
//! Tokio (std) and Embassy (no_std) runtimes without code duplication.
//!
//! # Architecture
//!
//! The server module is organized into runtime-specific implementations:
//!
//! - **std** (default): Tokio-based implementation using `tokio::spawn`
//!   for concurrent connection handling and `tokio::sync::Mutex` for
//!   thread-safe state management
//! - **embassy-net** (future): Embassy-based implementation using
//!   `embassy_futures::spawn` for concurrent connection handling
//!
//! Both implementations share the same API surface, enabling code that works
//! in both standard and bare-metal environments.
//!
//! # Connection Management
//!
//! The server maintains a thread-safe HashMap of active connections, tracking
//! per-connection state including:
//! - Client socket address
//! - Connection timestamp
//! - Message count
//! - (Future) Deduplication state
//! - (Future) Heartbeat tracking
//!
//! Each connection runs in an independent async task, allowing the server to
//! handle thousands of concurrent connections efficiently.
//!
//! # Usage
//!
//! ```rust,no_run
//! use nexo_retailer_protocol::NexoServer;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a new server
//! let server = NexoServer::bind("127.0.0.1:8080")?;
//!
//! // Run the server (accepts connections indefinitely)
//! server.run().await?;
//! # Ok(())
//! # }
//! ```

// Per-connection state tracking
pub mod connection;

// Message deduplication cache for replay attack prevention
pub mod dedup;

// Heartbeat/keepalive protocol for dead connection detection
pub mod heartbeat;

// Request handler trait for application callbacks
pub mod handler;

// Message dispatcher for routing incoming messages
pub mod dispatcher;

// Runtime-specific implementations
#[cfg(feature = "std")]
pub mod std_impl;

// Re-export the server type at module level
// Note: Only one of these will be active at a time due to feature gating
#[cfg(feature = "std")]
pub use std_impl::NexoServer;

// Re-export ConnectionState for convenience
pub use connection::ConnectionState;

// Re-export DeduplicationCache for convenience
pub use dedup::DeduplicationCache;

// Re-export HeartbeatConfig and HeartbeatMonitor for convenience
pub use heartbeat::{HeartbeatConfig, HeartbeatMonitor};

// Re-export RequestHandler trait for convenience
pub use handler::RequestHandler;

// Re-export Dispatcher for convenience
pub use dispatcher::Dispatcher;
