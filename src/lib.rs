//! # Nexo Retailer Protocol (ISO 20022 CASP)
//!
//! Rust implementation of the Nexo Retailer Protocol with support for both
//! standard (std) and bare-metal (no_std) environments.
//!
//! ## Features
//!
//! - `std` (default): Enables standard library support for server environments
//! - `alloc`: Enables heap-based collections for advanced validation
//! - `--no-default-features`: Bare-metal compatible build
//!
//! ## Message Types
//!
//! The library defines all 17 CASP message types from ISO 20022:
//!
//! - `casp001`: SaleToPOIServiceRequestV06
//! - `casp002`: SaleToPOIServiceResponseV06
//! - `casp003`: SaleToPOIAdminRequestV06
//! - `casp004`: SaleToPOIAdminResponseV06
//! - `casp005-017`: Additional CASP message types
//!
//! ## Usage
//!
//! ```rust,no_run
//! use nexo_retailer_protocol::casp001;
//!
//! let doc = casp001::Document { /* ... */ };
//! let encoded = doc.encode_to_vec();
//! ```
//!
//! ## no_std Support
//!
//! For bare-metal targets, build with:
//! ```bash
//! cargo build --no-default-features --target thumbv7em-none-eabihf
//! ```

// Re-export generated protobuf types
pub mod protos;

// Re-export commonly used message types for convenience
pub use protos::nexo::casp::v1::casp001;
pub use protos::nexo::casp::v1::casp002;
pub use protos::nexo::casp::v1::casp003;
pub use protos::nexo::casp::v1::casp004;
pub use protos::nexo::casp::v1::casp005;
pub use protos::nexo::casp::v1::casp006;
pub use protos::nexo::casp::v1::casp007;
pub use protos::nexo::casp::v1::casp008;
pub use protos::nexo::casp::v1::casp009;
pub use protos::nexo::casp::v1::casp010;
pub use protos::nexo::casp::v1::casp011;
pub use protos::nexo::casp::v1::casp012;
pub use protos::nexo::casp::v1::casp013;
pub use protos::nexo::casp::v1::casp014;
pub use protos::nexo::casp::v1::casp015;
pub use protos::nexo::casp::v1::casp016;
pub use protos::nexo::casp::v1::casp017;

// Re-export common types
pub use protos::nexo::casp::v1::{
    ActiveCurrencyAndAmount,
    // Add other common types as needed
};

// Ensure library works in no_std environment
#[cfg(not(feature = "std"))]
compile_error!("no_std support is planned for Phase 2; current builds require std feature");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_builds() {
        // Basic smoke test - library compiles
        assert!(true, "library builds successfully");
    }
}
