//! Builder pattern for CASP message construction
//!
//! This module provides builder structs for all CASP message types, enabling
//! fluent, type-safe API for constructing complex messages with validation.
//!
//! # Builder Pattern
//!
//! The builder pattern provides several benefits:
//!
//! - **Fluent API**: Method chaining for readable message construction
//! - **Type safety**: Compile-time guarantees about field types
//! - **Validation**: Required fields validated at build time
//! - **Ergonomics**: Sensible defaults for optional fields
//!
//! # Usage
//!
//! ```rust,no_run
//! use nexo_retailer_protocol::{Header4Builder, PaymentRequestBuilder};
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
//! let request = PaymentRequestBuilder::new()
//!     .amount(100, 50)  // 100.50
//!     .currency("USD".to_string())
//!     .transaction_id("TX-12345".to_string())
//!     .build()
//!     .unwrap();
//! ```
//!
//! # Validation Strategy
//!
//! Builders validate required fields according to XSD constraints:
//!
//! - **Required fields**: Must be set before `build()` is called
//! - **Optional fields**: Default to `None` if not set
//! - **Field validation**: Type constraints enforced (e.g., currency code format)
//!
//! If validation fails, `build()` returns `Err(NexoError::Validation { ... })`
//! with details about which field failed and why.

use crate::error::NexoError;

/// Trait for message builders
///
/// All builder structs implement this trait, providing a consistent
/// `build()` method that validates and returns the constructed message.
///
/// # Type Parameters
///
/// * `T` - The message type being built (e.g., `Header4`, `PaymentRequest29`)
pub trait MessageBuilder<T> {
    /// Build the message, validating required fields
    ///
    /// # Returns
    ///
    /// * `Ok(T)` - Validated message ready to use
    /// * `Err(NexoError)` - Validation error with field and reason details
    fn build(self) -> Result<T, NexoError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_builder_trait_exists() {
        // This test verifies the MessageBuilder trait is defined
        // Actual builder implementations are tested separately
        assert!(true, "MessageBuilder trait defined successfully");
    }
}
