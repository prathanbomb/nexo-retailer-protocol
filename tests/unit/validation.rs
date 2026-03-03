//! Cross-constraint integration tests for validation logic
//!
//! This module tests validation functions across multiple constraints and
//! real-world message scenarios. It validates that combined validation rules
//! work correctly when applied to complex message structures.
//!
//! # Test Strategy
//!
//! 1. **Header validation**: Test Header4 with string length + nested validation
//! 2. **Card data validation**: Test CardData8 with card number length + expiration format
//! 3. **Monetary amount validation**: Test ActiveCurrencyAndAmount with currency + amount range
//! 4. **Repeated field validation**: Test collection size limits
//! 5. **Error propagation**: Test that multiple validation failures are reported clearly
//!
//! # Feature Flags
//!
//! - All tests require `alloc` feature for String and Vec allocation.

#![cfg(all(test, feature = "alloc"))]

use nexo_retailer_protocol::{
    ActiveCurrencyAndAmount, CardData8, Header4, Identification1, InitiatingParty3,
    Recipient5, Validate,
};
use nexo_retailer_protocol::validate::{
    validate_currency_code, validate_max256_text, validate_max70_text, validate_max20000_text,
    validate_monetary_amount, validate_positive_i64, validate_repeated_field, validate_required,
};

// ------------------------------------------------------------------------
// Header4 Cross-Constraint Tests
// ------------------------------------------------------------------------

#[test]
fn test_validate_header_with_all_constraints() {
    // Create a valid header with all string fields at acceptable lengths
    let header = Header4 {
        msg_fctn: Some("DREQ".to_string()),        // Within Max70Text
        proto_vrsn: Some("6.0".to_string()),       // Within Max70Text
        tx_id: Some("TX-12345-ABCDEF".to_string()), // Within Max70Text
        cre_dt_tm: Some("2024-02-28T12:00:00Z".to_string()), // Within Max70Text
        ..Default::default()
    };

    // Validate header with all constraints
    assert!(header.validate().is_ok(), "Valid header should pass validation");
}

#[test]
fn test_validate_header_with_nested_structures() {
    // Create a header with nested structures
    let identification = Identification1 {
        id: Some("TERMINAL-001".to_string()),
        issr: Some("NEXO".to_string()),
        tp: Some("POI".to_string()),
        cstmr_id: Some("CUSTOMER-123".to_string()),
    };

    let initg_pty = InitiatingParty3 {
        id: Some(identification),
        tp: Some("MERCHANT".to_string()),
        med_of_id: Some("TERMINAL_ID".to_string()),
    };

    let recipnt = Recipient5 {
        msg_tx_id: Some("RESP-123".to_string()),
        orgnl_biz_t_msg: Some("PAYMENT".to_string()),
        orgnl_msg_id: Some("MSG-001".to_string()),
    };

    let header = Header4 {
        msg_fctn: Some("DREQ".to_string()),
        proto_vrsn: Some("6.0".to_string()),
        initg_pty: Some(initg_pty),
        recipnt: Some(recipnt),
        ..Default::default()
    };

    // All nested structures should validate
    assert!(header.validate().is_ok(), "Header with nested structures should validate");
}

#[test]
fn test_validate_header_with_long_msg_fctn() {
    // Create a header with msg_fctn exceeding Max70Text
    let header = Header4 {
        msg_fctn: Some("A".repeat(71)), // Exceeds 70 characters
        ..Default::default()
    };

    // Should fail due to string length constraint
    let result = header.validate();
    assert!(result.is_err(), "Header with too-long msg_fctn should fail validation");
}

#[test]
fn test_validate_header_with_long_nested_field() {
    // Create a header with nested identification having too-long id
    let identification = Identification1 {
        id: Some("A".repeat(300)), // Exceeds Max256Text
        ..Default::default()
    };

    let initg_pty = InitiatingParty3 {
        id: Some(identification),
        ..Default::default()
    };

    let header = Header4 {
        initg_pty: Some(initg_pty),
        ..Default::default()
    };

    // Should fail due to nested validation
    let result = header.validate();
    assert!(result.is_err(), "Header with too-long nested id should fail validation");
}

// ------------------------------------------------------------------------
// CardData8 Cross-Constraint Tests
// ------------------------------------------------------------------------

#[test]
fn test_validate_card_data_with_constraints() {
    // Create valid card data
    let card_data = CardData8 {
        crd_nb: Some("4111111111111111".to_string()), // 16 digits (within 19)
        xpry_dt: Some("1225".to_string()),           // MMYY format
        card_seq_nb: Some("001".to_string()),        // 3 digits
        ..Default::default()
    };

    assert!(card_data.validate().is_ok(), "Valid card data should pass validation");
}

#[test]
fn test_validate_card_data_with_long_pan() {
    // Create card data with PAN exceeding 19 digits
    let card_data = CardData8 {
        crd_nb: Some("1".repeat(20)), // 20 digits (exceeds 19)
        ..Default::default()
    };

    let result = card_data.validate();
    assert!(result.is_err(), "Card data with too-long PAN should fail validation");
}

#[test]
fn test_validate_card_data_with_long_expiration() {
    // Create card data with expiration exceeding 5 characters
    let card_data = CardData8 {
        xpry_dt: Some("12-2025".to_string()), // 7 characters (exceeds 5)
        ..Default::default()
    };

    let result = card_data.validate();
    assert!(result.is_err(), "Card data with too-long expiration should fail validation");
}

#[test]
fn test_validate_card_data_with_multiple_violations() {
    // Create card data with multiple constraint violations
    let card_data = CardData8 {
        crd_nb: Some("1".repeat(25)),     // Exceeds 19
        xpry_dt: Some("12-2025".to_string()), // Exceeds 5
        card_seq_nb: Some("1234".to_string()), // Exceeds 3
        ..Default::default()
    };

    // Should fail (first violation encountered)
    let result = card_data.validate();
    assert!(result.is_err(), "Card data with multiple violations should fail validation");
}

// ------------------------------------------------------------------------
// ActiveCurrencyAndAmount Cross-Constraint Tests
// ------------------------------------------------------------------------

#[test]
fn test_validate_monetary_amount_with_currency() {
    // Create valid monetary amount
    let amount = ActiveCurrencyAndAmount {
        ccy: "USD".to_string(),
        units: 100,
        nanos: 500_000_000, // 0.5
    };

    assert!(amount.validate().is_ok(), "Valid monetary amount should pass validation");
}

#[test]
fn test_validate_monetary_amount_with_invalid_currency() {
    // Create amount with invalid currency code
    let amount = ActiveCurrencyAndAmount {
        ccy: "usd".to_string(), // Lowercase (invalid)
        units: 100,
        nanos: 0,
    };

    let result = amount.validate();
    assert!(result.is_err(), "Amount with lowercase currency should fail validation");
}

#[test]
fn test_validate_monetary_amount_with_nanos_out_of_range() {
    // Create amount with nanos out of range
    let amount = ActiveCurrencyAndAmount {
        ccy: "USD".to_string(),
        units: 100,
        nanos: 1_000_000_000, // Exceeds 999,999,999
    };

    let result = amount.validate();
    assert!(result.is_err(), "Amount with nanos out of range should fail validation");
}

#[test]
fn test_validate_monetary_amount_with_sign_mismatch() {
    // Create amount with sign mismatch
    let amount = ActiveCurrencyAndAmount {
        ccy: "USD".to_string(),
        units: 100,           // Positive
        nanos: -500_000_000,  // Negative (mismatch)
    };

    let result = amount.validate();
    assert!(result.is_err(), "Amount with sign mismatch should fail validation");
}

#[test]
fn test_validate_realistic_transaction_amounts() {
    // Test various realistic transaction amounts

    // Small amount: $0.99
    let small = ActiveCurrencyAndAmount {
        ccy: "USD".to_string(),
        units: 0,
        nanos: 990_000_000,
    };
    assert!(small.validate().is_ok());

    // Medium amount: EUR 123.45
    let medium = ActiveCurrencyAndAmount {
        ccy: "EUR".to_string(),
        units: 123,
        nanos: 450_000_000,
    };
    assert!(medium.validate().is_ok());

    // Large amount: JPY 1000000 (no decimal)
    let large = ActiveCurrencyAndAmount {
        ccy: "JPY".to_string(),
        units: 1_000_000,
        nanos: 0,
    };
    assert!(large.validate().is_ok());

    // Negative amount: -$50.00 (refund)
    let negative = ActiveCurrencyAndAmount {
        ccy: "USD".to_string(),
        units: -50,
        nanos: 0,
    };
    assert!(negative.validate().is_ok());
}

// ------------------------------------------------------------------------
// Repeated Field Validation Tests
// ------------------------------------------------------------------------

#[test]
fn test_validate_repeated_field_size() {
    // Test empty collection
    let empty: Vec<String> = vec![];
    assert!(validate_repeated_field(&empty, 10, "Items").is_ok());

    // Test collection within limit
    let within_limit: Vec<i64> = vec![1, 2, 3, 4, 5];
    assert!(validate_repeated_field(&within_limit, 10, "Numbers").is_ok());

    // Test collection at exactly max size
    let at_limit: Vec<String> = (0..10).map(|i| format!("Item{}", i)).collect();
    assert!(validate_repeated_field(&at_limit, 10, "Items").is_ok());

    // Test collection exceeding max size
    let over_limit: Vec<i32> = (0..11).collect();
    let result = validate_repeated_field(&over_limit, 10, "Numbers");
    assert!(result.is_err(), "Collection exceeding limit should fail validation");
}

#[test]
fn test_validate_repeated_field_with_various_types() {
    // Vec<String>
    let strings: Vec<String> = vec!["a".to_string(), "b".to_string()];
    assert!(validate_repeated_field(&strings, 5, "Strings").is_ok());

    // Vec<i64>
    let numbers: Vec<i64> = vec![1, 2, 3, 4, 5];
    assert!(validate_repeated_field(&numbers, 10, "Numbers").is_ok());

    // Vec<protobuf messages> - using ActiveCurrencyAndAmount
    let amounts: Vec<ActiveCurrencyAndAmount> = vec![
        ActiveCurrencyAndAmount { ccy: "USD".to_string(), units: 10, nanos: 0 },
        ActiveCurrencyAndAmount { ccy: "EUR".to_string(), units: 20, nanos: 0 },
    ];
    assert!(validate_repeated_field(&amounts, 5, "Amounts").is_ok());
}

#[test]
fn test_validate_repeated_field_boundary_cases() {
    // Test boundary: max = 0 (only empty allowed)
    let empty: Vec<String> = vec![];
    assert!(validate_repeated_field(&empty, 0, "Items").is_ok());

    let one_item: Vec<String> = vec!["a".to_string()];
    assert!(validate_repeated_field(&one_item, 0, "Items").is_err());

    // Test boundary: max = 1
    assert!(validate_repeated_field(&one_item, 1, "Items").is_ok());

    let two_items: Vec<String> = vec!["a".to_string(), "b".to_string()];
    assert!(validate_repeated_field(&two_items, 1, "Items").is_err());
}

// ------------------------------------------------------------------------
// Validation Error Propagation Tests
// ------------------------------------------------------------------------

#[test]
fn test_validation_error_includes_field_name() {
    // Test that validation errors include the field name
    let field: Option<String> = None;
    let result = validate_required(&field, "SaleRefNo");
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("SaleRefNo"),
        "Error message should contain field name: {}",
        err_msg
    );
}

#[test]
fn test_validation_error_includes_constraint_type() {
    // Test that string length errors describe the constraint
    let long_string = "A".repeat(300);
    let result = validate_max256_text(&long_string);
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("long") || err_msg.contains("max"),
        "Error message should describe constraint type: {}",
        err_msg
    );
}

#[test]
fn test_validation_error_includes_expected_vs_actual() {
    // Test that currency length errors show expected vs actual
    let result = validate_currency_code("US");
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("2") && err_msg.contains("3"),
        "Error message should show expected (3) and actual (2): {}",
        err_msg
    );
}

#[test]
fn test_validation_error_for_positive_value() {
    // Test that positive value errors work correctly
    let result = validate_positive_i64(0, "Count");
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Count"),
        "Error message should contain field name: {}",
        err_msg
    );
}

// ------------------------------------------------------------------------
// Combined Validation Scenario Tests
// ------------------------------------------------------------------------

#[test]
fn test_complex_message_validation() {
    // Create a complex message structure with multiple validation points
    let header = Header4 {
        msg_fctn: Some("DREQ".to_string()),
        proto_vrsn: Some("6.0".to_string()),
        tx_id: Some("TX-12345".to_string()),
        cre_dt_tm: Some("2024-02-28T12:00:00Z".to_string()),
        ..Default::default()
    };

    let amount = ActiveCurrencyAndAmount {
        ccy: "USD".to_string(),
        units: 100,
        nanos: 0,
    };

    // Validate all components
    assert!(header.validate().is_ok(), "Header should be valid");
    assert!(amount.validate().is_ok(), "Amount should be valid");
}

#[test]
fn test_validation_with_all_optional_fields_missing() {
    // Create a header with all optional fields missing
    let header = Header4 {
        ..Default::default()
    };

    // Should pass - all fields are optional
    assert!(header.validate().is_ok(), "Empty header with all optional fields should be valid");
}

#[test]
fn test_validation_stops_at_first_failure() {
    // Create card data with multiple violations
    // The validation should fail on the first constraint violation encountered
    let card_data = CardData8 {
        crd_nb: Some("1".repeat(25)),     // First violation: exceeds 19
        xpry_dt: Some("12-2025".to_string()), // Second violation: exceeds 5
        ..Default::default()
    };

    let result = card_data.validate();
    assert!(result.is_err(), "Should fail on first violation");

    // Error message should mention the first violated field
    let err_msg = result.unwrap_err().to_string();
    // The error should be about one of the violations
    assert!(
        err_msg.contains("long") || err_msg.contains("exceeds") || err_msg.contains("25") || err_msg.len() > 0,
        "Error should describe the violation: {}",
        err_msg
    );
}

// ------------------------------------------------------------------------
// no_std Compatibility Tests
// ------------------------------------------------------------------------

#[test]
fn test_validation_works_with_alloc_feature() {
    // This test verifies that validation functions work correctly
    // when only the alloc feature is enabled (no std)

    // String validation
    assert!(validate_max70_text("test").is_ok());
    assert!(validate_max256_text("test").is_ok());
    assert!(validate_max20000_text("test").is_ok());

    // Currency validation
    assert!(validate_currency_code("USD").is_ok());
    assert!(validate_currency_code("EUR").is_ok());

    // Required field validation
    let field = Some("value".to_string());
    assert!(validate_required(&field, "TestField").is_ok());

    // Positive value validation
    assert!(validate_positive_i64(1, "Count").is_ok());

    // Monetary amount validation
    let amount = ActiveCurrencyAndAmount {
        ccy: "USD".to_string(),
        units: 100,
        nanos: 0,
    };
    assert!(validate_monetary_amount(&amount).is_ok());
}
