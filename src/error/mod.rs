//! Error types for Nexo Retailer Protocol
//!
//! This module defines no_std-compatible error types that work in both
//! standard and bare-metal environments. All error types implement
//! `core::error::Error` (not `std::error::Error`) for compatibility.
//!
//! # Features
//!
//! - **no_std compatible**: Uses `core::error::Error` stabilized in Rust 1.81
//! - **defmt support**: When `defmt` feature is enabled, errors implement `defmt::Format`
//! - **From conversions**: Automatic conversion from prost encoding/decoding errors
//!
//! # Usage
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

use core::error::Error;
use core::fmt;

pub mod codes;

/// Main error type for Nexo Retailer Protocol operations
///
/// This error enum covers common failure scenarios across codec, validation,
/// and transport layers. All variants provide meaningful error messages through
/// the `Display` trait implementation.
///
/// # no_std Compatibility
///
/// This type implements `core::error::Error` (not `std::error::Error`), making it
/// compatible with bare-metal targets. The `defmt` feature adds `defmt::Format`
/// derive for embedded logging.
///
/// # Examples
///
/// ```rust
/// use nexo_retailer_protocol::NexoError;
///
/// // Creating errors
/// let err = NexoError::Connection {
///     details: "failed to connect to payment terminal"
/// };
///
/// // Converting from prost errors
/// let decode_err: NexoError = prost::DecodeError::new(0).into();
/// ```
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum NexoError {
    /// Connection-related errors (network, TCP, transport layer)
    ///
    /// # Example
    /// ```rust
    /// # use nexo_retailer_protocol::NexoError;
    /// let err = NexoError::Connection {
    ///     details: "connection refused by payment terminal"
    /// };
    /// ```
    Connection {
        /// Human-readable details about the connection failure
        details: &'static str,
    },

    /// Timeout errors (operation took longer than allowed)
    ///
    /// Used when a codec, validation, or network operation exceeds its timeout.
    Timeout,

    /// Validation errors (message structure, constraints, business rules)
    ///
    /// Wraps `ValidationError` with context about which field failed validation.
    Validation {
        /// Field name that failed validation
        field: &'static str,
        /// Human-readable reason for validation failure
        reason: &'static str,
    },

    /// Encoding errors (protobuf serialization failures)
    ///
    /// Typically occurs when message size limits are exceeded or when
    /// invalid data is present in message fields.
    Encoding {
        /// Details about what went wrong during encoding
        details: &'static str,
    },

    /// Decoding errors (protobuf deserialization failures)
    ///
    /// Occurs when incoming bytes cannot be decoded into valid message types,
    typically due to corruption, version mismatch, or invalid wire format.
    Decoding {
        /// Details about what went wrong during decoding
        details: &'static str,
    },
}

impl fmt::Display for NexoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NexoError::Connection { details } => {
                write!(f, "Connection failed: {}", details)
            }
            NexoError::Timeout => {
                write!(f, "Operation timed out")
            }
            NexoError::Validation { field, reason } => {
                write!(f, "Validation failed for '{}': {}", field, reason)
            }
            NexoError::Encoding { details } => {
                write!(f, "Encoding failed: {}", details)
            }
            NexoError::Decoding { details } => {
                write!(f, "Decoding failed: {}", details)
            }
        }
    }
}

impl Error for NexoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // No underlying error source for this simple enum
        // Future versions could add Box<dyn Error> for source chaining
        None
    }
}

/// Automatic conversion from prost DecodeError
///
/// This allows using `?` operator on functions returning `prost::DecodeError`:
///
/// ```rust,ignore
/// fn decode_message(bytes: &[u8]) -> Result<Message, NexoError> {
///     let msg = Message::decode(bytes)?;  // Converts DecodeError to NexoError
///     Ok(msg)
/// }
/// ```
impl From<prost::DecodeError> for NexoError {
    fn from(_err: prost::DecodeError) -> Self {
        NexoError::Decoding {
            details: "prost decode failed",
        }
    }
}

/// Automatic conversion from prost EncodeError
///
/// This allows using `?` operator on functions returning `prost::EncodeError`:
///
/// ```rust,ignore
/// fn encode_message(msg: &Message) -> Result<Vec<u8>, NexoError> {
///     let bytes = msg.encode_to_vec()?;  // Converts EncodeError to NexoError
///     Ok(bytes)
/// }
/// ```
impl From<prost::EncodeError> for NexoError {
    fn from(_err: prost::EncodeError) -> Self {
        NexoError::Encoding {
            details: "prost encode failed",
        }
    }
}

/// Validation-specific errors for message field validation
///
/// These errors provide detailed information about which validation rule
/// failed and what the expected vs actual values were.
///
/// # no_std Compatibility
///
/// Implements `core::error::Error` for no_std compatibility.
/// Supports `defmt::Format` when `defmt` feature is enabled.
///
/// # Examples
///
/// ```rust
/// use nexo_retailer_protocol::error::ValidationError;
///
/// // Missing required field
/// let err = ValidationError::MissingRequiredField {
///     field: "SaleRefNo",
/// };
///
/// // Invalid currency code
/// let err = ValidationError::InvalidCurrencyFormat {
///     code: "USD".to_string(),
/// };
/// ```
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ValidationError {
    /// Required field is missing (XSD minOccurs="1" violation)
    MissingRequiredField {
        /// Name of the missing field
        field: String,
    },

    /// Currency code format validation failed (not ISO 4217 compliant)
    InvalidCurrencyFormat {
        /// The invalid currency code
        code: String,
    },

    /// Currency code length is not exactly 3 characters
    InvalidCurrencyLength {
        /// Expected length (always 3 for ISO 4217)
        expected: usize,
        /// Actual length found
        found: usize,
    },

    /// String exceeds maximum allowed length
    StringTooLong {
        /// Actual string length
        len: usize,
        /// Maximum allowed length
        max: usize,
    },

    /// Nanoseconds field out of valid range (-999999999 to +999999999)
    NanosOutOfRange {
        /// The invalid nanos value
        nanos: i32,
        /// Minimum valid value
        min: i32,
        /// Maximum valid value
        max: i32,
    },

    /// Nanoseconds sign does not match units sign (monetary amount validation)
    NanosSignMismatch {
        /// Units value
        units: i64,
        /// Nanos value
        nanos: i32,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::MissingRequiredField { field } => {
                write!(f, "Missing required field: '{}'", field)
            }
            ValidationError::InvalidCurrencyFormat { code } => {
                write!(f, "Invalid currency code format: '{}'", code)
            }
            ValidationError::InvalidCurrencyLength { expected, found } => {
                write!(
                    f,
                    "Currency code length mismatch: expected {} characters, found {}",
                    expected, found
                )
            }
            ValidationError::StringTooLong { len, max } => {
                write!(f, "String too long: {} characters (max: {})", len, max)
            }
            ValidationError::NanosOutOfRange { nanos, min, max } => {
                write!(
                    f,
                    "Nanoseconds out of range: {} (valid range: {} to {})",
                    nanos, min, max
                )
            }
            ValidationError::NanosSignMismatch { units, nanos } => {
                write!(
                    f,
                    "Nanoseconds sign mismatch: units={}, nanos={} (must have same sign)",
                    units, nanos
                )
            }
        }
    }
}

impl Error for ValidationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

/// Convenience type alias for encoding errors
///
/// Used in codec layer functions that return size-related encoding errors.
pub type EncodeError = NexoError;

/// Convenience type alias for decoding errors
///
/// Used in codec layer functions that return size-related decoding errors.
pub type DecodeError = NexoError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = NexoError::Connection {
            details: "test connection failure",
        };
        assert_eq!(err.to_string(), "Connection failed: test connection failure");

        let err = NexoError::Timeout;
        assert_eq!(err.to_string(), "Operation timed out");

        let err = NexoError::Validation {
            field: "test_field",
            reason: "test reason",
        };
        assert_eq!(
            err.to_string(),
            "Validation failed for 'test_field': test reason"
        );

        let err = NexoError::Encoding {
            details: "test encoding failure",
        };
        assert_eq!(err.to_string(), "Encoding failed: test encoding failure");

        let err = NexoError::Decoding {
            details: "test decoding failure",
        };
        assert_eq!(err.to_string(), "Decoding failed: test decoding failure");
    }

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::MissingRequiredField {
            field: "TestField".to_string(),
        };
        assert_eq!(err.to_string(), "Missing required field: 'TestField'");

        let err = ValidationError::InvalidCurrencyFormat {
            code: "ABC".to_string(),
        };
        assert_eq!(err.to_string(), "Invalid currency code format: 'ABC'");

        let err = ValidationError::InvalidCurrencyLength {
            expected: 3,
            found: 2,
        };
        assert_eq!(
            err.to_string(),
            "Currency code length mismatch: expected 3 characters, found 2"
        );

        let err = ValidationError::StringTooLong { len: 300, max: 256 };
        assert_eq!(
            err.to_string(),
            "String too long: 300 characters (max: 256)"
        );

        let err = ValidationError::NanosOutOfRange {
            nanos: 1_000_000_000,
            min: -999_999_999,
            max: 999_999_999,
        };
        assert_eq!(
            err.to_string(),
            "Nanoseconds out of range: 1000000000 (valid range: -999999999 to 999999999)"
        );

        let err = ValidationError::NanosSignMismatch {
            units: 100,
            nanos: -50,
        };
        assert_eq!(
            err.to_string(),
            "Nanoseconds sign mismatch: units=100, nanos=-50 (must have same sign)"
        );
    }

    #[test]
    fn test_error_source_returns_none() {
        let err = NexoError::Timeout;
        assert!(err.source().is_none());

        let val_err = ValidationError::MissingRequiredField {
            field: "Test".to_string(),
        };
        assert!(val_err.source().is_none());
    }

    #[test]
    fn test_from_prost_errors() {
        // Test DecodeError conversion
        let prost_decode_err = prost::DecodeError::new(0);
        let nexo_err: NexoError = prost_decode_err.into();
        assert!(matches!(nexo_err, NexoError::Decoding { .. }));

        // Test EncodeError conversion
        let prost_encode_err = prost::EncodeError::new(0);
        let nexo_err: NexoError = prost_encode_err.into();
        assert!(matches!(nexo_err, NexoError::Encoding { .. }));
    }
}
