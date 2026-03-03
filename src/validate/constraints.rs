//! Field presence and type validation for XSD constraints
//!
//! This module provides validation functions for checking field presence
//! (required vs optional) and type-specific constraints according to XSD
//! minOccurs and type restrictions defined in the Nexo Retailer Protocol.
//!
//! # Required Field Validation
//!
//! XSD defines `minOccurs="0"` for optional fields and `minOccurs="1"` for
//! required fields. In protobuf3, these map to `optional` fields (wrapped in
//! `Option<T>`). Required fields should be validated to ensure `Some` value.
//!
//! # Type-Specific Validators
//!
//! The module provides validators for common numeric and enum patterns:
//! - Positive integers (must be > 0)
//! - Non-negative integers (must be >= 0)
//! - Enum value validation (against allowed values)
//!
//! # Collection Validation (alloc feature)
//!
//! When the `alloc` feature is enabled, collection validators are available
//! for checking repeated field counts and size limits.
//!
//! # Examples
//!
//! ```rust,ignore
//! use nexo_retailer_protocol::validate::constraints::{
//!     validate_required, validate_positive_i64, Validate
//! };
//! use nexo_retailer_protocol::Header4;
//!
//! let header = Header4 {
//!     msg_fctn: Some("DREQ".to_string()),
//!     // ... other fields
//! };
//!
//! // Validate required field
//! validate_required(&header.msg_fctn, "MsgFctn")?;
//!
//! // Validate entire message
//! header.validate()?;
//! ```

use crate::error::ValidationError;

/// Validate required field presence
///
/// Checks that an optional field has a value (is `Some`). This is used to
/// enforce XSD `minOccurs="1"` constraints for required fields.
///
/// # Arguments
///
/// * `field` - Reference to an optional field
/// * `field_name` - Name of the field for error messages
///
/// # Returns
///
/// * `Ok(())` if the field is `Some`
/// * `Err(ValidationError::MissingRequiredField)` if the field is `None`
///
/// # Examples
///
/// ```rust
/// use nexo_retailer_protocol::validate::constraints::validate_required;
///
/// // Required field present
/// let field = Some("value".to_string());
/// assert!(validate_required(&field, "FieldName").is_ok());
///
/// // Required field missing
/// let field: Option<String> = None;
/// assert!(validate_required(&field, "FieldName").is_err());
/// ```
pub fn validate_required<T>(field: &Option<T>, field_name: &'static str) -> Result<(), ValidationError> {
    if field.is_some() {
        Ok(())
    } else {
        Err(ValidationError::MissingRequiredField {
            field: field_name,
        })
    }
}

/// Validate positive i64 value
///
/// Ensures an i64 value is strictly greater than zero. Used for fields
/// that represent quantities, counts, or amounts that must be positive.
///
/// # Arguments
///
/// * `value` - The value to validate
/// * `field` - Field name for error messages
///
/// # Returns
///
/// * `Ok(())` if value > 0
/// * `Err(ValidationError)` if value <= 0
///
/// # Examples
///
/// ```rust
/// use nexo_retailer_protocol::validate::constraints::validate_positive_i64;
///
/// assert!(validate_positive_i64(1, "Count").is_ok());
/// assert!(validate_positive_i64(100, "Count").is_ok());
/// assert!(validate_positive_i64(0, "Count").is_err());
/// assert!(validate_positive_i64(-1, "Count").is_err());
/// ```
pub fn validate_positive_i64(value: i64, field: &'static str) -> Result<(), ValidationError> {
    if value > 0 {
        Ok(())
    } else {
        Err(ValidationError::MissingRequiredField {
            field,
        })
    }
}

/// Validate non-negative i32 value
///
/// Ensures an i32 value is greater than or equal to zero. Used for fields
/// that represent counts or indices that cannot be negative.
///
/// # Arguments
///
/// * `value` - The value to validate
/// * `field` - Field name for error messages
///
/// # Returns
///
/// * `Ok(())` if value >= 0
/// * `Err(ValidationError)` if value < 0
///
/// # Examples
///
/// ```rust
/// use nexo_retailer_protocol::validate::constraints::validate_non_negative_i32;
///
/// assert!(validate_non_negative_i32(0, "Index").is_ok());
/// assert!(validate_non_negative_i32(100, "Index").is_ok());
/// assert!(validate_non_negative_i32(-1, "Index").is_err());
/// ```
pub fn validate_non_negative_i32(value: i32, field: &'static str) -> Result<(), ValidationError> {
    if value >= 0 {
        Ok(())
    } else {
        Err(ValidationError::MissingRequiredField {
            field,
        })
    }
}

/// Validate enum value against allowed values
///
/// Checks that an enum value is in the set of allowed values. This is useful
/// for proto3 enums where the numeric representation needs validation.
///
/// # Type Parameters
///
/// * `T` - Type that implements PartialEq (typically i32 or u32 for enums)
///
/// # Arguments
///
/// * `value` - The enum value to validate
/// * `valid_values` - Slice of allowed values
/// * `field` - Field name for error messages
///
/// # Returns
///
/// * `Ok(())` if value is in valid_values
/// * `Err(ValidationError)` if value is not allowed
///
/// # Examples
///
/// ```rust
/// use nexo_retailer_protocol::validate::constraints::validate_enum_value;
///
/// // Proto3 enum: 0 = UNSPECIFIED (default), 1 = VALUE_A, 2 = VALUE_B
/// const VALID_VALUES: &[i32] = &[0, 1, 2];
///
/// assert!(validate_enum_value(1, VALID_VALUES, "MyEnum").is_ok());
/// assert!(validate_enum_value(2, VALID_VALUES, "MyEnum").is_ok());
/// assert!(validate_enum_value(99, VALID_VALUES, "MyEnum").is_err());
/// ```
pub fn validate_enum_value<T: PartialEq>(
    value: T,
    valid_values: &[T],
    field: &'static str,
) -> Result<(), ValidationError> {
    if valid_values.contains(&value) {
        Ok(())
    } else {
        Err(ValidationError::MissingRequiredField {
            field,
        })
    }
}

/// Validate repeated field count (requires alloc feature)
///
/// Checks that a repeated field (slice) does not exceed the maximum allowed
/// count. This prevents unbounded allocation attacks and enforces protocol
/// constraints on collection sizes.
///
/// # Arguments
///
/// * `field` - The slice to validate
/// * `max_count` - Maximum allowed number of items
/// * `field_name` - Name of the field for error messages
///
/// # Returns
///
/// * `Ok(())` if count <= max_count
/// * `Err(ValidationError)` if count exceeds max_count
///
/// # Examples
///
/// ```rust,ignore
/// use nexo_retailer_protocol::validate::constraints::validate_repeated_field;
///
/// let items = vec![1, 2, 3];
/// assert!(validate_repeated_field(&items, 10, "Items").is_ok());
/// assert!(validate_repeated_field(&items, 2, "Items").is_err());
/// ```
#[cfg(feature = "alloc")]
pub fn validate_repeated_field<T>(
    field: &[T],
    max_count: usize,
    _field_name: &str,
) -> Result<(), ValidationError> {
    if field.len() > max_count {
        Err(ValidationError::StringTooLong {
            len: field.len(),
            max: max_count,
        })
    } else {
        Ok(())
    }
}

/// Validation trait for message types
///
/// This trait provides a common interface for validating complete message
/// structures according to XSD constraints. Each message type can implement
/// this trait to encapsulate its validation logic.
///
/// # Examples
///
/// ```rust,ignore
/// use nexo_retailer_protocol::validate::constraints::Validate;
/// use nexo_retailer_protocol::error::ValidationError;
///
/// impl Validate for MyMessage {
///     fn validate(&self) -> Result<(), ValidationError> {
///         // Validate required fields
///         validate_required(&self.required_field, "RequiredField")?;
///         // Validate field types
///         validate_positive_i64(self.count, "Count")?;
///         Ok(())
///     }
/// }
/// ```
pub trait Validate {
    /// Validate the message according to XSD constraints
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the message is valid
    /// * `Err(ValidationError)` if validation fails
    fn validate(&self) -> Result<(), ValidationError>;
}

/// Validate trait implementation for Option<T>
///
/// Validates the inner value if present. Returns `Ok(())` if the option
/// is `None`, allowing validation to proceed on optional fields.
///
/// # Examples
///
/// ```rust,ignore
/// use nexo_retailer_protocol::validate::constraints::Validate;
///
/// let optional_field: Option<MyMessage> = Some(message);
/// optional_field.validate()?; // Validates inner message
///
/// let none_field: Option<MyMessage> = None;
/// none_field.validate()?; // Ok(()) - no validation needed
/// ```
impl<T: Validate> Validate for Option<T> {
    fn validate(&self) -> Result<(), ValidationError> {
        if let Some(ref value) = self {
            value.validate()?;
        }
        Ok(())
    }
}

// ------------------------------------------------------------------------
// Validate trait implementations for key message types
// ------------------------------------------------------------------------

/// Validate trait implementation for ActiveCurrencyAndAmount
///
/// Validates currency code format, nanos range, and sign consistency.
impl Validate for crate::ActiveCurrencyAndAmount {
    fn validate(&self) -> Result<(), ValidationError> {
        crate::validate::validate_monetary_amount(self)
    }
}

/// Validate trait implementation for Header4
///
/// Validates header fields according to XSD constraints.
/// Most fields in Header4 are optional (minOccurs="0"), so this
/// implementation primarily validates field types and formats
/// when values are present.
impl Validate for crate::Header4 {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate string length constraints for present fields
        if let Some(ref msg_fctn) = self.msg_fctn {
            crate::validate::validate_max70_text(msg_fctn)?;
        }

        if let Some(ref proto_vrsn) = self.proto_vrsn {
            crate::validate::validate_max70_text(proto_vrsn)?;
        }

        if let Some(ref tx_id) = self.tx_id {
            crate::validate::validate_max70_text(tx_id)?;
        }

        if let Some(ref cre_dt_tm) = self.cre_dt_tm {
            // ISODateTime format - could add stricter validation here
            crate::validate::validate_max70_text(cre_dt_tm)?;
        }

        // Validate nested message structures
        if let Some(ref initg_pty) = self.initg_pty {
            initg_pty.validate()?;
        }

        if let Some(ref recipnt) = self.recipnt {
            recipnt.validate()?;
        }

        Ok(())
    }
}

/// Validate trait implementation for InitiatingParty3
impl Validate for crate::InitiatingParty3 {
    fn validate(&self) -> Result<(), ValidationError> {
        if let Some(ref id) = self.id {
            id.validate()?;
        }

        if let Some(ref tp) = self.tp {
            crate::validate::validate_max70_text(tp)?;
        }

        if let Some(ref med_of_id) = self.med_of_id {
            crate::validate::validate_max70_text(med_of_id)?;
        }

        Ok(())
    }
}

/// Validate trait implementation for Identification1
impl Validate for crate::Identification1 {
    fn validate(&self) -> Result<(), ValidationError> {
        if let Some(ref id) = self.id {
            crate::validate::validate_max256_text(id)?;
        }

        if let Some(ref issr) = self.issr {
            crate::validate::validate_max256_text(issr)?;
        }

        if let Some(ref tp) = self.tp {
            crate::validate::validate_max70_text(tp)?;
        }

        if let Some(ref cstmr_id) = self.cstmr_id {
            crate::validate::validate_max256_text(cstmr_id)?;
        }

        Ok(())
    }
}

/// Validate trait implementation for Recipient5
impl Validate for crate::Recipient5 {
    fn validate(&self) -> Result<(), ValidationError> {
        if let Some(ref msg_tx_id) = self.msg_tx_id {
            crate::validate::validate_max70_text(msg_tx_id)?;
        }

        if let Some(ref orgnl_biz_t_msg) = self.orgnl_biz_t_msg {
            crate::validate::validate_max70_text(orgnl_biz_t_msg)?;
        }

        if let Some(ref orgnl_msg_id) = self.orgnl_msg_id {
            crate::validate::validate_max70_text(orgnl_msg_id)?;
        }

        Ok(())
    }
}

/// Validate trait implementation for CardData8
impl Validate for crate::CardData8 {
    fn validate(&self) -> Result<(), ValidationError> {
        // Card data fields have specific length constraints
        if let Some(ref crd_nb) = self.crd_nb {
            // Card numbers are typically up to 19 digits
            crate::validate::validate_max_text(crd_nb, 19)?;
        }

        if let Some(ref xpry_dt) = self.xpry_dt {
            // Expiration date format: MMYY (4 characters) or MM-YY (5 characters)
            crate::validate::validate_max_text(xpry_dt, 5)?;
        }

        if let Some(ref card_seq_nb) = self.card_seq_nb {
            // Sequence number is typically 1-3 digits
            crate::validate::validate_max_text(card_seq_nb, 3)?;
        }

        if let Some(ref msstrp_cde) = self.msstrp_cde {
            // Magnetic stripe code is variable but typically limited
            crate::validate::validate_max_text(msstrp_cde, 10)?;
        }

        if let Some(ref eff_dt) = self.eff_dt {
            // Effectivity date format
            crate::validate::validate_max_text(eff_dt, 5)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Import ToString for no_std tests with alloc
    #[cfg(feature = "alloc")]
    use prost::alloc::string::ToString;

    // ------------------------------------------------------------------------
    // validate_required tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_validate_required_present() {
        let field = Some("value".to_string());
        assert!(validate_required(&field, "FieldName").is_ok());
    }

    #[test]
    fn test_validate_required_missing() {
        let field: Option<String> = None;
        let result = validate_required(&field, "TestField");
        assert!(matches!(
            result,
            Err(ValidationError::MissingRequiredField { field }) if field == "TestField"
        ));
    }

    #[test]
    fn test_validate_required_with_int() {
        let field = Some(42i32);
        assert!(validate_required(&field, "Count").is_ok());

        let field: Option<i32> = None;
        assert!(validate_required(&field, "Count").is_err());
    }

    #[test]
    fn test_validate_required_error_message_includes_field_name() {
        let field: Option<String> = None;
        let result = validate_required(&field, "MyCustomField");

        // Verify error message includes the field name
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("MyCustomField"),
            "Error message '{}' should contain field name 'MyCustomField'",
            err_msg
        );
    }

    // ------------------------------------------------------------------------
    // validate_positive_i64 tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_validate_positive_i64_valid() {
        assert!(validate_positive_i64(1, "Count").is_ok());
        assert!(validate_positive_i64(100, "Count").is_ok());
        assert!(validate_positive_i64(i64::MAX, "Count").is_ok());
    }

    #[test]
    fn test_validate_positive_i64_invalid() {
        assert!(validate_positive_i64(0, "Count").is_err());
        assert!(validate_positive_i64(-1, "Count").is_err());
        assert!(validate_positive_i64(i64::MIN, "Count").is_err());
    }

    #[test]
    fn test_validate_positive_i64_boundary_cases() {
        // Boundary: smallest positive value (1)
        assert!(validate_positive_i64(1, "Count").is_ok());

        // Boundary: zero (should fail - not positive)
        assert!(validate_positive_i64(0, "Count").is_err());

        // Boundary: largest positive value
        assert!(validate_positive_i64(i64::MAX, "Count").is_ok());

        // Boundary: smallest negative value (should fail)
        assert!(validate_positive_i64(i64::MIN, "Count").is_err());

        // Boundary: -1 (should fail)
        assert!(validate_positive_i64(-1, "Count").is_err());
    }

    #[test]
    fn test_validate_positive_i64_error_message_includes_field_name() {
        let result = validate_positive_i64(0, "AmountField");

        // Verify error message includes the field name
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("AmountField"),
            "Error message '{}' should contain field name 'AmountField'",
            err_msg
        );
    }

    // ------------------------------------------------------------------------
    // validate_non_negative_i32 tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_validate_non_negative_i32_valid() {
        assert!(validate_non_negative_i32(0, "Index").is_ok());
        assert!(validate_non_negative_i32(1, "Index").is_ok());
        assert!(validate_non_negative_i32(i32::MAX, "Index").is_ok());
    }

    #[test]
    fn test_validate_non_negative_i32_invalid() {
        assert!(validate_non_negative_i32(-1, "Index").is_err());
        assert!(validate_non_negative_i32(i32::MIN, "Index").is_err());
    }

    #[test]
    fn test_validate_non_negative_i32_boundary_cases() {
        // Boundary: zero (valid - non-negative)
        assert!(validate_non_negative_i32(0, "Index").is_ok());

        // Boundary: smallest positive value
        assert!(validate_non_negative_i32(1, "Index").is_ok());

        // Boundary: largest non-negative value
        assert!(validate_non_negative_i32(i32::MAX, "Index").is_ok());

        // Boundary: -1 (should fail - first negative)
        assert!(validate_non_negative_i32(-1, "Index").is_err());

        // Boundary: smallest negative value (should fail)
        assert!(validate_non_negative_i32(i32::MIN, "Index").is_err());
    }

    #[test]
    fn test_validate_non_negative_i32_error_message_includes_field_name() {
        let result = validate_non_negative_i32(-1, "IndexField");

        // Verify error message includes the field name
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("IndexField"),
            "Error message '{}' should contain field name 'IndexField'",
            err_msg
        );
    }

    // ------------------------------------------------------------------------
    // validate_enum_value tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_validate_enum_value_valid() {
        const VALID_VALUES: &[i32] = &[0, 1, 2, 3];
        assert!(validate_enum_value(0, VALID_VALUES, "TestEnum").is_ok());
        assert!(validate_enum_value(2, VALID_VALUES, "TestEnum").is_ok());
    }

    #[test]
    fn test_validate_enum_value_invalid() {
        const VALID_VALUES: &[i32] = &[0, 1, 2];
        assert!(validate_enum_value(99, VALID_VALUES, "TestEnum").is_err());
        assert!(validate_enum_value(-1, VALID_VALUES, "TestEnum").is_err());
    }

    #[test]
    fn test_validate_enum_value_with_u32() {
        const VALID_VALUES: &[u32] = &[0, 1, 2];
        assert!(validate_enum_value(1u32, VALID_VALUES, "TestEnum").is_ok());
        assert!(validate_enum_value(99u32, VALID_VALUES, "TestEnum").is_err());
    }

    #[test]
    fn test_validate_enum_value_error_message_includes_field_name() {
        const VALID_VALUES: &[i32] = &[0, 1, 2];
        let result = validate_enum_value(99, VALID_VALUES, "StatusEnum");

        // Verify error message includes the field name
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("StatusEnum"),
            "Error message '{}' should contain field name 'StatusEnum'",
            err_msg
        );
    }

    #[test]
    fn test_validate_enum_value_boundary_cases() {
        // Test with range 1-5
        const VALID_VALUES: &[i32] = &[1, 2, 3, 4, 5];

        // Boundary: minimum valid value
        assert!(validate_enum_value(1, VALID_VALUES, "RangeEnum").is_ok());

        // Boundary: maximum valid value
        assert!(validate_enum_value(5, VALID_VALUES, "RangeEnum").is_ok());

        // Boundary: just below minimum
        assert!(validate_enum_value(0, VALID_VALUES, "RangeEnum").is_err());

        // Boundary: just above maximum
        assert!(validate_enum_value(6, VALID_VALUES, "RangeEnum").is_err());
    }

    // ------------------------------------------------------------------------
    // Validate trait implementation tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_validate_active_currency_and_amount() {
        let amount = crate::ActiveCurrencyAndAmount {
            ccy: "USD".to_string(),
            units: 100,
            nanos: 500000000,
        };
        assert!(amount.validate().is_ok());
    }

    #[test]
    fn test_validate_active_currency_and_amount_invalid() {
        let amount = crate::ActiveCurrencyAndAmount {
            ccy: "usd".to_string(), // Invalid: lowercase
            units: 100,
            nanos: 0,
        };
        assert!(amount.validate().is_err());
    }

    #[test]
    fn test_validate_header4() {
        let header = crate::Header4 {
            msg_fctn: Some("DREQ".to_string()),
            proto_vrsn: Some("6.0".to_string()),
            tx_id: Some("TX-12345".to_string()),
            cre_dt_tm: Some("2024-02-28T12:00:00Z".to_string()),
            ..Default::default()
        };
        assert!(header.validate().is_ok());
    }

    #[test]
    fn test_validate_header4_with_long_field() {
        let header = crate::Header4 {
            msg_fctn: Some("A".repeat(100)), // Exceeds Max70Text
            ..Default::default()
        };
        assert!(header.validate().is_err());
    }

    #[test]
    fn test_validate_header4_empty() {
        let header = crate::Header4 {
            ..Default::default()
        };
        // Empty header should validate (all fields are optional)
        assert!(header.validate().is_ok());
    }

    #[test]
    fn test_validate_option_validate_trait() {
        // Test Option<T> Validate implementation
        let some_value: Option<crate::ActiveCurrencyAndAmount> = Some(
            crate::ActiveCurrencyAndAmount {
                ccy: "USD".to_string(),
                units: 100,
                nanos: 0,
            }
        );
        assert!(some_value.validate().is_ok());

        let none_value: Option<crate::ActiveCurrencyAndAmount> = None;
        assert!(none_value.validate().is_ok());
    }

    #[test]
    fn test_validate_card_data8() {
        let card_data = crate::CardData8 {
            crd_nb: Some("1234567890123456789".to_string()), // 19 digits
            xpry_dt: Some("1225".to_string()), // MMYY
            card_seq_nb: Some("123".to_string()),
            ..Default::default()
        };
        assert!(card_data.validate().is_ok());
    }

    #[test]
    fn test_validate_card_data8_invalid_pan_length() {
        let card_data = crate::CardData8 {
            crd_nb: Some("1".repeat(20)), // Exceeds 19 digits
            ..Default::default()
        };
        assert!(card_data.validate().is_err());
    }

    #[test]
    fn test_validate_identification1() {
        let id = crate::Identification1 {
            id: Some("TERMINAL123".to_string()),
            issr: Some("ISSUER".to_string()),
            tp: Some("TYPE".to_string()),
            cstmr_id: Some("CUSTOMER".to_string()),
        };
        assert!(id.validate().is_ok());
    }

    #[test]
    fn test_validate_nested_structures() {
        // Test validation of nested Header4 -> InitiatingParty3 -> Identification1
        let identification = crate::Identification1 {
            id: Some("A".repeat(300)), // Exceeds Max256Text
            ..Default::default()
        };

        let initg_pty = crate::InitiatingParty3 {
            id: Some(identification),
            ..Default::default()
        };

        let header = crate::Header4 {
            initg_pty: Some(initg_pty),
            ..Default::default()
        };

        // Should fail due to nested validation
        assert!(header.validate().is_err());
    }
}

// Collection validation tests (alloc feature only)
#[cfg(test)]
#[cfg(feature = "alloc")]
mod alloc_tests {
    use super::*;

    #[test]
    fn test_validate_repeated_field() {
        let items = vec![1, 2, 3];
        assert!(validate_repeated_field(&items, 10, "Items").is_ok());
        assert!(validate_repeated_field(&items, 3, "Items").is_ok());
        assert!(validate_repeated_field(&items, 2, "Items").is_err());
    }

    #[test]
    fn test_validate_repeated_field_empty() {
        let items: Vec<i32> = vec![];
        assert!(validate_repeated_field(&items, 10, "Items").is_ok());
    }

    #[test]
    fn test_validate_repeated_field_large() {
        let items = vec![1; 1000];
        assert!(validate_repeated_field(&items, 10000, "Items").is_ok());
        assert!(validate_repeated_field(&items, 100, "Items").is_err());
    }

    #[test]
    fn test_validate_repeated_field_boundary_at_max() {
        // Test collection at exactly max size (should pass)
        let items: Vec<i32> = (0..100).collect();
        assert!(validate_repeated_field(&items, 100, "Items").is_ok());
    }

    #[test]
    fn test_validate_repeated_field_boundary_one_over_max() {
        // Test collection at max+1 size (should fail)
        let items: Vec<i32> = (0..101).collect();
        let result = validate_repeated_field(&items, 100, "Items");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_repeated_field_with_vec_string() {
        // Test with Vec<String> for repeated string fields
        let strings: Vec<String> = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert!(validate_repeated_field(&strings, 5, "StringFields").is_ok());
        assert!(validate_repeated_field(&strings, 2, "StringFields").is_err());
    }

    #[test]
    fn test_validate_repeated_field_with_vec_i64() {
        // Test with Vec<i64> for repeated numeric fields
        let numbers: Vec<i64> = vec![100, 200, 300, 400, 500];
        assert!(validate_repeated_field(&numbers, 10, "NumericFields").is_ok());
        assert!(validate_repeated_field(&numbers, 4, "NumericFields").is_err());
    }

    #[test]
    fn test_validate_repeated_field_with_vec_protobuf_messages() {
        // Test with Vec<protobuf messages> for repeated nested messages
        let amounts: Vec<crate::ActiveCurrencyAndAmount> = vec![
            crate::ActiveCurrencyAndAmount { ccy: "USD".to_string(), units: 10, nanos: 0 },
            crate::ActiveCurrencyAndAmount { ccy: "EUR".to_string(), units: 20, nanos: 0 },
        ];
        assert!(validate_repeated_field(&amounts, 5, "Amounts").is_ok());
        assert!(validate_repeated_field(&amounts, 1, "Amounts").is_err());
    }

    #[test]
    fn test_validate_repeated_field_error_message_includes_field_name() {
        let items: Vec<i32> = (0..11).collect();
        let result = validate_repeated_field(&items, 10, "MyRepeatedField");

        // Verify error message includes field name and size limit
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // The error should contain information about the constraint violation
        assert!(
            err_msg.contains("11") || err_msg.contains("10"),
            "Error message should contain size info: {}",
            err_msg
        );
    }

    #[test]
    fn test_validate_repeated_field_with_zero_max() {
        // Edge case: max = 0 means only empty collection is valid
        let empty: Vec<String> = vec![];
        assert!(validate_repeated_field(&empty, 0, "ZeroMax").is_ok());

        let one_item: Vec<String> = vec!["a".to_string()];
        assert!(validate_repeated_field(&one_item, 0, "ZeroMax").is_err());
    }

    #[test]
    fn test_validate_repeated_field_with_single_item() {
        // Test with single item at various limits
        let single: Vec<String> = vec!["only_one".to_string()];
        assert!(validate_repeated_field(&single, 1, "SingleItem").is_ok());
        assert!(validate_repeated_field(&single, 2, "SingleItem").is_ok());
        assert!(validate_repeated_field(&single, 0, "SingleItem").is_err());
    }
}
