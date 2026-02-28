//! # Nexo Retailer Protocol (ISO 20022 CASP)
//!
//! Rust implementation of the Nexo Retailer Protocol with support for both
//! standard (std) and bare-metal (no_std) environments.
//!
//! ## Features
//!
//! - `std` (default): Enables standard library support for server environments
//! - `alloc`: Enables heap-based collections for advanced validation
//! - `defmt`: Enables embedded logging format support (defmt::Format derive)
//! - `--no-default-features`: Bare-metal compatible build
//!
//! ## Message Types
//!
//! The library defines all CASP message types from ISO 20022:
//!
//! All message types are available at the crate root, including:
//! - `Casp001Document`: SaleToPOIServiceRequestV06
//! - `Casp002Document`: SaleToPOIServiceResponseV06
//! - `Casp003Document`: SaleToPOIAdminRequestV06
//! - `Casp004Document`: SaleToPOIAdminResponseV06
//! - Plus all shared types (Header4, SecurityTrailer4, CardData8, etc.)
//!
//! ## Error Handling
//!
//! The library provides comprehensive error types via the [`NexoError`] enum:
//!
//! ```rust,no_run
//! use nexo_retailer_protocol::NexoError;
//!
//! fn process_message() -> Result<(), NexoError> {
//!     Err(NexoError::Validation {
//!         field: "currency_code",
//!         reason: "invalid format"
//!     })
//! }
//! ```
//!
//! For detailed field validation errors, see [`ValidationError`].
//!
//! ## Usage
//!
//! ```rust,no_run
//! use nexo_retailer_protocol::{Casp001Document, Header4};
//! use prost::Message;
//!
//! // Note: This is a simplified example. Actual usage requires
//! // constructing the full message structure with all required fields.
//! // See specific message types for complete examples.
//! ```
//!
//! ## no_std Support
//!
//! For bare-metal targets, build with:
//! ```bash
//! cargo build --no-default-features --target thumbv7em-none-eabihf
//! ```
//!
//! All error types implement [`core::error::Error`] for no_std compatibility.
//! When the `defmt` feature is enabled, [`NexoError`] derives `defmt::Format`
//! for embedded logging.
//!

// Include generated protobuf code
// prost-build generates all proto messages in a single file
// All types are available at the crate root
include!("protos/nexo.casp.v1.rs");

// Public module exports
pub mod error;
pub mod features;

// Re-export commonly used error types at crate root
pub use error::{DecodeError, EncodeError, NexoError, ValidationError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_builds() {
        // Basic smoke test - library compiles
        assert!(true, "library builds successfully");
    }
}
