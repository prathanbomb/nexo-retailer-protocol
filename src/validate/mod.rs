//! Message validation for Nexo Retailer Protocol
//!
//! This module provides validation functions for CASP message fields
//! according to XSD constraints and the Nexo protocol specification.

pub mod currency;
pub mod strings;

// Re-exports for convenience
pub use currency::{validate_currency_code, validate_monetary_amount};
pub use strings::{validate_max_text, validate_max256_text, validate_max20000_text, validate_max70_text};
