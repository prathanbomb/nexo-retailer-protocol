//! Bare-metal (no_std) specific implementations
//!
//! This module contains code that works without the standard library.
//! It is compiled when the `std` feature is NOT enabled.
//!
//! # Current Status
//!
//! This module is a placeholder for Phase 3 (Transport Layer) where
//! no_std-specific implementations will be added:
//!
//! - Embassy async runtime for embedded devices
//! - heapless fixed-capacity collections
//! - defmt logging integration

//! no_std-specific transport implementation
//!
//! This will be implemented in Phase 3 (Transport Layer) to provide
//! TCP/TLS support using Embassy for embedded environments.
//!
//! # Planned Implementation
//!
//! ```rust,ignore
//! use embassy_net::TcpSocket;
//!
//! pub struct EmbassyTransport {
//!     // Embassy TCP socket, etc.
//! }
//!
//! impl EmbassyTransport {
//!     pub async fn connect(&self) -> Result<(), NexoError> {
//!         // Embassy-based connection logic
//!     }
//! }
//! ```
pub mod no_std_transport {}
