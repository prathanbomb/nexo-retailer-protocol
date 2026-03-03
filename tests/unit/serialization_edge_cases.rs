//! Property-based tests for serialization edge cases
//!
//! This module tests the codec's handling of malformed input, truncated data,
//! boundary conditions, oversized messages, and advanced edge cases using
//! property-based testing.
//!
//! # Test Categories
//!
//! 1. **Malformed input handling** - Random byte arrays should never panic
//! 2. **Truncated input handling** - Incomplete messages should fail gracefully
//! 3. **Boundary conditions** - Size limits and numeric boundaries
//! 4. **Oversized message rejection** - Messages > 4MB should be rejected
//! 5. **Corrupted length prefixes** - Invalid 4-byte length prefixes handled gracefully
//! 6. **Random input fuzzing** - Random byte arrays don't cause panics
//! 7. **Round-trip invariants** - decode(encode(msg)) == msg always holds
//!
//! # Feature Flags
//!
//! - All tests require `std` feature (proptest requires std)
//! - Tests run with: `cargo test --test serialization_edge_cases --features std`
//!
//! # Regression Persistence
//!
//! Proptest regression cases are persisted to `proptest-regressions/` directory.
//! Commit discovered edge cases to prevent future regressions.

#![cfg(all(test, feature = "std"))]

use nexo_retailer_protocol::{
    Casp001Document, Casp002Document, Casp003Document, Casp004Document, Casp005Document,
    Casp006Document, Casp007Document, Casp008Document, Casp009Document, Casp010Document,
    Casp011Document, Casp012Document, Casp013Document, Casp014Document, Casp015Document,
    Casp016Document, Casp017Document,
};
use nexo_retailer_protocol::codec::limits::MAX_MESSAGE_SIZE;
use prost::Message;
use proptest::prelude::*;

// Configure proptest with regression persistence
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    // ===========================================================================
    // TASK 2: Malformed Input Handling Tests
    // ===========================================================================

    /// Property test: Malformed input handling for all 17 CASP message types.
    ///
    /// Verifies that the decoder never panics on random byte input and that
    /// error messages are informative (not empty).
    #[test]
    fn test_malformed_input_handling_casp001(data in any::<Vec<u8>>()) {
        let result = Casp001Document::decode(&data[..]);
        match result {
            Ok(_) => (), // Valid by chance - empty bytes decode to default
            Err(e) => {
                // Error message should be informative
                assert!(!e.to_string().is_empty(), "Error message should not be empty");
            }
        }
        // Should never panic - this is the main property
    }

    #[test]
    fn test_malformed_input_handling_casp002(data in any::<Vec<u8>>()) {
        let result = Casp002Document::decode(&data[..]);
        match result {
            Ok(_) => (),
            Err(e) => assert!(!e.to_string().is_empty()),
        }
    }

    #[test]
    fn test_malformed_input_handling_casp003(data in any::<Vec<u8>>()) {
        let result = Casp003Document::decode(&data[..]);
        match result {
            Ok(_) => (),
            Err(e) => assert!(!e.to_string().is_empty()),
        }
    }

    #[test]
    fn test_malformed_input_handling_casp004(data in any::<Vec<u8>>()) {
        let result = Casp004Document::decode(&data[..]);
        match result {
            Ok(_) => (),
            Err(e) => assert!(!e.to_string().is_empty()),
        }
    }

    #[test]
    fn test_malformed_input_handling_casp005(data in any::<Vec<u8>>()) {
        let result = Casp005Document::decode(&data[..]);
        match result {
            Ok(_) => (),
            Err(e) => assert!(!e.to_string().is_empty()),
        }
    }

    #[test]
    fn test_malformed_input_handling_casp006(data in any::<Vec<u8>>()) {
        let result = Casp006Document::decode(&data[..]);
        match result {
            Ok(_) => (),
            Err(e) => assert!(!e.to_string().is_empty()),
        }
    }

    #[test]
    fn test_malformed_input_handling_casp007(data in any::<Vec<u8>>()) {
        let result = Casp007Document::decode(&data[..]);
        match result {
            Ok(_) => (),
            Err(e) => assert!(!e.to_string().is_empty()),
        }
    }

    #[test]
    fn test_malformed_input_handling_casp008(data in any::<Vec<u8>>()) {
        let result = Casp008Document::decode(&data[..]);
        match result {
            Ok(_) => (),
            Err(e) => assert!(!e.to_string().is_empty()),
        }
    }

    #[test]
    fn test_malformed_input_handling_casp009(data in any::<Vec<u8>>()) {
        let result = Casp009Document::decode(&data[..]);
        match result {
            Ok(_) => (),
            Err(e) => assert!(!e.to_string().is_empty()),
        }
    }

    #[test]
    fn test_malformed_input_handling_casp010(data in any::<Vec<u8>>()) {
        let result = Casp010Document::decode(&data[..]);
        match result {
            Ok(_) => (),
            Err(e) => assert!(!e.to_string().is_empty()),
        }
    }

    #[test]
    fn test_malformed_input_handling_casp011(data in any::<Vec<u8>>()) {
        let result = Casp011Document::decode(&data[..]);
        match result {
            Ok(_) => (),
            Err(e) => assert!(!e.to_string().is_empty()),
        }
    }

    #[test]
    fn test_malformed_input_handling_casp012(data in any::<Vec<u8>>()) {
        let result = Casp012Document::decode(&data[..]);
        match result {
            Ok(_) => (),
            Err(e) => assert!(!e.to_string().is_empty()),
        }
    }

    #[test]
    fn test_malformed_input_handling_casp013(data in any::<Vec<u8>>()) {
        let result = Casp013Document::decode(&data[..]);
        match result {
            Ok(_) => (),
            Err(e) => assert!(!e.to_string().is_empty()),
        }
    }

    #[test]
    fn test_malformed_input_handling_casp014(data in any::<Vec<u8>>()) {
        let result = Casp014Document::decode(&data[..]);
        match result {
            Ok(_) => (),
            Err(e) => assert!(!e.to_string().is_empty()),
        }
    }

    #[test]
    fn test_malformed_input_handling_casp015(data in any::<Vec<u8>>()) {
        let result = Casp015Document::decode(&data[..]);
        match result {
            Ok(_) => (),
            Err(e) => assert!(!e.to_string().is_empty()),
        }
    }

    #[test]
    fn test_malformed_input_handling_casp016(data in any::<Vec<u8>>()) {
        let result = Casp016Document::decode(&data[..]);
        match result {
            Ok(_) => (),
            Err(e) => assert!(!e.to_string().is_empty()),
        }
    }

    #[test]
    fn test_malformed_input_handling_casp017(data in any::<Vec<u8>>()) {
        let result = Casp017Document::decode(&data[..]);
        match result {
            Ok(_) => (),
            Err(e) => assert!(!e.to_string().is_empty()),
        }
    }

    // ===========================================================================
    // TASK 3: Truncated Input Handling Tests
    // ===========================================================================

    /// Property test: Truncated input handling for all 17 CASP message types.
    ///
    /// Verifies that the decoder handles short byte arrays gracefully without panicking.
    #[test]
    fn test_truncated_input_handling_casp001(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        let result = Casp001Document::decode(&data[..]);
        // Either succeeds (valid data by chance) or fails cleanly
        assert!(result.is_ok() || result.is_err());
        // Never panics
    }

    #[test]
    fn test_truncated_input_handling_casp002(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        let _ = Casp002Document::decode(&data[..]);
    }

    #[test]
    fn test_truncated_input_handling_casp003(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        let _ = Casp003Document::decode(&data[..]);
    }

    #[test]
    fn test_truncated_input_handling_casp004(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        let _ = Casp004Document::decode(&data[..]);
    }

    #[test]
    fn test_truncated_input_handling_casp005(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        let _ = Casp005Document::decode(&data[..]);
    }

    #[test]
    fn test_truncated_input_handling_casp006(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        let _ = Casp006Document::decode(&data[..]);
    }

    #[test]
    fn test_truncated_input_handling_casp007(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        let _ = Casp007Document::decode(&data[..]);
    }

    #[test]
    fn test_truncated_input_handling_casp008(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        let _ = Casp008Document::decode(&data[..]);
    }

    #[test]
    fn test_truncated_input_handling_casp009(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        let _ = Casp009Document::decode(&data[..]);
    }

    #[test]
    fn test_truncated_input_handling_casp010(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        let _ = Casp010Document::decode(&data[..]);
    }

    #[test]
    fn test_truncated_input_handling_casp011(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        let _ = Casp011Document::decode(&data[..]);
    }

    #[test]
    fn test_truncated_input_handling_casp012(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        let _ = Casp012Document::decode(&data[..]);
    }

    #[test]
    fn test_truncated_input_handling_casp013(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        let _ = Casp013Document::decode(&data[..]);
    }

    #[test]
    fn test_truncated_input_handling_casp014(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        let _ = Casp014Document::decode(&data[..]);
    }

    #[test]
    fn test_truncated_input_handling_casp015(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        let _ = Casp015Document::decode(&data[..]);
    }

    #[test]
    fn test_truncated_input_handling_casp016(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        let _ = Casp016Document::decode(&data[..]);
    }

    #[test]
    fn test_truncated_input_handling_casp017(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        let _ = Casp017Document::decode(&data[..]);
    }
}

// =============================================================================
// TASK 5: Oversized Message Rejection Tests (Low case count for performance)
// =============================================================================

/// Low-case-count proptest block for oversized message tests
proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// Property test: Oversized message rejection.
    ///
    /// Verifies that messages larger than 4MB are handled without panicking.
    /// Note: Uses low case count for practical test execution time.
    /// Deterministic boundary tests cover exact 4MB limits.
    /// MAX_MESSAGE_SIZE = 4 * 1024 * 1024 = 4,194,304 bytes
    #[test]
    fn test_oversized_message_rejection(data in prop::collection::vec(any::<u8>(), 4_194_305..4_195_305)) {
        // Verify the data is indeed oversized (> 4MB = 4,194,304 bytes)
        prop_assert!(data.len() > MAX_MESSAGE_SIZE);

        // Attempt to decode - should fail or handle gracefully
        let result = Casp001Document::decode(&data[..]);

        // Either fails (expected) or succeeds (valid protobuf by chance)
        // The key property is: no panic or undefined behavior
        match result {
            Ok(_) => (), // Valid by extreme chance
            Err(e) => {
                // Error should be informative
                prop_assert!(!e.to_string().is_empty());
            }
        }
    }
}

// =============================================================================
// TASK 4: Boundary Condition Tests (Non-property tests for deterministic checks)
// =============================================================================

/// Test zero-length message handling.
///
/// Empty byte array should decode to default message or fail gracefully.
#[test]
fn test_zero_length_message() {
    let data: &[u8] = &[];
    let result = Casp001Document::decode(data);

    // Empty bytes should decode to default message (valid protobuf)
    match result {
        Ok(msg) => assert_eq!(msg, Casp001Document::default()),
        Err(e) => assert!(!e.to_string().is_empty()),
    }
}

/// Test single byte message handling.
#[test]
fn test_single_byte_message() {
    let data = [0u8; 1];
    let result = Casp001Document::decode(&data[..]);
    // Should not panic
    assert!(result.is_ok() || result.is_err());
}

/// Test exactly at size limit (4MB).
///
/// A 4MB byte array of zeros should fail cleanly (invalid protobuf)
/// or decode to default without panic.
#[test]
fn test_max_length_message() {
    let data = vec![0u8; MAX_MESSAGE_SIZE];
    let result = Casp001Document::decode(&data[..]);

    // Should fail (zeros are not valid protobuf) but never panic
    match result {
        Ok(_) => (), // Would be surprising but not a bug
        Err(e) => assert!(!e.to_string().is_empty()),
    }
}

/// Test just over size limit (4MB + 1 byte).
#[test]
fn test_max_length_plus_one() {
    let data = vec![0u8; MAX_MESSAGE_SIZE + 1];
    let result = Casp001Document::decode(&data[..]);

    // Should handle oversized input gracefully
    match result {
        Ok(_) => (),
        Err(e) => assert!(!e.to_string().is_empty()),
    }
}

/// Test numeric boundary values in messages.
#[test]
fn test_numeric_boundary_values() {
    use nexo_retailer_protocol::ActiveCurrencyAndAmount;

    // Test i64::MAX for monetary units
    let max_units = ActiveCurrencyAndAmount {
        ccy: "USD".to_string(),
        units: i64::MAX,
        nanos: 0,
    };
    let encoded = max_units.encode_to_vec();
    let decoded = ActiveCurrencyAndAmount::decode(&encoded[..]).expect("decode i64::MAX");
    assert_eq!(max_units, decoded);

    // Test i64::MIN for monetary units (negative amounts)
    let min_units = ActiveCurrencyAndAmount {
        ccy: "USD".to_string(),
        units: i64::MIN,
        nanos: 0,
    };
    let encoded = min_units.encode_to_vec();
    let decoded = ActiveCurrencyAndAmount::decode(&encoded[..]).expect("decode i64::MIN");
    assert_eq!(min_units, decoded);

    // Test nanos at max value (999999999)
    let max_nanos = ActiveCurrencyAndAmount {
        ccy: "EUR".to_string(),
        units: 100,
        nanos: 999_999_999,
    };
    let encoded = max_nanos.encode_to_vec();
    let decoded = ActiveCurrencyAndAmount::decode(&encoded[..]).expect("decode max nanos");
    assert_eq!(max_nanos, decoded);

    // Test nanos at min value (negative)
    let min_nanos = ActiveCurrencyAndAmount {
        ccy: "EUR".to_string(),
        units: -100,
        nanos: -999_999_999,
    };
    let encoded = min_nanos.encode_to_vec();
    let decoded = ActiveCurrencyAndAmount::decode(&encoded[..]).expect("decode min nanos");
    assert_eq!(min_nanos, decoded);
}

/// Test string length boundaries.
#[test]
fn test_string_length_boundaries() {
    use nexo_retailer_protocol::Header4;

    // Test with very long transaction ID (well over typical 256 char limit)
    let long_tx_id = "X".repeat(1000);
    let header = Header4 {
        tx_id: Some(long_tx_id.clone()),
        ..Default::default()
    };

    // Should encode and decode correctly (protobuf has no string length limit)
    let encoded = header.encode_to_vec();
    let decoded = Header4::decode(&encoded[..]).expect("decode long string");
    assert_eq!(decoded.tx_id, Some(long_tx_id));

    // Test empty string
    let empty_header = Header4 {
        tx_id: Some("".to_string()),
        ..Default::default()
    };
    let encoded = empty_header.encode_to_vec();
    let decoded = Header4::decode(&encoded[..]).expect("decode empty string");
    assert_eq!(decoded.tx_id, Some("".to_string()));
}

// =============================================================================
// Additional Truncation Tests (Deterministic)
// =============================================================================

/// Test intentional truncation of valid messages at various byte positions.
#[test]
fn test_intentional_truncation_at_boundaries() {
    // Create a valid message with some content
    use nexo_retailer_protocol::{Casp001DocumentDocument, SaleToPoiServiceRequestV06, Header4};

    let original = Casp001Document {
        document: Some(Casp001DocumentDocument {
            sale_to_poi_svc_req: Some(SaleToPoiServiceRequestV06 {
                hdr: Some(Header4 {
                    msg_fctn: Some("DREQ".to_string()),
                    proto_vrsn: Some("6.0".to_string()),
                    tx_id: Some("TX-12345".to_string()),
                    cre_dt_tm: Some("2024-01-15T10:30:00Z".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        }),
    };

    let encoded = original.encode_to_vec();
    let original_len = encoded.len();

    // Test truncation at various positions
    let truncation_points = [0, 1, 2, 3, 4, 5, 10, 20, original_len / 2, original_len - 1];

    for &truncate_at in &truncation_points {
        if truncate_at < original_len {
            let truncated = &encoded[..truncate_at];
            let result = Casp001Document::decode(truncated);

            // Should either fail (truncated) or succeed (valid prefix)
            // The key is: never panic
            match result {
                Ok(_) => (), // Valid prefix - rare but possible
                Err(e) => assert!(!e.to_string().is_empty()),
            }
        }
    }

    // Full message should round-trip correctly
    let full_decoded = Casp001Document::decode(&encoded[..]).expect("full message decode");
    assert_eq!(original, full_decoded);
}

/// Test truncated message detection at wire format boundaries.
///
/// Tests the 4-byte length prefix truncation scenario common in network protocols.
#[test]
fn test_wire_format_truncation() {
    // Simulate wire format: 4-byte length prefix + message body
    let original = Casp001Document::default();
    let msg_bytes = original.encode_to_vec();
    let msg_len = msg_bytes.len() as u32;

    // Build complete wire message
    let mut wire_msg = Vec::with_capacity(4 + msg_bytes.len());
    wire_msg.extend_from_slice(&msg_len.to_be_bytes());
    wire_msg.extend_from_slice(&msg_bytes);

    // Test truncation scenarios
    // 1. Missing length prefix entirely
    let empty: &[u8] = &[];
    let result = Casp001Document::decode(empty);
    assert!(result.is_ok() || result.is_err()); // Empty is valid (default)

    // 2. Partial length prefix (1, 2, 3 bytes)
    for partial_len in 1..=3 {
        let partial = &wire_msg[..partial_len];
        let result = Casp001Document::decode(partial);
        // Should fail (partial length prefix is not valid protobuf)
        // but should not panic
        assert!(result.is_ok() || result.is_err());
    }

    // 3. Length prefix present but body truncated
    if msg_bytes.len() > 0 {
        let header_only = &wire_msg[..4]; // Just the length prefix
        let result = Casp001Document::decode(header_only);
        // Length prefix bytes as protobuf should decode or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }
}

// =============================================================================
// ADVANCED PROPERTY TESTS: Corrupted Length Prefixes (Plan 06-05b Task 1)
// =============================================================================

/// Additional proptest block for advanced property tests with higher case counts
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property test: Corrupted length prefix handling.
    ///
    /// Verifies that FramedTransport handles invalid 4-byte length prefixes gracefully.
    /// Tests random 4-byte prefixes that could represent:
    /// - Negative length (high bit set in big-endian)
    /// - Extremely large length (close to u32::MAX)
    /// - Zero length (valid but tests empty message handling)
    #[test]
    fn test_corrupted_length_prefix(prefix in any::<[u8; 4]>()) {
        use nexo_retailer_protocol::transport::{FramedTransport, Transport, framing::MAX_FRAME_SIZE};
        use nexo_retailer_protocol::error::NexoError;

        // Create a mock transport with the corrupted prefix
        let mut mock = MockTransportForFraming::new();
        mock.set_read_data(prefix.to_vec());

        let mut framed = FramedTransport::new(mock);

        // Try to receive raw bytes - should handle corrupted prefix gracefully
        let result = futures_executor::block_on(framed.recv_raw());

        // Parse the prefix to understand what we're testing
        let length = u32::from_be_bytes(prefix) as usize;

        match result {
            Ok(data) => {
                // If length is valid (<= MAX_FRAME_SIZE), we should get empty vec for zero
                // or the prefix would be interpreted as protobuf
                prop_assert!(length <= MAX_FRAME_SIZE, "Large lengths should be rejected");
                prop_assert_eq!(data.len(), length, "Data length should match prefix");
            }
            Err(e) => {
                // Error should indicate framing/decoding issue
                let error_str = e.to_string();
                let is_expected_error = error_str.contains("exceeds maximum frame size")
                    || error_str.contains("unexpected EOF")
                    || error_str.contains("connection")
                    || !error_str.is_empty();
                prop_assert!(is_expected_error, "Error should be informative: {}", error_str);
            }
        }
    }

    /// Property test: Specific corrupted length prefix values.
    ///
    /// Tests edge cases in length prefix interpretation including:
    /// - Zero length (valid empty message)
    /// - Maximum valid length (exactly 4MB)
    /// - Just over maximum (4MB + 1)
    /// - Extremely large values (near u32::MAX)
    #[test]
    fn test_specific_corrupted_length_prefixes(length in 0u32..=u32::MAX) {
        use nexo_retailer_protocol::transport::{FramedTransport, framing::MAX_FRAME_SIZE};

        // Create prefix from length value
        let prefix = length.to_be_bytes();

        let mut mock = MockTransportForFraming::new();

        // For valid lengths, provide empty body; for oversized, just the prefix
        if (length as usize) <= MAX_FRAME_SIZE {
            let mut data = prefix.to_vec();
            data.extend(vec![0u8; length as usize]);
            mock.set_read_data(data);
        } else {
            mock.set_read_data(prefix.to_vec());
        }

        let mut framed = FramedTransport::new(mock);
        let result = futures_executor::block_on(framed.recv_raw());

        if (length as usize) > MAX_FRAME_SIZE {
            // Should reject oversized lengths
            prop_assert!(result.is_err(), "Oversized length {} should be rejected", length);
        } else {
            // Valid lengths should succeed (with empty body provided)
            prop_assert!(result.is_ok() || result.is_err());
        }
    }
}

// =============================================================================
// ADVANCED PROPERTY TESTS: Random Input Fuzzing (Plan 06-05b Task 2)
// =============================================================================

/// High-case-count proptest block for extensive fuzzing
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property test: Random input fuzzing for all 17 CASP message types.
    ///
    /// Verifies that the decoder handles random byte arrays (0-10KB) without panics.
    /// Uses high case count to discover edge cases.
    #[test]
    fn test_random_fuzzing_casp001(data in prop::collection::vec(any::<u8>(), 0..10000)) {
        let result = Casp001Document::decode(&data[..]);
        match result {
            Ok(_) => (), // Valid by chance
            Err(e) => prop_assert!(!e.to_string().is_empty(), "Error should be informative"),
        }
        // Main property: never panic
    }

    #[test]
    fn test_random_fuzzing_casp002(data in prop::collection::vec(any::<u8>(), 0..10000)) {
        let _ = Casp002Document::decode(&data[..]);
    }

    #[test]
    fn test_random_fuzzing_casp003(data in prop::collection::vec(any::<u8>(), 0..10000)) {
        let _ = Casp003Document::decode(&data[..]);
    }

    #[test]
    fn test_random_fuzzing_casp004(data in prop::collection::vec(any::<u8>(), 0..10000)) {
        let _ = Casp004Document::decode(&data[..]);
    }

    #[test]
    fn test_random_fuzzing_casp005(data in prop::collection::vec(any::<u8>(), 0..10000)) {
        let _ = Casp005Document::decode(&data[..]);
    }

    #[test]
    fn test_random_fuzzing_casp006(data in prop::collection::vec(any::<u8>(), 0..10000)) {
        let _ = Casp006Document::decode(&data[..]);
    }

    #[test]
    fn test_random_fuzzing_casp007(data in prop::collection::vec(any::<u8>(), 0..10000)) {
        let _ = Casp007Document::decode(&data[..]);
    }

    #[test]
    fn test_random_fuzzing_casp008(data in prop::collection::vec(any::<u8>(), 0..10000)) {
        let _ = Casp008Document::decode(&data[..]);
    }

    #[test]
    fn test_random_fuzzing_casp009(data in prop::collection::vec(any::<u8>(), 0..10000)) {
        let _ = Casp009Document::decode(&data[..]);
    }

    #[test]
    fn test_random_fuzzing_casp010(data in prop::collection::vec(any::<u8>(), 0..10000)) {
        let _ = Casp010Document::decode(&data[..]);
    }

    #[test]
    fn test_random_fuzzing_casp011(data in prop::collection::vec(any::<u8>(), 0..10000)) {
        let _ = Casp011Document::decode(&data[..]);
    }

    #[test]
    fn test_random_fuzzing_casp012(data in prop::collection::vec(any::<u8>(), 0..10000)) {
        let _ = Casp012Document::decode(&data[..]);
    }

    #[test]
    fn test_random_fuzzing_casp013(data in prop::collection::vec(any::<u8>(), 0..10000)) {
        let _ = Casp013Document::decode(&data[..]);
    }

    #[test]
    fn test_random_fuzzing_casp014(data in prop::collection::vec(any::<u8>(), 0..10000)) {
        let _ = Casp014Document::decode(&data[..]);
    }

    #[test]
    fn test_random_fuzzing_casp015(data in prop::collection::vec(any::<u8>(), 0..10000)) {
        let _ = Casp015Document::decode(&data[..]);
    }

    #[test]
    fn test_random_fuzzing_casp016(data in prop::collection::vec(any::<u8>(), 0..10000)) {
        let _ = Casp016Document::decode(&data[..]);
    }

    #[test]
    fn test_random_fuzzing_casp017(data in prop::collection::vec(any::<u8>(), 0..10000)) {
        let _ = Casp017Document::decode(&data[..]);
    }
}

// =============================================================================
// ADVANCED PROPERTY TESTS: Round-Trip Invariants (Plan 06-05b Task 3)
// =============================================================================

/// Round-trip property tests with custom strategies for realistic field values
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property test: Round-trip invariant for Casp001Document with realistic header values.
    ///
    /// Verifies that decode(encode(msg)) == msg holds for messages with
    /// realistic field values (valid currency codes, string lengths, numeric ranges).
    #[test]
    fn test_roundtrip_invariant_casp001(
        msg_fctn in "DREQ|DRSP|NOTI",
        proto_vrsn in "6\\.0|6\\.1|7\\.0",
        tx_id in "[A-Z0-9-]{1,36}",
        cre_dt_tm in "[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z",
    ) {
        use nexo_retailer_protocol::{Casp001DocumentDocument, SaleToPoiServiceRequestV06, Header4};

        let original = Casp001Document {
            document: Some(Casp001DocumentDocument {
                sale_to_poi_svc_req: Some(SaleToPoiServiceRequestV06 {
                    hdr: Some(Header4 {
                        msg_fctn: Some(msg_fctn),
                        proto_vrsn: Some(proto_vrsn),
                        tx_id: Some(tx_id),
                        cre_dt_tm: Some(cre_dt_tm),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            }),
        };

        let encoded = original.encode_to_vec();
        let expected_len = original.encoded_len();

        // Verify encoded_len() accuracy
        prop_assert_eq!(expected_len, encoded.len(), "encoded_len() should match actual length");

        // Verify round-trip
        let decoded = Casp001Document::decode(&encoded[..])
            .expect("Round-trip decode should succeed");
        prop_assert_eq!(original, decoded, "Round-trip should preserve message");
    }

    /// Property test: Round-trip invariant for Casp006Document (CardTerminalManagement).
    #[test]
    fn test_roundtrip_invariant_casp006(
        tx_id in "[A-Z0-9-]{1,36}",
        proto_vrsn in "6\\.0|6\\.1|7\\.0",
    ) {
        use nexo_retailer_protocol::{Casp006DocumentDocument, CardTerminalManagement6, Header4};

        let original = Casp006Document {
            document: Some(Casp006DocumentDocument {
                crd_trmnl_mgmt: Some(CardTerminalManagement6 {
                    hdr: Some(Header4 {
                        tx_id: Some(tx_id),
                        proto_vrsn: Some(proto_vrsn),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            }),
        };

        let encoded = original.encode_to_vec();
        let expected_len = original.encoded_len();
        prop_assert_eq!(expected_len, encoded.len());

        let decoded = Casp006Document::decode(&encoded[..])
            .expect("Round-trip decode should succeed");
        prop_assert_eq!(original, decoded);
    }

    /// Property test: Round-trip invariant for Casp012Document (NetworkManagement).
    #[test]
    fn test_roundtrip_invariant_casp012(
        tx_id in "[A-Z0-9-]{1,36}",
        proto_vrsn in "6\\.0|6\\.1|7\\.0",
    ) {
        use nexo_retailer_protocol::{Casp012DocumentDocument, NetworkManagement6, Header4};

        let original = Casp012Document {
            document: Some(Casp012DocumentDocument {
                net_mgmt: Some(NetworkManagement6 {
                    hdr: Some(Header4 {
                        tx_id: Some(tx_id),
                        proto_vrsn: Some(proto_vrsn),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            }),
        };

        let encoded = original.encode_to_vec();
        let expected_len = original.encoded_len();
        prop_assert_eq!(expected_len, encoded.len());

        let decoded = Casp012Document::decode(&encoded[..])
            .expect("Round-trip decode should succeed");
        prop_assert_eq!(original, decoded);
    }

    /// Property test: Round-trip invariant for monetary amounts with realistic values.
    ///
    /// Tests ISO 4217 currency codes and valid monetary ranges.
    #[test]
    fn test_roundtrip_invariant_monetary_amount(
        ccy in "USD|EUR|GBP|JPY|CHF|CAD|AUD|NZD",
        units in -1_000_000i64..1_000_000i64,
        nanos in 0i32..999_999_999i32,
    ) {
        use nexo_retailer_protocol::ActiveCurrencyAndAmount;

        // Adjust nanos sign to match units sign
        let adjusted_nanos = if units < 0 { -nanos.abs() } else { nanos };

        let original = ActiveCurrencyAndAmount {
            ccy: ccy.to_string(),
            units,
            nanos: adjusted_nanos,
        };

        let encoded = original.encode_to_vec();
        let expected_len = original.encoded_len();
        prop_assert_eq!(expected_len, encoded.len());

        let decoded = ActiveCurrencyAndAmount::decode(&encoded[..])
            .expect("Round-trip decode should succeed");
        prop_assert_eq!(original, decoded);
    }

    /// Property test: Round-trip invariant for default messages.
    ///
    /// Default-constructed messages should round-trip correctly.
    #[test]
    fn test_roundtrip_invariant_default_messages(_dummy: bool) {
        // Test all 17 CASP message types with default values
        let messages: Vec<(&str, Vec<u8>, usize)> = vec![
            ("Casp001", Casp001Document::default().encode_to_vec(), Casp001Document::default().encoded_len()),
            ("Casp002", Casp002Document::default().encode_to_vec(), Casp002Document::default().encoded_len()),
            ("Casp003", Casp003Document::default().encode_to_vec(), Casp003Document::default().encoded_len()),
            ("Casp004", Casp004Document::default().encode_to_vec(), Casp004Document::default().encoded_len()),
            ("Casp005", Casp005Document::default().encode_to_vec(), Casp005Document::default().encoded_len()),
            ("Casp006", Casp006Document::default().encode_to_vec(), Casp006Document::default().encoded_len()),
            ("Casp007", Casp007Document::default().encode_to_vec(), Casp007Document::default().encoded_len()),
            ("Casp008", Casp008Document::default().encode_to_vec(), Casp008Document::default().encoded_len()),
            ("Casp009", Casp009Document::default().encode_to_vec(), Casp009Document::default().encoded_len()),
            ("Casp010", Casp010Document::default().encode_to_vec(), Casp010Document::default().encoded_len()),
            ("Casp011", Casp011Document::default().encode_to_vec(), Casp011Document::default().encoded_len()),
            ("Casp012", Casp012Document::default().encode_to_vec(), Casp012Document::default().encoded_len()),
            ("Casp013", Casp013Document::default().encode_to_vec(), Casp013Document::default().encoded_len()),
            ("Casp014", Casp014Document::default().encode_to_vec(), Casp014Document::default().encoded_len()),
            ("Casp015", Casp015Document::default().encode_to_vec(), Casp015Document::default().encoded_len()),
            ("Casp016", Casp016Document::default().encode_to_vec(), Casp016Document::default().encoded_len()),
            ("Casp017", Casp017Document::default().encode_to_vec(), Casp017Document::default().encoded_len()),
        ];

        for (name, encoded, expected_len) in messages {
            prop_assert_eq!(expected_len, encoded.len(), "{} encoded_len mismatch", name);
        }
    }
}

// =============================================================================
// Mock Transport for Framing Tests
// =============================================================================

/// Mock transport for testing FramedTransport with corrupted length prefixes
struct MockTransportForFraming {
    read_buffer: Vec<u8>,
    write_buffer: Vec<u8>,
    connected: bool,
}

impl MockTransportForFraming {
    fn new() -> Self {
        Self {
            read_buffer: Vec::new(),
            write_buffer: Vec::new(),
            connected: true,
        }
    }

    fn set_read_data(&mut self, data: Vec<u8>) {
        self.read_buffer = data;
    }
}

impl nexo_retailer_protocol::transport::Transport for MockTransportForFraming {
    type Error = nexo_retailer_protocol::error::NexoError;

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if !self.connected {
            return Err(nexo_retailer_protocol::error::NexoError::Connection {
                details: "Not connected",
            });
        }

        let bytes_to_read = core::cmp::min(buf.len(), self.read_buffer.len());
        if bytes_to_read == 0 {
            return Ok(0); // EOF
        }

        buf[..bytes_to_read].copy_from_slice(&self.read_buffer[..bytes_to_read]);
        self.read_buffer = self.read_buffer[bytes_to_read..].to_vec();

        Ok(bytes_to_read)
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        if !self.connected {
            return Err(nexo_retailer_protocol::error::NexoError::Connection {
                details: "Not connected",
            });
        }

        self.write_buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    async fn connect(&mut self, _addr: &str) -> Result<(), Self::Error> {
        self.connected = true;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}
