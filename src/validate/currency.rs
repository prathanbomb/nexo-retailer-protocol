//! Currency code validation for ISO 4217 compliance
//!
//! This module provides validation for currency codes and monetary amounts
//! according to the Nexo Retailer Protocol specification and ISO 4217 standard.
//!
//! # ISO 4217 Currency Code Validation
//!
//! Currency codes must follow the pattern `[A-Z]{3,3}`:
//! - Exactly 3 characters long
//! - Only uppercase ASCII letters (A-Z)
//! - No numbers, special characters, or lowercase letters
//!
//! **Note:** This module validates format only, not semantic correctness.
//! For example, "XYZ" passes format validation even though it's not a real
//! ISO 4217 currency code. Semantic validation (checking against the official
//! ISO 4217 currency list) is deferred to application-layer logic.
//!
//! # Monetary Amount Validation
//!
//! The `ActiveCurrencyAndAmount` type uses integer representation to avoid
//! IEEE 754 floating-point precision loss:
//! - `units`: Whole currency units (int64)
//! - `nanos`: Nano (10^-9) units (int32)
//!
//! Nanos constraints:
//! - Range: -999,999,999 to +999,999,999
//! - Sign must match units (both positive or both negative)
//! - Zero is special case: units=0, nanos=0 is valid
//!
//! # Examples
//!
//! ```rust
//! use nexo_retailer_protocol::validate::currency::{validate_currency_code, validate_monetary_amount};
//! use nexo_retailer_protocol::ActiveCurrencyAndAmount;
//!
//! // Valid currency codes
//! assert!(validate_currency_code("USD").is_ok());
//! assert!(validate_currency_code("EUR").is_ok());
//! assert!(validate_currency_code("JPY").is_ok());
//!
//! // Invalid currency codes
//! assert!(validate_currency_code("US").is_err());  // Too short
//! assert!(validate_currency_code("USDD").is_err()); // Too long
//! assert!(validate_currency_code("usd").is_err());  // Lowercase
//! assert!(validate_currency_code("123").is_err());  // Numbers
//!
//! // Valid monetary amount
//! let amount = ActiveCurrencyAndAmount {
//!     ccy: "USD".to_string(),
//!     units: 100,
//!     nanos: 500000000, // 0.5
//! };
//! assert!(validate_monetary_amount(&amount).is_ok());
//! ```

use crate::error::ValidationError;

/// Validate ISO 4217 currency code format
///
/// Checks that the currency code follows the pattern `[A-Z]{3,3}`:
/// - Exactly 3 characters
/// - Only uppercase ASCII letters (A-Z)
///
/// # Arguments
///
/// * `code` - The currency code to validate
///
/// # Returns
///
/// * `Ok(())` if the code is valid
/// * `Err(ValidationError)` if the code is invalid
///
/// # Examples
///
/// ```rust
/// use nexo_retailer_protocol::validate::currency::validate_currency_code;
///
/// // Valid codes
/// assert!(validate_currency_code("USD").is_ok());
/// assert!(validate_currency_code("EUR").is_ok());
/// assert!(validate_currency_code("JPY").is_ok());
///
/// // Invalid length
/// assert!(validate_currency_code("US").is_err());
/// assert!(validate_currency_code("USDD").is_err());
///
/// // Invalid format
/// assert!(validate_currency_code("usd").is_err());
/// assert!(validate_currency_code("Usd").is_err());
/// assert!(validate_currency_code("123").is_err());
/// assert!(validate_currency_code("$$$").is_err());
/// ```
pub fn validate_currency_code(code: &str) -> Result<(), ValidationError> {
    // Check length: ISO 4217 codes are exactly 3 characters
    if code.len() != 3 {
        return Err(ValidationError::InvalidCurrencyLength {
            expected: 3,
            found: code.len(),
        });
    }

    // Check format: Only uppercase ASCII letters
    if !code.bytes().all(|b| b.is_ascii_uppercase()) {
        return Err(ValidationError::InvalidCurrencyFormat {
            code: "currency_code",
        });
    }

    Ok(())
}

/// Validate monetary amount according to XSD constraints
///
/// Validates an `ActiveCurrencyAndAmount` structure:
/// 1. Currency code format (via `validate_currency_code`)
/// 2. Nanos range: -999,999,999 to +999,999,999
/// 3. Nanos sign matches units sign
///
/// # Arguments
///
/// * `amount` - The monetary amount to validate
///
/// # Returns
///
/// * `Ok(())` if the amount is valid
/// * `Err(ValidationError)` if validation fails
///
/// # Examples
///
/// ```rust
/// use nexo_retailer_protocol::validate::currency::validate_monetary_amount;
/// use nexo_retailer_protocol::ActiveCurrencyAndAmount;
///
/// // Valid amount
/// let amount = ActiveCurrencyAndAmount {
///     ccy: "USD".to_string(),
///     units: 100,
///     nanos: 500000000,
/// };
/// assert!(validate_monetary_amount(&amount).is_ok());
///
/// // Invalid currency code
/// let amount = ActiveCurrencyAndAmount {
///     ccy: "usd".to_string(),  // Lowercase
///     units: 100,
///     nanos: 0,
/// };
/// assert!(validate_monetary_amount(&amount).is_err());
///
/// // Nanos out of range
/// let amount = ActiveCurrencyAndAmount {
///     ccy: "USD".to_string(),
///     units: 100,
///     nanos: 1_000_000_000,  // Exceeds 999,999,999
/// };
/// assert!(validate_monetary_amount(&amount).is_err());
///
/// // Sign mismatch
/// let amount = ActiveCurrencyAndAmount {
///     ccy: "USD".to_string(),
///     units: 100,
///     nanos: -500000000,  // Positive units, negative nanos
/// };
/// assert!(validate_monetary_amount(&amount).is_err());
/// ```
pub fn validate_monetary_amount(amount: &crate::ActiveCurrencyAndAmount) -> Result<(), ValidationError> {
    // Validate currency code format
    validate_currency_code(&amount.ccy)?;

    // Validate nanos range: -999999999 to +999999999
    const NANOS_MIN: i32 = -999_999_999;
    const NANOS_MAX: i32 = 999_999_999;

    if amount.nanos < NANOS_MIN || amount.nanos > NANOS_MAX {
        return Err(ValidationError::NanosOutOfRange {
            nanos: amount.nanos,
            min: NANOS_MIN,
            max: NANOS_MAX,
        });
    }

    // Validate nanos sign matches units sign
    // Special case: if units is 0, nanos can be anything in range
    // (though logically nanos should also be 0 for zero amount)
    if amount.units > 0 && amount.nanos < 0 {
        return Err(ValidationError::NanosSignMismatch {
            units: amount.units,
            nanos: amount.nanos,
        });
    }

    if amount.units < 0 && amount.nanos > 0 {
        return Err(ValidationError::NanosSignMismatch {
            units: amount.units,
            nanos: amount.nanos,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Import ToString for no_std tests with alloc
    #[cfg(feature = "alloc")]
    use prost::alloc::string::ToString;

    // ------------------------------------------------------------------------
    // Currency Code Validation Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_valid_currency_codes() {
        // Common ISO 4217 codes
        let valid_codes = ["USD", "EUR", "GBP", "JPY", "CAD", "AUD", "CHF", "CNY"];

        for code in valid_codes {
            assert!(validate_currency_code(code).is_ok(), "{} should be valid", code);
        }
    }

    #[test]
    fn test_invalid_length_too_short() {
        let result = validate_currency_code("US");
        assert!(matches!(result, Err(ValidationError::InvalidCurrencyLength { expected: 3, found: 2 })));

        let result = validate_currency_code("");
        assert!(matches!(result, Err(ValidationError::InvalidCurrencyLength { expected: 3, found: 0 })));
    }

    #[test]
    fn test_invalid_length_too_long() {
        let result = validate_currency_code("USDD");
        assert!(matches!(result, Err(ValidationError::InvalidCurrencyLength { expected: 3, found: 4 })));

        let result = validate_currency_code("US Dollars");
        assert!(matches!(result, Err(ValidationError::InvalidCurrencyLength { .. })));
    }

    #[test]
    fn test_invalid_format_lowercase() {
        let result = validate_currency_code("usd");
        assert!(matches!(result, Err(ValidationError::InvalidCurrencyFormat { .. })));

        let result = validate_currency_code("Usd");
        assert!(matches!(result, Err(ValidationError::InvalidCurrencyFormat { .. })));

        let _result = validate_currency_code("USD"); // Note: this is valid
        assert!(validate_currency_code("USD").is_ok());
    }

    #[test]
    fn test_invalid_format_numbers() {
        let result = validate_currency_code("123");
        assert!(matches!(result, Err(ValidationError::InvalidCurrencyFormat { .. })));

        let result = validate_currency_code("U1D");
        assert!(matches!(result, Err(ValidationError::InvalidCurrencyFormat { .. })));
    }

    #[test]
    fn test_invalid_format_special_chars() {
        let result = validate_currency_code("$$D");
        assert!(matches!(result, Err(ValidationError::InvalidCurrencyFormat { .. })));

        let result = validate_currency_code("U-D");
        assert!(matches!(result, Err(ValidationError::InvalidCurrencyFormat { .. })));

        let result = validate_currency_code("U D");
        assert!(matches!(result, Err(ValidationError::InvalidCurrencyFormat { .. })));
    }

    // ------------------------------------------------------------------------
    // Monetary Amount Validation Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_valid_monetary_amount() {
        let amount = crate::ActiveCurrencyAndAmount {
            ccy: "USD".to_string(),
            units: 100,
            nanos: 500000000, // 100.50
        };
        assert!(validate_monetary_amount(&amount).is_ok());
    }

    #[test]
    fn test_valid_zero_amount() {
        let amount = crate::ActiveCurrencyAndAmount {
            ccy: "USD".to_string(),
            units: 0,
            nanos: 0,
        };
        assert!(validate_monetary_amount(&amount).is_ok());
    }

    #[test]
    fn test_valid_negative_amount() {
        let amount = crate::ActiveCurrencyAndAmount {
            ccy: "USD".to_string(),
            units: -100,
            nanos: -500000000, // -100.50
        };
        assert!(validate_monetary_amount(&amount).is_ok());
    }

    #[test]
    fn test_valid_nanos_at_boundaries() {
        // Maximum positive nanos
        let amount = crate::ActiveCurrencyAndAmount {
            ccy: "USD".to_string(),
            units: 1,
            nanos: 999_999_999,
        };
        assert!(validate_monetary_amount(&amount).is_ok());

        // Maximum negative nanos
        let amount = crate::ActiveCurrencyAndAmount {
            ccy: "USD".to_string(),
            units: -1,
            nanos: -999_999_999,
        };
        assert!(validate_monetary_amount(&amount).is_ok());
    }

    #[test]
    fn test_invalid_currency_code_in_amount() {
        let amount = crate::ActiveCurrencyAndAmount {
            ccy: "usd".to_string(), // Lowercase
            units: 100,
            nanos: 0,
        };
        assert!(validate_monetary_amount(&amount).is_err());
    }

    #[test]
    fn test_nanos_out_of_range_positive() {
        let amount = crate::ActiveCurrencyAndAmount {
            ccy: "USD".to_string(),
            units: 1,
            nanos: 1_000_000_000, // Exceeds 999,999,999
        };
        let result = validate_monetary_amount(&amount);
        assert!(matches!(
            result,
            Err(ValidationError::NanosOutOfRange {
                nanos: 1_000_000_000,
                min: -999_999_999,
                max: 999_999_999
            })
        ));
    }

    #[test]
    fn test_nanos_out_of_range_negative() {
        let amount = crate::ActiveCurrencyAndAmount {
            ccy: "USD".to_string(),
            units: -1,
            nanos: -1_000_000_000, // Below -999,999,999
        };
        let result = validate_monetary_amount(&amount);
        assert!(matches!(
            result,
            Err(ValidationError::NanosOutOfRange {
                nanos: -1_000_000_000,
                min: -999_999_999,
                max: 999_999_999
            })
        ));
    }

    #[test]
    fn test_sign_mismatch_positive_units_negative_nanos() {
        let amount = crate::ActiveCurrencyAndAmount {
            ccy: "USD".to_string(),
            units: 100,
            nanos: -500000000, // Sign mismatch
        };
        let result = validate_monetary_amount(&amount);
        assert!(matches!(
            result,
            Err(ValidationError::NanosSignMismatch {
                units: 100,
                nanos: -500000000
            })
        ));
    }

    #[test]
    fn test_sign_mismatch_negative_units_positive_nanos() {
        let amount = crate::ActiveCurrencyAndAmount {
            ccy: "USD".to_string(),
            units: -100,
            nanos: 500000000, // Sign mismatch
        };
        let result = validate_monetary_amount(&amount);
        assert!(matches!(
            result,
            Err(ValidationError::NanosSignMismatch {
                units: -100,
                nanos: 500000000
            })
        ));
    }

    #[test]
    fn test_zero_units_with_nonzero_nanos() {
        // This is technically valid per the validation rules
        // (units=0 is a special case where nanos sign doesn't matter)
        let amount = crate::ActiveCurrencyAndAmount {
            ccy: "USD".to_string(),
            units: 0,
            nanos: 500000000,
        };
        assert!(validate_monetary_amount(&amount).is_ok());

        let amount = crate::ActiveCurrencyAndAmount {
            ccy: "USD".to_string(),
            units: 0,
            nanos: -500000000,
        };
        assert!(validate_monetary_amount(&amount).is_ok());
    }

    #[test]
    fn test_i32_max_and_min() {
        // Ensure we handle edge cases with i32::MAX and i32::MIN
        let amount = crate::ActiveCurrencyAndAmount {
            ccy: "USD".to_string(),
            units: 1,
            nanos: i32::MAX, // Should fail (exceeds 999,999,999)
        };
        assert!(validate_monetary_amount(&amount).is_err());

        let amount = crate::ActiveCurrencyAndAmount {
            ccy: "USD".to_string(),
            units: -1,
            nanos: i32::MIN, // Should fail (below -999,999,999)
        };
        assert!(validate_monetary_amount(&amount).is_err());
    }
}
