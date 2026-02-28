//! Message validation for Nexo Retailer Protocol
//!
//! This module provides validation functions for CASP message fields
//! according to XSD constraints and the Nexo protocol specification.
//!
//! # Validation Modules
//!
//! - **currency**: ISO 4217 currency code validation and monetary amount validation
//! - **strings**: String length validation for XSD types (Max256Text, Max20000Text, etc.)
//! - **constraints**: Field presence validation, type validators, and Validate trait
//!
//! # Feature Flags
//!
//! - **alloc feature**: Enables collection validation (repeated field size limits)
//! - **no_std**: Basic validation (currency, strings, required fields) works without alloc
//!
//! # Usage
//!
//! ```rust,ignore
//! use nexo_retailer_protocol::{
//!     Validate, validate_currency_code, validate_monetary_amount,
//!     validate_max256_text, validate_required
//! };
//! use nexo_retailer_protocol::{Header4, ActiveCurrencyAndAmount};
//!
//! // Validate individual fields
//! validate_currency_code("USD")?;
//! validate_max256_text("Some text")?;
//!
//! // Validate entire messages
//! let header = Header4 { /* fields */ };
//! header.validate()?;
//!
//! let amount = ActiveCurrencyAndAmount { /* fields */ };
//! amount.validate()?;
//! ```
//!
//! # Validate Trait
//!
//! The `Validate` trait is implemented for key message types:
//! - `ActiveCurrencyAndAmount`: Currency code, nanos range, sign consistency
//! - `Header4`: String length constraints, nested message validation
//! - `CardData8`: Card number length, expiration date format
//! - And more...
//!
//! ```rust,ignore
//! use nexo_retailer_protocol::Validate;
//!
//! message.validate()?; // Validates all fields recursively
//! ```

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
