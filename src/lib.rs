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
//! ## Codec (Encoding/Decoding)
//!
//! The library provides a codec layer for encoding and decoding all CASP message types:
//!
//! ```rust,no_run
//! use nexo_retailer_protocol::{encode_message, decode_message, Casp001Document};
//!
//! // Encode a message to bytes
//! let message = Casp001Document { /* fields */ };
//! let bytes = encode_message(&message)?;
//!
//! // Decode bytes back to a message
//! let decoded: Casp001Document = decode_message(&bytes)?;
//! # Ok::<(), nexo_retailer_protocol::NexoError>(())
//! ```
//!
//! The codec layer:
//! - Enforces 4MB message size limits to prevent unbounded allocation attacks
//! - Works with all 17 CASP message types via generic implementation
//! - Is no_std compatible for bare-metal environments
//! - Provides trait abstraction for testing with mock codecs
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
pub mod codec;
pub mod error;
pub mod features;
pub mod validate;

// Re-export commonly used error types at crate root
pub use error::{DecodeError, EncodeError, NexoError, ValidationError};
pub use validate::{
    validate_currency_code, validate_monetary_amount,
    validate_max_text, validate_max256_text, validate_max20000_text, validate_max70_text
};

// Re-export codec types at crate root for convenience
pub use codec::{Codec, ProstCodec, encode as encode_message, decode as decode_message};

#[cfg(test)]
mod tests {
    #[test]
    fn test_library_builds() {
        // Basic smoke test - library compiles
        assert!(true, "library builds successfully");
    }
}
