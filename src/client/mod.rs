//! Client API for Nexo Retailer Protocol
//!
//! This module provides a high-level client API for POS initiators that wraps
//! the transport layer with connection management and request/response correlation.
//! The client is generic over the `Transport` trait to support both Tokio (std)
//! and Embassy (no_std) runtimes without code duplication.
//!
//! # Architecture
//!
//! The client module is organized into runtime-specific implementations:
//!
//! - **std** (default): Tokio-based implementation using `tokio::sync::oneshot`
//! - **embassy-net**: Embassy-based implementation using `embassy_futures::channel::oneshot`
//!
//! Both implementations share the same API surface, enabling code that works
//! in both standard and bare-metal environments.
//!
//! # Usage
//!
//! ```rust,no_run
//! use nexo_retailer_protocol::NexoClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a new client (Tokio transport)
//! let mut client = NexoClient::new();
//!
//! // Connect to the payment terminal
//! client.connect("192.168.1.100:8080").await?;
//!
//! // Send a payment request and receive response
//! let request = PaymentRequest::default();
//! let response: PaymentResponse = client.send_and_receive(&request).await?;
//!
//! // Disconnect when done
//! client.disconnect().await?;
//! # Ok(())
//! # }
//! ```

// Runtime-specific implementations
#[cfg(feature = "std")]
pub mod std;

#[cfg(feature = "embassy-net")]
pub mod embassy;

// Re-export the client type at module level
// Note: Only one of these will be active at a time due to feature gating
#[cfg(feature = "std")]
pub use std::NexoClient;

#[cfg(all(feature = "embassy-net", not(feature = "std")))]
pub use embassy::NexoClient;
