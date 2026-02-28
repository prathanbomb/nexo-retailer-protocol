//! Message validation for Nexo Retailer Protocol
//!
//! This module provides validation functions for CASP message fields
//! according to XSD constraints and the Nexo protocol specification.

pub mod currency;

// Re-exports for convenience
pub use currency::{validate_currency_code, validate_monetary_amount};
