//! String validation for XSD constraints
//!
//! This module provides validation functions for string fields according to
//! XSD constraints defined in the Nexo Retailer Protocol specification.
//!
//! # XSD String Type Mapping
//!
//! The protocol uses several XSD string types with length constraints:
//!
//! | XSD Type | Description | Max Length | Validator |
//! |----------|-------------|------------|-----------|
//! | Max256Text | General text fields | 256 chars | `validate_max256_text` |
//! | Max20000Text | Long text fields | 20,000 chars | `validate_max20000_text` |
//! | Max70Text | Short identifiers | 70 chars | `validate_max70_text` |
//!
//! # UTF-8 Encoding
//!
//! **Note:** This module does not perform UTF-8 validation beyond what Rust's
//! `String` type guarantees. The `prost::alloc::string::String` type (which is
//! a re-export of `alloc::string::String`) is always valid UTF-8 by construction.
//! Invalid UTF-8 cannot exist in a Rust `String`, so additional UTF-8 validation
//! is unnecessary.
//!
//! If you need to validate UTF-8 when parsing from bytes, use
//! `String::from_utf8()` which returns an error if the bytes are invalid UTF-8.
//!
//! # Examples
//!
//! ```rust
//! use nexo_retailer_protocol::validate::strings::{
//!     validate_max_text, validate_max256_text, validate_max20000_text
//! };
//!
//! // Valid strings
//! assert!(validate_max256_text("Hello, World!").is_ok());
//! assert!(validate_max20000_text(&"A".repeat(10000)).is_ok());
//!
//! // Invalid: exceeds limit
//! assert!(validate_max256_text(&"A".repeat(257)).is_err());
//!
//! // Empty string is valid
//! assert!(validate_max256_text("").is_ok());
//! ```

use crate::error::ValidationError;

/// Validate string length against a maximum
///
/// Checks that the string length (in bytes) does not exceed the specified maximum.
/// Note that this counts bytes, not Unicode code points. Multi-byte UTF-8
/// characters count as multiple bytes.
///
/// # Arguments
///
/// * `s` - The string to validate
/// * `max_len` - Maximum allowed length in bytes
///
/// # Returns
///
/// * `Ok(())` if the string is within the length limit
/// * `Err(ValidationError::StringTooLong)` if the string exceeds the limit
///
/// # Examples
///
/// ```rust
/// use nexo_retailer_protocol::validate::strings::validate_max_text;
///
/// // Valid: under limit
/// assert!(validate_max_text("hello", 10).is_ok());
///
/// // Valid: exactly at limit
/// assert!(validate_max_text("hello", 5).is_ok());
///
/// // Invalid: exceeds limit
/// assert!(validate_max_text("hello world", 5).is_err());
/// ```
pub fn validate_max_text(s: &str, max_len: usize) -> Result<(), ValidationError> {
    if s.len() > max_len {
        return Err(ValidationError::StringTooLong {
            len: s.len(),
            max: max_len,
        });
    }
    Ok(())
}

/// Validate Max256Text XSD type
///
/// XSD `Max256Text` is used for general-purpose text fields with a maximum
/// length of 256 characters.
///
/// # Arguments
///
/// * `s` - The string to validate
///
/// # Returns
///
/// * `Ok(())` if the string is 256 bytes or less
/// * `Err(ValidationError::StringTooLong)` if the string exceeds 256 bytes
///
/// # Examples
///
/// ```rust
/// use nexo_retailer_protocol::validate::strings::validate_max256_text;
///
/// assert!(validate_max256_text("SaleRef12345").is_ok());
/// assert!(validate_max256_text(&"A".repeat(256)).is_ok());
/// assert!(validate_max256_text(&"A".repeat(257)).is_err());
/// ```
pub fn validate_max256_text(s: &str) -> Result<(), ValidationError> {
    validate_max_text(s, 256)
}

/// Validate Max20000Text XSD type
///
/// XSD `Max20000Text` is used for long text fields such as detailed messages,
/// debug information, or formatted output. Maximum length is 20,000 bytes.
///
/// # Arguments
///
/// * `s` - The string to validate
///
/// # Returns
///
/// * `Ok(())` if the string is 20,000 bytes or less
/// * `Err(ValidationError::StringTooLong)` if the string exceeds 20,000 bytes
///
/// # Examples
///
/// ```rust
/// use nexo_retailer_protocol::validate::strings::validate_max20000_text;
///
/// assert!(validate_max20000_text("A reasonably long message").is_ok());
/// assert!(validate_max20000_text(&"A".repeat(20000)).is_ok());
/// assert!(validate_max20000_text(&"A".repeat(20001)).is_err());
/// ```
pub fn validate_max20000_text(s: &str) -> Result<(), ValidationError> {
    validate_max_text(s, 20000)
}

/// Validate Max70Text XSD type
///
/// XSD `Max70Text` is used for short identifier fields or labels.
/// Maximum length is 70 bytes.
///
/// # Arguments
///
/// * `s` - The string to validate
///
/// # Returns
///
/// * `Ok(())` if the string is 70 bytes or less
/// * `Err(ValidationError::StringTooLong)` if the string exceeds 70 bytes
///
/// # Examples
///
/// ```rust
/// use nexo_retailer_protocol::validate::strings::validate_max70_text;
///
/// assert!(validate_max70_text("Short label").is_ok());
/// assert!(validate_max70_text(&"A".repeat(70)).is_ok());
/// assert!(validate_max70_text(&"A".repeat(71)).is_err());
/// ```
pub fn validate_max70_text(s: &str) -> Result<(), ValidationError> {
    validate_max_text(s, 70)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // validate_max_text tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_validate_max_text_under_limit() {
        assert!(validate_max_text("hello", 10).is_ok());
        assert!(validate_max_text("", 10).is_ok()); // Empty string
        assert!(validate_max_text("a", 1).is_ok()); // Exactly 1 char
    }

    #[test]
    fn test_validate_max_text_at_limit() {
        assert!(validate_max_text("hello", 5).is_ok());
        assert!(validate_max_text(&"A".repeat(100), 100).is_ok());
    }

    #[test]
    fn test_validate_max_text_over_limit() {
        let result = validate_max_text("hello world", 5);
        assert!(matches!(
            result,
            Err(ValidationError::StringTooLong { len: 11, max: 5 })
        ));

        let result = validate_max_text(&"A".repeat(101), 100);
        assert!(matches!(
            result,
            Err(ValidationError::StringTooLong { len: 101, max: 100 })
        ));
    }

    #[test]
    fn test_validate_max_text_multibyte_utf8() {
        // UTF-8 characters can be multiple bytes
        // "日本語" is 3 characters but 9 bytes (3 bytes each)
        let text = "日本語";
        assert_eq!(text.len(), 9); // 9 bytes
        assert_eq!(text.chars().count(), 3); // 3 characters

        // Validation counts bytes, not characters
        assert!(validate_max_text(text, 9).is_ok()); // Exactly 9 bytes
        assert!(validate_max_text(text, 8).is_err()); // Less than 9 bytes
    }

    // ------------------------------------------------------------------------
    // validate_max256_text tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_validate_max256_text_valid() {
        assert!(validate_max256_text("SaleRef12345").is_ok());
        assert!(validate_max256_text("").is_ok());
        assert!(validate_max256_text(&"A".repeat(256)).is_ok());
    }

    #[test]
    fn test_validate_max256_text_invalid() {
        let result = validate_max256_text(&"A".repeat(257));
        assert!(matches!(
            result,
            Err(ValidationError::StringTooLong { len: 257, max: 256 })
        ));
    }

    #[test]
    fn test_validate_max256_text_multibyte() {
        // Create a string with multibyte characters
        // Each Japanese character is 3 bytes in UTF-8
        let text = "日本語"; // 9 bytes total
        assert!(validate_max256_text(&text).is_ok());

        // Create a string that's within 256 bytes
        let text = "日本語".repeat(28); // 252 bytes (9 * 28)
        assert!(validate_max256_text(&text).is_ok());

        // Create a string that exceeds 256 bytes
        let text = "日本語".repeat(29); // 261 bytes (9 * 29)
        assert!(validate_max256_text(&text).is_err());
    }

    // ------------------------------------------------------------------------
    // validate_max20000_text tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_validate_max20000_text_valid() {
        assert!(validate_max20000_text("A reasonably long message").is_ok());
        assert!(validate_max20000_text("").is_ok());
        assert!(validate_max20000_text(&"A".repeat(10000)).is_ok());
        assert!(validate_max20000_text(&"A".repeat(20000)).is_ok());
    }

    #[test]
    fn test_validate_max20000_text_invalid() {
        let result = validate_max20000_text(&"A".repeat(20001));
        assert!(matches!(
            result,
            Err(ValidationError::StringTooLong { len: 20001, max: 20000 })
        ));
    }

    #[test]
    fn test_validate_max20000_text_large_string() {
        // Test with a large but valid string
        let large_string = "A".repeat(19000) + &"B".repeat(999); // 19999 bytes
        assert!(validate_max20000_text(&large_string).is_ok());

        // One byte over the limit
        let too_large = large_string + "C"; // 19999 + 1 = 20000 bytes (still valid)
        assert!(validate_max20000_text(&too_large).is_ok());

        // Two bytes over the limit
        let too_large = too_large + "D"; // 20001 bytes (invalid)
        assert!(validate_max20000_text(&too_large).is_err());
    }

    // ------------------------------------------------------------------------
    // validate_max70_text tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_validate_max70_text_valid() {
        assert!(validate_max70_text("Short label").is_ok());
        assert!(validate_max70_text("").is_ok());
        assert!(validate_max70_text(&"A".repeat(70)).is_ok());
    }

    #[test]
    fn test_validate_max70_text_invalid() {
        let result = validate_max70_text(&"A".repeat(71));
        assert!(matches!(
            result,
            Err(ValidationError::StringTooLong { len: 71, max: 70 })
        ));
    }

    #[test]
    fn test_validate_max70_text_id_fields() {
        // Test typical ID field lengths
        assert!(validate_max70_text("TX-12345-67890").is_ok());
        assert!(validate_max70_text("Message-ID-2024-02-28-001").is_ok());
    }

    // ------------------------------------------------------------------------
    // UTF-8 encoding tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_utf8_emoji() {
        // Emoji are multibyte in UTF-8
        let emoji = "😀🎉"; // 2 emoji, 8 bytes total
        assert_eq!(emoji.len(), 8);

        assert!(validate_max_text(emoji, 8).is_ok());
        assert!(validate_max_text(emoji, 7).is_err());
    }

    #[test]
    fn test_utf8_mixed_script() {
        // Mixed scripts: Latin, Cyrillic, CJK
        // "Hello" = 5 bytes (ASCII)
        // "Привет" = 12 bytes (Cyrillic is 2 bytes per char)
        // "こんにちは" = 15 bytes (Japanese hiragana is 3 bytes per char, 5 chars)
        // Total = 5 + 12 + 15 = 32 bytes
        let mixed = "HelloПриветこんにちは";
        assert_eq!(mixed.len(), 32);

        assert!(validate_max256_text(&mixed).is_ok());
        assert!(validate_max_text(&mixed, 32).is_ok());
        assert!(validate_max_text(&mixed, 31).is_err());
    }

    #[test]
    fn test_empty_string_always_valid() {
        // Empty string should always pass validation
        assert!(validate_max_text("", 0).is_ok());
        assert!(validate_max256_text("").is_ok());
        assert!(validate_max20000_text("").is_ok());
        assert!(validate_max70_text("").is_ok());
    }

    #[test]
    fn test_zero_max_length() {
        // Edge case: max length of 0 means only empty string is valid
        assert!(validate_max_text("", 0).is_ok());
        assert!(validate_max_text("a", 0).is_err());
    }

    #[test]
    fn test_string_with_newlines() {
        // Strings with newlines and special characters
        let text = "Line 1\nLine 2\r\nLine 3\tTabbed";
        assert!(validate_max256_text(text).is_ok());
    }

    // ------------------------------------------------------------------------
    // Additional boundary tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_validate_max256_text_exactly_at_boundary() {
        // Test at exactly 256 bytes (should pass)
        let text = "A".repeat(256);
        assert!(validate_max256_text(&text).is_ok());
    }

    #[test]
    fn test_validate_max256_text_one_over_boundary() {
        // Test at 257 bytes (should fail)
        let text = "A".repeat(257);
        let result = validate_max256_text(&text);
        assert!(result.is_err());
        // Verify error message includes actual length
        let err = result.unwrap_err();
        assert!(err.to_string().contains("257"));
    }

    #[test]
    fn test_validate_max70_text_exactly_at_boundary() {
        // Test at exactly 70 bytes (should pass)
        let text = "A".repeat(70);
        assert!(validate_max70_text(&text).is_ok());
    }

    #[test]
    fn test_validate_max70_text_one_over_boundary() {
        // Test at 71 bytes (should fail)
        let text = "A".repeat(71);
        let result = validate_max70_text(&text);
        assert!(result.is_err());
        // Verify error message includes actual length
        let err = result.unwrap_err();
        assert!(err.to_string().contains("71"));
    }

    #[test]
    fn test_validate_max20000_text_exactly_at_boundary() {
        // Test at exactly 20000 bytes (should pass)
        let text = "A".repeat(20000);
        assert!(validate_max20000_text(&text).is_ok());
    }

    #[test]
    fn test_validate_max20000_text_one_over_boundary() {
        // Test at 20001 bytes (should fail)
        let text = "A".repeat(20001);
        let result = validate_max20000_text(&text);
        assert!(result.is_err());
        // Verify error message includes actual length
        let err = result.unwrap_err();
        assert!(err.to_string().contains("20001"));
    }

    #[test]
    fn test_validate_max_text_custom_limits() {
        // Test validate_max_text with various custom limits
        assert!(validate_max_text("hello", 5).is_ok());
        assert!(validate_max_text("hello", 6).is_ok());
        assert!(validate_max_text("hello", 4).is_err());

        // Test with limit of 1
        assert!(validate_max_text("a", 1).is_ok());
        assert!(validate_max_text("ab", 1).is_err());

        // Test with limit of 1000
        let text = "A".repeat(1000);
        assert!(validate_max_text(&text, 1000).is_ok());
        assert!(validate_max_text(&text, 999).is_err());
    }

    #[test]
    fn test_validate_max_text_error_message_content() {
        let result = validate_max_text("hello world", 5);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();

        // Error message should include actual length and max
        assert!(err_msg.contains("11"), "Should contain actual length");
        assert!(err_msg.contains("5"), "Should contain max length");
    }
}
