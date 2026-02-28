//! Platform feature flag module
//!
//! This module organizes platform-specific implementations based on feature flags.
//! The feature flag architecture follows the **additive principle**:
//!
//! - **`std`** (default): Enables standard library support for server environments
//! - **`alloc`**: Enables heap-based collections for advanced validation
//! - **`defmt`**: Enables embedded logging format support
//! - **`--no-default-features`**: Bare-metal compatible build (no std, no alloc)
//!
//! # Feature Flag Philosophy
//!
//! All feature flags are additive - there is no "no_std" feature. Instead:
//! - Build with `cargo build --no-default-features` for pure no_std
//! - Build with `cargo build --features std,alloc` for full std
//! - Build with `cargo build --features defmt` for embedded logging
//!
//! # Module Structure
//!
//! - **std.rs**: std-specific implementations (Tokio, standard library)
//! - **no_std.rs**: no_std-specific implementations (Embassy, heapless)
//!
//! These modules are placeholders for Phase 3+ (Transport Layer) where
//! platform-specific code will be needed.

#[cfg(feature = "std")]
pub mod std;

#[cfg(not(feature = "std"))]
pub mod no_std;
