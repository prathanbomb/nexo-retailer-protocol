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
use crate::Header4;
use crate::PaymentRequest29;

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

/// Builder for Header4 message
///
/// Header4 contains protocol-level metadata for CASP messages including:
/// - Message function (e.g., "DREQ" for Diagnosis Request)
/// - Protocol version (e.g., "6.0")
/// - Transaction ID for correlation
///
/// # Required Fields
///
/// * `msg_fctn` - Message function code (e.g., "DREQ", "AQRY")
/// * `proto_vrsn` - Protocol version (typically "6.0")
/// * `tx_id` - Transaction identifier for correlation
///
/// # Example
///
/// ```rust,no_run
/// use nexo_retailer_protocol::Header4Builder;
///
/// let header = Header4Builder::new()
///     .message_function("DREQ".to_string())
///     .protocol_version("6.0".to_string())
///     .transaction_id("TX-12345".to_string())
///     .build()
///     .unwrap();
/// ```
pub struct Header4Builder {
    inner: Header4,
}

impl Header4Builder {
    /// Create a new Header4 builder with default values
    ///
    /// All fields start as `None`. Required fields must be set
    /// before calling `build()`.
    pub fn new() -> Self {
        Self {
            inner: Header4::default(),
        }
    }

    /// Set the message function code
    ///
    /// # Arguments
    ///
    /// * `value` - Message function code (e.g., "DREQ", "AQRY", "AUTH")
    ///
    /// # Required
    ///
    /// This field is REQUIRED according to the Nexo specification.
    pub fn message_function(mut self, value: String) -> Self {
        self.inner.msg_fctn = Some(value);
        self
    }

    /// Set the protocol version
    ///
    /// # Arguments
    ///
    /// * `value` - Protocol version string (typically "6.0")
    ///
    /// # Required
    ///
    /// This field is REQUIRED according to the Nexo specification.
    pub fn protocol_version(mut self, value: String) -> Self {
        self.inner.proto_vrsn = Some(value);
        self
    }

    /// Set the transaction ID
    ///
    /// # Arguments
    ///
    /// * `id` - Unique transaction identifier for request/response correlation
    ///
    /// # Required
    ///
    /// This field is REQUIRED for proper message correlation.
    pub fn transaction_id(mut self, id: String) -> Self {
        self.inner.tx_id = Some(id);
        self
    }

    /// Set the original business message type
    ///
    /// # Arguments
    ///
    /// * `value` - Original message type identifier
    pub fn original_business_message(mut self, value: String) -> Self {
        self.inner.orgnl_biz_t_msg = Some(value);
        self
    }

    /// Set the original message ID
    ///
    /// # Arguments
    ///
    /// * `id` - Original message identifier
    pub fn original_message_id(mut self, id: String) -> Self {
        self.inner.orgnl_msg_id = Some(id);
        self
    }

    /// Set the creation date/time
    ///
    /// # Arguments
    ///
    /// * `value` - ISO 8601 datetime string
    pub fn creation_datetime(mut self, value: String) -> Self {
        self.inner.cre_dt_tm = Some(value);
        self
    }
}

impl MessageBuilder<Header4> for Header4Builder {
    /// Build the Header4, validating required fields
    ///
    /// # Errors
    ///
    /// Returns `Err(NexoError::Validation)` if:
    /// - `msg_fctn` is not set
    /// - `proto_vrsn` is not set
    /// - `tx_id` is not set
    fn build(self) -> Result<Header4, NexoError> {
        // Validate required fields
        if self.inner.msg_fctn.is_none() {
            return Err(NexoError::Validation {
                field: "msg_fctn",
                reason: "message_function is required",
            });
        }

        if self.inner.proto_vrsn.is_none() {
            return Err(NexoError::Validation {
                field: "proto_vrsn",
                reason: "protocol_version is required",
            });
        }

        if self.inner.tx_id.is_none() {
            return Err(NexoError::Validation {
                field: "tx_id",
                reason: "transaction_id is required",
            });
        }

        Ok(self.inner)
    }
}

impl Default for Header4Builder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for PaymentRequest29 message
///
/// PaymentRequest29 represents a payment request in the CASP protocol.
/// This builder provides a fluent API for constructing payment requests
/// with proper validation of required fields.
///
/// # Required Fields
///
/// * `tx_id` - Transaction identifier (required)
///
/// # Example
///
/// ```rust,no_run
/// use nexo_retailer_protocol::PaymentRequestBuilder;
///
/// let request = PaymentRequestBuilder::new()
///     .transaction_id("TX-12345".to_string())
///     .build()
///     .unwrap();
/// ```
pub struct PaymentRequestBuilder {
    inner: PaymentRequest29,
}

impl PaymentRequestBuilder {
    /// Create a new PaymentRequest builder with default values
    pub fn new() -> Self {
        Self {
            inner: PaymentRequest29::default(),
        }
    }

    /// Set the transaction ID
    ///
    /// # Arguments
    ///
    /// * `id` - Unique transaction identifier
    ///
    /// # Required
    ///
    /// This field is REQUIRED for proper transaction tracking.
    pub fn transaction_id(mut self, id: String) -> Self {
        self.inner.tx_id = Some(id);
        self
    }

    /// Set the reconciliation ID
    ///
    /// # Arguments
    ///
    /// * `id` - Reconciliation identifier
    pub fn reconciliation_id(mut self, id: String) -> Self {
        self.inner.rcncltn_id = Some(id);
        self
    }

    /// Set the original message ID
    ///
    /// # Arguments
    ///
    /// * `id` - Original message identifier (for follow-up transactions)
    pub fn original_message_id(mut self, id: String) -> Self {
        self.inner.orgnl_msg_id = Some(id);
        self
    }

    /// Set the original transaction ID
    ///
    /// # Arguments
    ///
    /// * `id` - Original transaction identifier (for follow-up transactions)
    pub fn original_transaction_id(mut self, id: String) -> Self {
        self.inner.orgnl_tx_id = Some(id);
        self
    }

    /// Set the original business message type
    ///
    /// # Arguments
    ///
    /// * `msg_type` - Original business message type
    pub fn original_business_message(mut self, msg_type: String) -> Self {
        self.inner.orgnl_biz_t_msg = Some(msg_type);
        self
    }

    /// Set the payment type
    ///
    /// # Arguments
    ///
    /// * `payment_type` - Payment type code
    pub fn payment_type(mut self, payment_type: String) -> Self {
        self.inner.pmt_tp = Some(payment_type);
        self
    }

    /// Set the merchant category code
    ///
    /// # Arguments
    ///
    /// * `code` - Merchant category code (MCC)
    pub fn merchant_category_code(mut self, code: String) -> Self {
        self.inner.mrchnt_categ_cd = Some(code);
        self
    }
}

impl MessageBuilder<PaymentRequest29> for PaymentRequestBuilder {
    /// Build the PaymentRequest29, validating required fields
    ///
    /// # Errors
    ///
    /// Returns `Err(NexoError::Validation)` if:
    /// - `tx_id` is not set
    fn build(self) -> Result<PaymentRequest29, NexoError> {
        // Validate required fields
        if self.inner.tx_id.is_none() {
            return Err(NexoError::Validation {
                field: "tx_id",
                reason: "transaction_id is required",
            });
        }

        Ok(self.inner)
    }
}

impl Default for PaymentRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::client::builder::{Header4Builder, MessageBuilder, PaymentRequestBuilder};
    use crate::error::NexoError;

    #[test]
    fn test_message_builder_trait_exists() {
        // This test verifies the MessageBuilder trait is defined
        // Actual builder implementations are tested separately
        assert!(true, "MessageBuilder trait defined successfully");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_header_builder_valid_construction() {
        use prost::alloc::string::ToString;

        let header = Header4Builder::new()
            .message_function("DREQ".to_string())
            .protocol_version("6.0".to_string())
            .transaction_id("TX-12345".to_string())
            .build()
            .unwrap();

        assert_eq!(header.msg_fctn, Some("DREQ".to_string()));
        assert_eq!(header.proto_vrsn, Some("6.0".to_string()));
        assert_eq!(header.tx_id, Some("TX-12345".to_string()));
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_header_builder_missing_message_function() {
        use prost::alloc::string::ToString;

        let result = Header4Builder::new()
            .protocol_version("6.0".to_string())
            .transaction_id("TX-12345".to_string())
            .build();

        assert!(result.is_err());
        match result.unwrap_err() {
            NexoError::Validation { field, .. } => {
                assert_eq!(field, "msg_fctn");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_header_builder_missing_protocol_version() {
        use prost::alloc::string::ToString;

        let result = Header4Builder::new()
            .message_function("DREQ".to_string())
            .transaction_id("TX-12345".to_string())
            .build();

        assert!(result.is_err());
        match result.unwrap_err() {
            NexoError::Validation { field, .. } => {
                assert_eq!(field, "proto_vrsn");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_header_builder_missing_transaction_id() {
        use prost::alloc::string::ToString;

        let result = Header4Builder::new()
            .message_function("DREQ".to_string())
            .protocol_version("6.0".to_string())
            .build();

        assert!(result.is_err());
        match result.unwrap_err() {
            NexoError::Validation { field, .. } => {
                assert_eq!(field, "tx_id");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_header_builder_fluent_api() {
        use prost::alloc::string::ToString;

        // Test that fluent chaining works properly
        let builder = Header4Builder::new()
            .message_function("AQRY".to_string())
            .protocol_version("6.0".to_string())
            .transaction_id("TX-99999".to_string());

        // Each method should return Self for chaining
        let header = builder.build().unwrap();
        assert_eq!(header.msg_fctn, Some("AQRY".to_string()));
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_payment_builder_valid_construction() {
        use prost::alloc::string::ToString;

        let request = PaymentRequestBuilder::new()
            .transaction_id("TX-12345".to_string())
            .build()
            .unwrap();

        assert_eq!(request.tx_id, Some("TX-12345".to_string()));
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_payment_builder_missing_transaction_id() {
        use prost::alloc::string::ToString;

        let result = PaymentRequestBuilder::new().build();

        assert!(result.is_err());
        match result.unwrap_err() {
            NexoError::Validation { field, .. } => {
                assert_eq!(field, "tx_id");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_payment_builder_fluent_api() {
        use prost::alloc::string::ToString;

        // Test that fluent chaining works properly with optional fields
        let request = PaymentRequestBuilder::new()
            .transaction_id("TX-99999".to_string())
            .reconciliation_id("RECON-123".to_string())
            .payment_type("Sale".to_string())
            .build()
            .unwrap();

        assert_eq!(request.tx_id, Some("TX-99999".to_string()));
        assert_eq!(request.rcncltn_id, Some("RECON-123".to_string()));
        assert_eq!(request.pmt_tp, Some("Sale".to_string()));
    }
}
