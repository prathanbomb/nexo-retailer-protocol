//! Codec round-trip unit tests for all 17 CASP message types
//!
//! This module tests the encode/decode round-trip correctness for all CASP
//! message types (Casp001Document through Casp017Document).
//!
//! # Test Strategy
//!
//! 1. **Default round-trip tests**: Verify that default-constructed messages
//!    encode and decode correctly.
//! 2. **Populated round-trip tests**: Verify that messages with simple populated
//!    fields encode and decode correctly.
//! 3. **Encoded length tests**: Verify that `encoded_len()` matches the
//!    actual encoded byte length.
//!
//! # Feature Flags
//!
//! - Round-trip tests require `alloc` feature for Vec allocation.
//! - Property-based tests require `std` feature (proptest requires std).

#![cfg(all(test, feature = "alloc"))]

use nexo_retailer_protocol::{
    Casp001Document, Casp002Document, Casp003Document, Casp004Document, Casp005Document,
    Casp006Document, Casp007Document, Casp008Document, Casp009Document, Casp010Document,
    Casp011Document, Casp012Document, Casp013Document, Casp014Document, Casp015Document,
    Casp016Document, Casp017Document,
};
use prost::Message;

/// Macro to generate round-trip tests for a CASP message type.
///
/// Generates three test functions:
/// - `test_{type_name}_roundtrip_default` - Tests default message round-trip
/// - `test_{type_name}_roundtrip_populated` - Tests message with fields set
/// - `test_{type_name}_encoded_len_matches` - Verifies encoded_len() accuracy
macro_rules! gen_roundtrip_test {
    ($type_name:ident, $create_populated_fn:expr) => {
        paste::paste! {
            #[test]
            fn [<test_ $type_name:snake _roundtrip_default>]() {
                let original = $type_name::default();
                let encoded = original.encode_to_vec();
                let decoded = $type_name::decode(&encoded[..]).expect("decode should succeed");
                assert_eq!(original, decoded, "round-trip should preserve default message");
            }

            #[test]
            fn [<test_ $type_name:snake _roundtrip_populated>]() {
                let original = $create_populated_fn();
                let encoded = original.encode_to_vec();
                let decoded = $type_name::decode(&encoded[..]).expect("decode should succeed");
                assert_eq!(original, decoded, "round-trip should preserve populated message");
            }

            #[test]
            fn [<test_ $type_name:snake _encoded_len_matches>]() {
                let msg = $create_populated_fn();
                let expected_len = msg.encoded_len();
                let encoded = msg.encode_to_vec();
                assert_eq!(
                    expected_len,
                    encoded.len(),
                    "encoded_len() should match actual encoded byte length"
                );
            }
        }
    };
}

// =============================================================================
// Test Data Helpers - Create populated messages with realistic field values
// =============================================================================

use nexo_retailer_protocol::{
    Header4, Casp001DocumentDocument, Casp002DocumentDocument, Casp003DocumentDocument,
    Casp004DocumentDocument, Casp005DocumentDocument, Casp006DocumentDocument,
    Casp007DocumentDocument, Casp008DocumentDocument, Casp009DocumentDocument,
    Casp010DocumentDocument, Casp011DocumentDocument, Casp012DocumentDocument,
    Casp013DocumentDocument, Casp014DocumentDocument, Casp015DocumentDocument,
    Casp016DocumentDocument, Casp017DocumentDocument,
    SaleToPoiServiceRequestV06, SaleToPoiServiceResponseV06,
    SaleToPoiAdminRequestV06, SaleToPoiAdminResponseV06,
    TransactionManagement6, CardTerminalManagement6, PinManagement6,
    DisplayManagement6, InputManagement6, CardDataManagement6,
    LoginManagement6, NetworkManagement6, SecurityManagement2,
    CertificateManagement6, TotaliserManagement6, PrintManagement6,
    ApplicationManagement6,
    ActiveCurrencyAndAmount,
};

// =============================================================================
// ISO 4217 Currency Code Test Data
// =============================================================================

/// Create a monetary amount in USD following ActiveCurrencyAndAmount rules.
///
/// Uses integer representation: units + nanos/10^9
/// Example: $100.50 = { units: 100, nanos: 500000000 }
fn create_usd_amount(units: i64, nanos: i32) -> ActiveCurrencyAndAmount {
    ActiveCurrencyAndAmount {
        ccy: "USD".to_string(),
        units,
        nanos,
    }
}

/// Create a monetary amount in EUR following ActiveCurrencyAndAmount rules.
fn create_eur_amount(units: i64, nanos: i32) -> ActiveCurrencyAndAmount {
    ActiveCurrencyAndAmount {
        ccy: "EUR".to_string(),
        units,
        nanos,
    }
}

/// Create a monetary amount in JPY following ActiveCurrencyAndAmount rules.
/// JPY has no decimal places, so nanos should be 0.
fn create_jpy_amount(units: i64) -> ActiveCurrencyAndAmount {
    ActiveCurrencyAndAmount {
        ccy: "JPY".to_string(),
        units,
        nanos: 0,
    }
}

/// Create a test header with realistic field values.
fn create_test_header() -> Header4 {
    Header4 {
        msg_fctn: Some("DREQ".to_string()),
        proto_vrsn: Some("6.0".to_string()),
        tx_id: Some("TX-12345".to_string()),
        cre_dt_tm: Some("2024-01-15T10:30:00Z".to_string()),
        ..Default::default()
    }
}

/// Create a populated Casp001Document (SaleToPOIServiceRequestV06).
fn create_test_casp001() -> Casp001Document {
    Casp001Document {
        document: Some(Casp001DocumentDocument {
            sale_to_poi_svc_req: Some(SaleToPoiServiceRequestV06 {
                hdr: Some(create_test_header()),
                ..Default::default()
            }),
        }),
    }
}

/// Create a populated Casp002Document (SaleToPOIServiceResponseV06).
fn create_test_casp002() -> Casp002Document {
    Casp002Document {
        document: Some(Casp002DocumentDocument {
            sale_to_poi_svc_rsp: Some(SaleToPoiServiceResponseV06 {
                hdr: Some(create_test_header()),
                ..Default::default()
            }),
        }),
    }
}

/// Create a populated Casp003Document (SaleToPOIAdminRequestV06).
fn create_test_casp003() -> Casp003Document {
    Casp003Document {
        document: Some(Casp003DocumentDocument {
            sale_to_poi_adm_req: Some(SaleToPoiAdminRequestV06 {
                hdr: Some(create_test_header()),
                ..Default::default()
            }),
        }),
    }
}

/// Create a populated Casp004Document (SaleToPOIAdminResponseV06).
fn create_test_casp004() -> Casp004Document {
    Casp004Document {
        document: Some(Casp004DocumentDocument {
            sale_to_poi_adm_rsp: Some(SaleToPoiAdminResponseV06 {
                hdr: Some(create_test_header()),
                ..Default::default()
            }),
        }),
    }
}

/// Create a populated Casp005Document (TransactionManagement6).
fn create_test_casp005() -> Casp005Document {
    Casp005Document {
        document: Some(Casp005DocumentDocument {
            tx_mgmt: Some(TransactionManagement6 {
                hdr: Some(create_test_header()),
                ..Default::default()
            }),
        }),
    }
}

/// Create a populated Casp006Document (CardTerminalManagement6).
fn create_test_casp006() -> Casp006Document {
    Casp006Document {
        document: Some(Casp006DocumentDocument {
            crd_trmnl_mgmt: Some(CardTerminalManagement6 {
                hdr: Some(create_test_header()),
                ..Default::default()
            }),
        }),
    }
}

/// Create a populated Casp007Document (PinManagement6).
fn create_test_casp007() -> Casp007Document {
    Casp007Document {
        document: Some(Casp007DocumentDocument {
            pin_mgmt: Some(PinManagement6 {
                hdr: Some(create_test_header()),
                ..Default::default()
            }),
        }),
    }
}

/// Create a populated Casp008Document (DisplayManagement6).
fn create_test_casp008() -> Casp008Document {
    Casp008Document {
        document: Some(Casp008DocumentDocument {
            dsply_mgmt: Some(DisplayManagement6 {
                hdr: Some(create_test_header()),
                ..Default::default()
            }),
        }),
    }
}

/// Create a populated Casp009Document (InputManagement6).
fn create_test_casp009() -> Casp009Document {
    Casp009Document {
        document: Some(Casp009DocumentDocument {
            inp_mgmt: Some(InputManagement6 {
                hdr: Some(create_test_header()),
                ..Default::default()
            }),
        }),
    }
}

/// Create a populated Casp010Document (CardDataManagement6).
fn create_test_casp010() -> Casp010Document {
    Casp010Document {
        document: Some(Casp010DocumentDocument {
            crd_data_mgmt: Some(CardDataManagement6 {
                hdr: Some(create_test_header()),
                ..Default::default()
            }),
        }),
    }
}

/// Create a populated Casp011Document (LoginManagement6).
fn create_test_casp011() -> Casp011Document {
    Casp011Document {
        document: Some(Casp011DocumentDocument {
            login_mgmt: Some(LoginManagement6 {
                hdr: Some(create_test_header()),
                ..Default::default()
            }),
        }),
    }
}

/// Create a populated Casp012Document (NetworkManagement6).
fn create_test_casp012() -> Casp012Document {
    Casp012Document {
        document: Some(Casp012DocumentDocument {
            net_mgmt: Some(NetworkManagement6 {
                hdr: Some(create_test_header()),
                ..Default::default()
            }),
        }),
    }
}

/// Create a populated Casp013Document (SecurityManagement2).
fn create_test_casp013() -> Casp013Document {
    Casp013Document {
        document: Some(Casp013DocumentDocument {
            scnty_mgmt: Some(SecurityManagement2 {
                hdr: Some(create_test_header()),
                ..Default::default()
            }),
        }),
    }
}

/// Create a populated Casp014Document (CertificateManagement6).
fn create_test_casp014() -> Casp014Document {
    Casp014Document {
        document: Some(Casp014DocumentDocument {
            cert_mgmt: Some(CertificateManagement6 {
                hdr: Some(create_test_header()),
                ..Default::default()
            }),
        }),
    }
}

/// Create a populated Casp015Document (TotaliserManagement6).
fn create_test_casp015() -> Casp015Document {
    Casp015Document {
        document: Some(Casp015DocumentDocument {
            totlsr_mgmt: Some(TotaliserManagement6 {
                hdr: Some(create_test_header()),
                ..Default::default()
            }),
        }),
    }
}

/// Create a populated Casp016Document (PrintManagement6).
fn create_test_casp016() -> Casp016Document {
    Casp016Document {
        document: Some(Casp016DocumentDocument {
            prnt_mgmt: Some(PrintManagement6 {
                hdr: Some(create_test_header()),
                ..Default::default()
            }),
        }),
    }
}

/// Create a populated Casp017Document (ApplicationManagement6).
fn create_test_casp017() -> Casp017Document {
    Casp017Document {
        document: Some(Casp017DocumentDocument {
            app_mgmt: Some(ApplicationManagement6 {
                hdr: Some(create_test_header()),
                ..Default::default()
            }),
        }),
    }
}

// =============================================================================
// Generate Round-Trip Tests for All 17 CASP Message Types
// =============================================================================

gen_roundtrip_test!(Casp001Document, create_test_casp001);
gen_roundtrip_test!(Casp002Document, create_test_casp002);
gen_roundtrip_test!(Casp003Document, create_test_casp003);
gen_roundtrip_test!(Casp004Document, create_test_casp004);
gen_roundtrip_test!(Casp005Document, create_test_casp005);
gen_roundtrip_test!(Casp006Document, create_test_casp006);
gen_roundtrip_test!(Casp007Document, create_test_casp007);
gen_roundtrip_test!(Casp008Document, create_test_casp008);
gen_roundtrip_test!(Casp009Document, create_test_casp009);
gen_roundtrip_test!(Casp010Document, create_test_casp010);
gen_roundtrip_test!(Casp011Document, create_test_casp011);
gen_roundtrip_test!(Casp012Document, create_test_casp012);
gen_roundtrip_test!(Casp013Document, create_test_casp013);
gen_roundtrip_test!(Casp014Document, create_test_casp014);
gen_roundtrip_test!(Casp015Document, create_test_casp015);
gen_roundtrip_test!(Casp016Document, create_test_casp016);
gen_roundtrip_test!(Casp017Document, create_test_casp017);

// =============================================================================
// Monetary Amount Round-Trip Tests (ISO 4217 Currency Codes)
// =============================================================================

/// Test that ActiveCurrencyAndAmount with USD currency round-trips correctly.
#[test]
fn test_usd_amount_roundtrip() {
    let original = create_usd_amount(100, 500000000); // $100.50
    let encoded = original.encode_to_vec();
    let decoded = ActiveCurrencyAndAmount::decode(&encoded[..]).expect("decode should succeed");
    assert_eq!(original, decoded);
    assert_eq!(original.ccy, "USD");
    assert_eq!(original.units, 100);
    assert_eq!(original.nanos, 500000000);
}

/// Test that ActiveCurrencyAndAmount with EUR currency round-trips correctly.
#[test]
fn test_eur_amount_roundtrip() {
    let original = create_eur_amount(50, 25000000); // EUR 50.025
    let encoded = original.encode_to_vec();
    let decoded = ActiveCurrencyAndAmount::decode(&encoded[..]).expect("decode should succeed");
    assert_eq!(original, decoded);
    assert_eq!(original.ccy, "EUR");
}

/// Test that ActiveCurrencyAndAmount with JPY currency round-trips correctly.
#[test]
fn test_jpy_amount_roundtrip() {
    let original = create_jpy_amount(10000); // JPY 10,000 (no decimal places)
    let encoded = original.encode_to_vec();
    let decoded = ActiveCurrencyAndAmount::decode(&encoded[..]).expect("decode should succeed");
    assert_eq!(original, decoded);
    assert_eq!(original.ccy, "JPY");
    assert_eq!(original.nanos, 0);
}

/// Test negative monetary amounts (refunds, credits).
#[test]
fn test_negative_amount_roundtrip() {
    let original = ActiveCurrencyAndAmount {
        ccy: "USD".to_string(),
        units: -50,
        nanos: -750000000, // -$50.75 (sign must match units)
    };
    let encoded = original.encode_to_vec();
    let decoded = ActiveCurrencyAndAmount::decode(&encoded[..]).expect("decode should succeed");
    assert_eq!(original, decoded);
}

/// Test encoded_len accuracy for monetary amounts.
#[test]
fn test_amount_encoded_len_matches() {
    let amounts = vec![
        create_usd_amount(100, 500000000),
        create_eur_amount(1000, 0),
        create_jpy_amount(1000000),
    ];

    for amount in amounts {
        let expected_len = amount.encoded_len();
        let encoded = amount.encode_to_vec();
        assert_eq!(
            expected_len, encoded.len(),
            "encoded_len mismatch for {:?}",
            amount.ccy
        );
    }
}

// =============================================================================
// Property-Based Tests (std feature required for proptest)
// =============================================================================

#[cfg(feature = "std")]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Configure proptest with regression persistence
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Property test: Casp001Document round-trip invariant
        ///
        /// Verifies that decode(encode(msg)) == msg holds for randomly
        /// generated message structures.
        #[test]
        fn protobuf_roundtrip_casp001(
            msg_fctn in "DREQ|DRSP|NOTI",
            tx_id in "[A-Z0-9-]{5,20}",
            proto_vrsn in "6\\.0|6\\.1|7\\.0",
        ) {
            let original = Casp001Document {
                document: Some(Casp001DocumentDocument {
                    sale_to_poi_svc_req: Some(SaleToPoiServiceRequestV06 {
                        hdr: Some(Header4 {
                            msg_fctn: Some(msg_fctn),
                            proto_vrsn: Some(proto_vrsn),
                            tx_id: Some(tx_id),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                }),
            };

            let encoded = original.encode_to_vec();
            let decoded = Casp001Document::decode(&encoded[..]).expect("decode should succeed");

            prop_assert_eq!(original, decoded);
        }

        /// Property test: encoded_len() accuracy for Casp001Document
        ///
        /// Verifies that encoded_len() matches actual encoded byte length.
        #[test]
        fn encode_length_matches_encoded_len_casp001(
            tx_id in "[A-Z0-9-]{1,50}",
            proto_vrsn in "[0-9]\\.[0-9]",
        ) {
            let msg = Casp001Document {
                document: Some(Casp001DocumentDocument {
                    sale_to_poi_svc_req: Some(SaleToPoiServiceRequestV06 {
                        hdr: Some(Header4 {
                            tx_id: Some(tx_id),
                            proto_vrsn: Some(proto_vrsn),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                }),
            };

            let expected_len = msg.encoded_len();
            let encoded = msg.encode_to_vec();

            prop_assert_eq!(expected_len, encoded.len());
        }

        /// Property test: Casp006Document round-trip invariant
        #[test]
        fn protobuf_roundtrip_casp006(
            tx_id in "[A-Z0-9-]{5,20}",
        ) {
            let original = Casp006Document {
                document: Some(Casp006DocumentDocument {
                    crd_trmnl_mgmt: Some(CardTerminalManagement6 {
                        hdr: Some(Header4 {
                            tx_id: Some(tx_id),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                }),
            };

            let encoded = original.encode_to_vec();
            let decoded = Casp006Document::decode(&encoded[..]).expect("decode should succeed");

            prop_assert_eq!(original, decoded);
        }

        /// Property test: Casp012Document round-trip invariant
        #[test]
        fn protobuf_roundtrip_casp012(
            tx_id in "[A-Z0-9-]{5,20}",
        ) {
            let original = Casp012Document {
                document: Some(Casp012DocumentDocument {
                    net_mgmt: Some(NetworkManagement6 {
                        hdr: Some(Header4 {
                            tx_id: Some(tx_id),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                }),
            };

            let encoded = original.encode_to_vec();
            let decoded = Casp012Document::decode(&encoded[..]).expect("decode should succeed");

            prop_assert_eq!(original, decoded);
        }

        /// Property test: Empty message round-trip
        ///
        /// Verifies that messages with all default values round-trip correctly.
        #[test]
        fn empty_message_roundtrip(
            _useless in 0..10i32,
        ) {
            let original = Casp001Document::default();
            let encoded = original.encode_to_vec();
            let decoded = Casp001Document::decode(&encoded[..]).expect("decode should succeed");
            prop_assert_eq!(original, decoded);
        }
    }
}
