//! Message validation for Nexo Retailer Protocol
//!
//! This module provides validation functions for CASP message fields
//! according to XSD constraints and the Nexo protocol specification.

pub mod constraints;
pub mod currency;
pub mod strings;

// Re-exports for convenience
pub use constraints::{
    Validate, validate_enum_value, validate_non_negative_i32, validate_positive_i64,
    validate_required,
};
// Re-export validate_repeated_field only when alloc feature is enabled
#[cfg(feature = "alloc")]
pub use constraints::validate_repeated_field;

pub use currency::{validate_currency_code, validate_monetary_amount};
pub use strings::{validate_max_text, validate_max256_text, validate_max20000_text, validate_max70_text};
