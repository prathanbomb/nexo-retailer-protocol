//! Standard library (std) specific implementations
//!
//! This module contains code that requires the standard library.
//! It is only compiled when the `std` feature is enabled.
//!
//! # Current Status
//!
//! This module is a placeholder for Phase 3 (Transport Layer) where
//! std-specific implementations will be added:
//!
//! - Tokio async runtime for TCP connections
//! - Standard library IO operations
//! - Thread-local storage for connection pooling

#[cfg(feature = "std")]
pub mod std_transport {
    //! std-specific transport implementation
    //!
    //! This will be implemented in Phase 3 (Transport Layer) to provide
    //! TCP/TLS support using Tokio for server environments.
    //!
    //! # Planned Implementation
    //!
    //! ```rust,ignore
    //! pub struct StdTransport {
    //!     // Tokio TCP stream, TLS config, etc.
    //! }
    //!
    //! impl StdTransport {
    //!     pub async fn connect(&self) -> Result<(), NexoError> {
    //!         // Tokio-based connection logic
    //!     }
    //! }
    //! ```
}
