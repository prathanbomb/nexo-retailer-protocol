#![cfg_attr(not(feature = "std"), no_std)]

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
//! ## Message Validation
//!
//! The library provides comprehensive validation for XSD constraints:
//!
//! ```rust,ignore
//! use nexo_retailer_protocol::{
//!     Validate, validate_currency_code, validate_monetary_amount,
//!     validate_max256_text, validate_required
//! };
//!
//! // Validate individual fields
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! validate_currency_code("USD")?;
//! validate_max256_text("Some text")?;
//!
//! // Validate entire messages
//! let header = Header4::default();
//! header.validate()?;
//! # Ok(())
//! ```
//!
//! Validation features:
//! - **Currency validation**: ISO 4217 format `[A-Z]{3,3}` (e.g., "USD", "EUR")
//! - **String length validation**: Max256Text, Max20000Text, Max70Text
//! - **Field presence validation**: Required vs optional fields (XSD minOccurs)
//! - **Monetary amount validation**: Nanos range, sign consistency
//! - **no_std compatible**: Basic validation works without alloc feature
//! - **alloc feature**: Enables collection validation (repeated fields)
//!
//! See the [`validate`] module for detailed validation functions.
//!
//! ## Codec (Encoding/Decoding)
//!
//! The library provides a codec layer for encoding and decoding all CASP message types:
//!
//! ```rust
//! use nexo_retailer_protocol::{encode_message, decode_message, Casp001Document};
//!
//! // Encode a message to bytes
//! let message = Casp001Document::default();
//! let bytes = encode_message(&message).unwrap();
//!
//! // Decode bytes back to a message
//! let decoded: Casp001Document = decode_message(&bytes).unwrap();
//! ```
//!
//! The codec layer:
//! - Enforces 4MB message size limits to prevent unbounded allocation attacks
//! - Works with all 17 CASP message types via generic implementation
//! - Is no_std compatible for bare-metal environments
//! - Provides trait abstraction for testing with mock codecs
//!
//! ## Transport Layer
//!
//! The library provides a runtime-agnostic transport trait for sending messages over TCP:
//!
//! ```rust,ignore
//! use nexo_retailer_protocol::{Transport, FramedTransport};
//!
//! // Any type implementing Transport can be wrapped with FramedTransport
//! let transport = MyTransport::new();
//! let mut framed = FramedTransport::new(transport);
//!
//! // Send/receive messages with length-prefixed framing
//! framed.send_message(&message).await?;
//! let received = framed.recv_message::<MessageType>().await?;
//! ```
//!
//! The transport layer:
//! - Provides runtime-agnostic trait for both Tokio (std) and Embassy (no_std)
//! - Implements length-prefixed TCP framing (4-byte big-endian)
//! - Enforces 4MB message size limits
//! - Handles partial reads/writes automatically
//! - Works with all prost::Message types
//!
//! ## Client API
//!
//! The library provides a high-level client API for POS initiators:
//!
//! ```rust,no_run
//! use nexo_retailer_protocol::NexoClient;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a new client
//! let mut client = NexoClient::new();
//!
//! // Connect to payment terminal
//! client.connect("192.168.1.100:8080").await?;
//!
//! // Send a payment request
//! let request = Casp001Document::default();
//! let response: Casp002Document = client.send_and_receive(&request).await?;
//!
//! // Disconnect when done
//! client.disconnect().await?;
//! # Ok(())
//! # }
//! ```
//!
//! The client API:
//! - Provides connection management (connect, disconnect, is_connected)
//! - Supports both Tokio (std) and Embassy (no_std) runtimes
//! - Handles request/response correlation automatically
//! - Uses length-prefixed framing for message transmission
//!
//! ### Automatic Reconnection
//!
//! The client supports automatic reconnection with exponential backoff:
//!
//! ```rust,no_run
//! use nexo_retailer_protocol::{NexoClient, ReconnectConfig};
//! use std::time::Duration;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create client with reconnection config
//! let config = ReconnectConfig::new()
//!     .with_base_delay(Duration::from_millis(100))
//!     .with_max_delay(Duration::from_secs(60))
//!     .with_max_attempts(5);
//!
//! let mut client = NexoClient::new()
//!     .with_reconnect_config(config);
//!
//! client.connect("192.168.1.100:8080").await?;
//!
//! // After connection lost, reconnect with backoff
//! client.reconnect().await?;
//! # Ok(())
//! # }
//! ```
//!
//! Reconnection features:
//! - Exponential backoff: delay = base * 2^attempt (100ms → 200ms → 400ms...)
//! - Max delay cap prevents unbounded wait times
//! - Jitter (std only) prevents thundering herd when multiple clients reconnect
//! - Configurable max attempts allows infinite retry if set to u32::MAX
//!
//! ### Builder Pattern
//!
//! The library provides builder structs for fluent, type-safe message construction:
//!
//! ```rust,no_run
//! use nexo_retailer_protocol::{
//!     Header4Builder, PaymentRequestBuilder, SaleToPoiServiceRequestV06Builder
//! };
//!
//! // Build a header with fluent API
//! let header = Header4Builder::new()
//!     .message_function("DREQ".to_string())
//!     .protocol_version("6.0".to_string())
//!     .transaction_id("TX-12345".to_string())
//!     .build()
//!     .unwrap();
//!
//! // Build a payment request
//! let payment = PaymentRequestBuilder::new()
//!     .transaction_id("TX-12345".to_string())
//!     .build()
//!     .unwrap();
//!
//! // Build a complete service request
//! let request = SaleToPoiServiceRequestV06Builder::new()
//!     .header(header)
//!     .build()
//!     .unwrap();
//! ```
//!
//! Available builders:
//! - `Header4Builder` - Protocol headers
//! - `PaymentRequestBuilder` - Payment request messages
//! - `SaleToPoiServiceRequestV06Builder` - Service requests
//! - `SaleToPoiServiceResponseV06Builder` - Service responses
//!
//! Builders validate required fields at build time and return `Result<T, NexoError>`
//! for proper error handling.
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

// Required for alloc feature
#[cfg(feature = "alloc")]
extern crate alloc;

// Include generated protobuf code
// prost-build generates all proto messages in a single file
// All types are available at the crate root
include!("protos/nexo.casp.v1.rs");

// Public module exports
pub mod client;
pub mod codec;
pub mod error;
pub mod features;
pub mod transport;
pub mod validate;

// Re-export commonly used error types at crate root
pub use error::{DecodeError, EncodeError, NexoError, ValidationError};
pub use validate::{
    // Validate trait and core validators
    Validate,
    validate_required,
    validate_positive_i64,
    validate_non_negative_i32,
    validate_enum_value,
    // Currency validation
    validate_currency_code,
    validate_monetary_amount,
    // String validation
    validate_max_text,
    validate_max256_text,
    validate_max20000_text,
    validate_max70_text,
};

// Re-export validate_repeated_field only when alloc feature is enabled
#[cfg(feature = "alloc")]
pub use validate::validate_repeated_field;

// Re-export codec types at crate root for convenience
// Codec uses Vec<u8> which requires alloc feature
#[cfg(feature = "alloc")]
pub use codec::{Codec, ProstCodec, encode as encode_message, decode as decode_message};

// Re-export transport types at crate root for convenience
pub use transport::{Transport, FramedTransport};

// Re-export Tokio transport when std feature is enabled
#[cfg(feature = "std")]
pub use transport::TokioTransport;

// Re-export TimeoutConfig when std feature is enabled
#[cfg(feature = "std")]
pub use transport::TimeoutConfig;

// Re-export Embassy transport when embassy-net feature is enabled
#[cfg(feature = "embassy-net")]
pub use transport::{EmbassyTransport, EmbassyTimeoutConfig};

// Re-export NexoClient at crate root for ergonomic API
// The client is generic over Transport, so it works with both std and embassy
#[cfg(feature = "std")]
pub use client::NexoClient;

#[cfg(all(feature = "embassy-net", not(feature = "std")))]
pub use client::NexoClient;

// Re-export reconnection types at crate root
pub use client::reconnect::{ReconnectConfig, Backoff};

// Re-export builders at crate root for ergonomic message construction
pub use client::builder::{
    MessageBuilder,
    Header4Builder,
    PaymentRequestBuilder,
    SaleToPoiServiceRequestV06Builder,
    SaleToPoiServiceResponseV06Builder,
};

#[cfg(test)]
mod tests {
    use super::*;

    // Import ToString for no_std tests with alloc
    #[cfg(feature = "alloc")]
    use prost::alloc::string::ToString;

    #[test]
    #[cfg(feature = "std")]
    fn test_client_exported_at_crate_root() {
        // Verify NexoClient is accessible from crate root
        let client = NexoClient::new();
        assert!(!client.is_connected());
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_builders_exported_at_crate_root() {
        // Verify builders are accessible from crate root
        let _header_builder = Header4Builder::new();
        let _payment_builder = PaymentRequestBuilder::new();

        // Verify service request/response builders are exported
        let _request_builder = SaleToPoiServiceRequestV06Builder::new();
        let _response_builder = SaleToPoiServiceResponseV06Builder::new();
    }

    #[test]
    fn test_library_builds() {
        // Basic smoke test - library compiles
        assert!(true, "library builds successfully");
    }

    // Integration tests demonstrating end-to-end validation
    #[test]
    #[cfg(feature = "alloc")]
    fn test_validation_integration_valid_message() {
        // Test a valid monetary amount
        let amount = ActiveCurrencyAndAmount {
            ccy: "USD".to_string(),
            units: 100,
            nanos: 500000000, // 100.50
        };
        assert!(amount.validate().is_ok());
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_validation_integration_invalid_currency() {
        // Test invalid currency code (lowercase)
        let amount = ActiveCurrencyAndAmount {
            ccy: "usd".to_string(), // Invalid: should be uppercase
            units: 100,
            nanos: 0,
        };
        assert!(amount.validate().is_err());
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_validation_integration_invalid_nanos() {
        // Test invalid nanos (sign mismatch)
        let amount = ActiveCurrencyAndAmount {
            ccy: "USD".to_string(),
            units: 100,
            nanos: -500000000, // Sign mismatch: positive units, negative nanos
        };
        assert!(amount.validate().is_err());
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_validation_integration_header_validation() {
        // Test Header4 validation
        let header = Header4 {
            msg_fctn: Some("DREQ".to_string()),
            proto_vrsn: Some("6.0".to_string()),
            tx_id: Some("TX-12345".to_string()),
            ..Default::default()
        };
        assert!(header.validate().is_ok());
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_validation_integration_header_long_field() {
        // Test Header4 with field exceeding Max70Text limit
        let header = Header4 {
            msg_fctn: Some("A".repeat(100)), // Exceeds 70 bytes
            ..Default::default()
        };
        assert!(header.validate().is_err());
    }

    #[test]
    fn test_validation_integration_standalone_validators() {
        // Test standalone validator functions
        assert!(validate_currency_code("USD").is_ok());
        assert!(validate_currency_code("usd").is_err());

        assert!(validate_max256_text("Hello").is_ok());
        assert!(validate_max256_text(&"A".repeat(257)).is_err());

        let field = Some("value");
        assert!(validate_required(&field, "TestField").is_ok());

        let none_field: Option<&str> = None;
        assert!(validate_required(&none_field, "TestField").is_err());
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_validation_integration_alloc_feature() {
        // This test validates that alloc feature enables collection validation
        // It only compiles when alloc feature is enabled
        let items = vec![1, 2, 3];
        assert!(validate_repeated_field(&items, 10, "Items").is_ok());
        assert!(validate_repeated_field(&items, 2, "Items").is_err());
    }
}
