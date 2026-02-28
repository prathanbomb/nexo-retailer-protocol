//! Error code constants and categories for Nexo Retailer Protocol
//!
//! This module defines numeric error codes and categories that can be used
//! for programmatic error handling, logging, and error response mapping.
//!
//! # Error Code Taxonomy
//!
//! Error codes are organized by category:
//! - **0xxx**: Connection/Transport errors
//! - **1xxx**: Timeout errors
//! - **2xxx**: Validation errors
//! - **3xxx**: Encoding errors
//! - **4xxx**: Decoding errors
//!
//! # Usage
//!
//! ```rust,no_run
//! use nexo_retailer_protocol::error::codes;
//!
//! // Check error category
//! let code = codes::ERR_CONNECTION_REFUSED;
//! if codes::is_connection_error(code) {
//!     println!("Connection error: {}", code);
//! }
//! ```

/// Connection error code range: 0000-0999
pub const ERR_CONNECTION_BASE: u16 = 0;

/// Timeout error code range: 1000-1999
pub const ERR_TIMEOUT_BASE: u16 = 1000;

/// Validation error code range: 2000-2999
pub const ERR_VALIDATION_BASE: u16 = 2000;

/// Encoding error code range: 3000-3999
pub const ERR_ENCODING_BASE: u16 = 3000;

/// Decoding error code range: 4000-4999
pub const ERR_DECODING_BASE: u16 = 4000;

// ============================================================================
// Connection Error Codes (0xxx)
// ============================================================================

/// Connection refused by remote endpoint
pub const ERR_CONNECTION_REFUSED: u16 = ERR_CONNECTION_BASE + 1;

/// Connection timeout during initial connection
pub const ERR_CONNECTION_TIMEOUT: u16 = ERR_CONNECTION_BASE + 2;

/// Network unreachable
pub const ERR_NETWORK_UNREACHABLE: u16 = ERR_CONNECTION_BASE + 3;

/// Connection reset by peer
pub const ERR_CONNECTION_RESET: u16 = ERR_CONNECTION_BASE + 4;

/// Host unreachable
pub const ERR_HOST_UNREACHABLE: u16 = ERR_CONNECTION_BASE + 5;

// ============================================================================
// Timeout Error Codes (1xxx)
// ============================================================================

/// Operation timed out (generic timeout)
pub const ERR_TIMEOUT: u16 = ERR_TIMEOUT_BASE + 1;

/// Read timeout (no data received within timeout period)
pub const ERR_READ_TIMEOUT: u16 = ERR_TIMEOUT_BASE + 2;

/// Write timeout (unable to send data within timeout period)
pub const ERR_WRITE_TIMEOUT: u16 = ERR_TIMEOUT_BASE + 3;

// ============================================================================
// Validation Error Codes (2xxx)
// ============================================================================

/// Required field is missing
pub const ERR_MISSING_REQUIRED_FIELD: u16 = ERR_VALIDATION_BASE + 1;

/// Invalid currency code format
pub const ERR_INVALID_CURRENCY_FORMAT: u16 = ERR_VALIDATION_BASE + 2;

/// Currency code length is incorrect
pub const ERR_INVALID_CURRENCY_LENGTH: u16 = ERR_VALIDATION_BASE + 3;

/// String exceeds maximum allowed length
pub const ERR_STRING_TOO_LONG: u16 = ERR_VALIDATION_BASE + 4;

/// Nanoseconds field out of valid range
pub const ERR_NANOS_OUT_OF_RANGE: u16 = ERR_VALIDATION_BASE + 5;

/// Nanoseconds sign does not match units sign
pub const ERR_NANOS_SIGN_MISMATCH: u16 = ERR_VALIDATION_BASE + 6;

/// Invalid enum value
pub const ERR_INVALID_ENUM_VALUE: u16 = ERR_VALIDATION_BASE + 7;

/// Type mismatch (e.g., expected string, got integer)
pub const ERR_TYPE_MISMATCH: u16 = ERR_VALIDATION_BASE + 8;

// ============================================================================
// Encoding Error Codes (3xxx)
// ============================================================================

/// Generic encoding error
pub const ERR_ENCODING_FAILED: u16 = ERR_ENCODING_BASE + 1;

/// Message size exceeds maximum allowed size
pub const ERR_MESSAGE_TOO_LARGE: u16 = ERR_ENCODING_BASE + 2;

/// Invalid field value during encoding
pub const ERR_INVALID_FIELD_VALUE: u16 = ERR_ENCODING_BASE + 3;

// ============================================================================
// Decoding Error Codes (4xxx)
// ============================================================================

/// Generic decoding error
pub const ERR_DECODING_FAILED: u16 = ERR_DECODING_BASE + 1;

/// Invalid wire format
pub const ERR_INVALID_WIRE_FORMAT: u16 = ERR_DECODING_BASE + 2;

/// Unknown field in message
pub const ERR_UNKNOWN_FIELD: u16 = ERR_DECODING_BASE + 3;

/// Truncated message (incomplete data)
pub const ERR_TRUNCATED_MESSAGE: u16 = ERR_DECODING_BASE + 4;

/// Message size exceeds maximum during decode
pub const ERR_MESSAGE_SIZE_EXCEEDED: u16 = ERR_DECODING_BASE + 5;

// ============================================================================
// Error Category Helpers
// ============================================================================

/// Check if error code is in the connection error category
///
/// # Example
/// ```rust
/// use nexo_retailer_protocol::error::codes;
///
/// let code = codes::ERR_CONNECTION_REFUSED;
/// assert!(codes::is_connection_error(code));
/// ```
#[inline]
pub const fn is_connection_error(code: u16) -> bool {
    code >= ERR_CONNECTION_BASE && code < ERR_TIMEOUT_BASE
}

/// Check if error code is in the timeout error category
///
/// # Example
/// ```rust
/// use nexo_retailer_protocol::error::codes;
///
/// let code = codes::ERR_TIMEOUT;
/// assert!(codes::is_timeout_error(code));
/// ```
#[inline]
pub const fn is_timeout_error(code: u16) -> bool {
    code >= ERR_TIMEOUT_BASE && code < ERR_VALIDATION_BASE
}

/// Check if error code is in the validation error category
///
/// # Example
/// ```rust
/// use nexo_retailer_protocol::error::codes;
///
/// let code = codes::ERR_MISSING_REQUIRED_FIELD;
/// assert!(codes::is_validation_error(code));
/// ```
#[inline]
pub const fn is_validation_error(code: u16) -> bool {
    code >= ERR_VALIDATION_BASE && code < ERR_ENCODING_BASE
}

/// Check if error code is in the encoding error category
///
/// # Example
/// ```rust
/// use nexo_retailer_protocol::error::codes;
///
/// let code = codes::ERR_ENCODING_FAILED;
/// assert!(codes::is_encoding_error(code));
/// ```
#[inline]
pub const fn is_encoding_error(code: u16) -> bool {
    code >= ERR_ENCODING_BASE && code < ERR_DECODING_BASE
}

/// Check if error code is in the decoding error category
///
/// # Example
/// ```rust
/// use nexo_retailer_protocol::error::codes;
///
/// let code = codes::ERR_DECODING_FAILED;
/// assert!(codes::is_decoding_error(code));
/// ```
#[inline]
pub const fn is_decoding_error(code: u16) -> bool {
    code >= ERR_DECODING_BASE && code < 5000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_ranges() {
        // Connection errors
        assert!(is_connection_error(ERR_CONNECTION_REFUSED));
        assert!(is_connection_error(ERR_CONNECTION_TIMEOUT));
        assert!(is_connection_error(ERR_NETWORK_UNREACHABLE));
        assert!(!is_connection_error(ERR_TIMEOUT));

        // Timeout errors
        assert!(is_timeout_error(ERR_TIMEOUT));
        assert!(is_timeout_error(ERR_READ_TIMEOUT));
        assert!(is_timeout_error(ERR_WRITE_TIMEOUT));
        assert!(!is_timeout_error(ERR_CONNECTION_REFUSED));
        assert!(!is_timeout_error(ERR_MISSING_REQUIRED_FIELD));

        // Validation errors
        assert!(is_validation_error(ERR_MISSING_REQUIRED_FIELD));
        assert!(is_validation_error(ERR_INVALID_CURRENCY_FORMAT));
        assert!(is_validation_error(ERR_STRING_TOO_LONG));
        assert!(!is_validation_error(ERR_TIMEOUT));
        assert!(!is_validation_error(ERR_ENCODING_FAILED));

        // Encoding errors
        assert!(is_encoding_error(ERR_ENCODING_FAILED));
        assert!(is_encoding_error(ERR_MESSAGE_TOO_LARGE));
        assert!(!is_encoding_error(ERR_TIMEOUT));
        assert!(!is_encoding_error(ERR_DECODING_FAILED));

        // Decoding errors
        assert!(is_decoding_error(ERR_DECODING_FAILED));
        assert!(is_decoding_error(ERR_INVALID_WIRE_FORMAT));
        assert!(is_decoding_error(ERR_TRUNCATED_MESSAGE));
        assert!(!is_decoding_error(ERR_TIMEOUT));
        assert!(!is_decoding_error(ERR_ENCODING_FAILED));
    }

    #[test]
    fn test_error_code_values() {
        // Verify error codes are in expected ranges
        assert!(ERR_CONNECTION_REFUSED < 1000);
        assert!(ERR_TIMEOUT >= 1000 && ERR_TIMEOUT < 2000);
        assert!(ERR_MISSING_REQUIRED_FIELD >= 2000 && ERR_MISSING_REQUIRED_FIELD < 3000);
        assert!(ERR_ENCODING_FAILED >= 3000 && ERR_ENCODING_FAILED < 4000);
        assert!(ERR_DECODING_FAILED >= 4000 && ERR_DECODING_FAILED < 5000);
    }
}
