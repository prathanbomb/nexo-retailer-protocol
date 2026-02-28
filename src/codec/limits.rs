//! Message size limits for codec operations
//!
//! This module defines size limits for message encoding and decoding to prevent
//! unbounded memory allocation attacks (RESEARCH.md Pitfall 2).
//!
//! # Rationale
//!
//! The 4MB default limit follows the gRPC standard for maximum message size.
//! This prevents attackers from sending arbitrarily large messages that would
//! cause out-of-memory errors while still accommodating legitimate large messages
//! (e.g., batch transactions with many line items).
//!
//! # Usage
//!
//! Always check message size before encoding or decoding:
//!
//! ```rust,ignore
//! use nexo_retailer_protocol::codec::limits::MAX_MESSAGE_SIZE;
//!
//! // Before decode
//! if bytes.len() > MAX_MESSAGE_SIZE {
//!     return Err(NexoError::Decoding {
//!         details: "message too large"
//!     });
//! }
//!
//! // Before encode
//! if msg.encoded_len() > MAX_MESSAGE_SIZE {
//!     return Err(NexoError::Encoding {
//!         details: "message too large"
//!     });
//! }
//! ```

/// Maximum message size in bytes (4MB)
///
/// This limit follows the gRPC standard default to prevent unbounded allocation
/// attacks. Large messages exceeding this size are rejected before encoding or
/// decoding to prevent memory exhaustion.
///
/// # Why 4MB?
///
/// - gRPC default maximum message size
/// - Sufficient for batch transactions with hundreds of line items
/// - Prevents OOM from malicious input while accommodating legitimate use cases
/// - Aligns with industry best practices for message-based protocols
///
/// # Per-Message-Type Limits
///
/// Some message types may warrant lower limits (e.g., CardData8 should never
/// exceed a few KB). Implement per-message validation as needed:
///
/// ```rust,ignore
/// const MAX_CARD_DATA_SIZE: usize = 16 * 1024; // 16KB for card data
/// ```
pub const MAX_MESSAGE_SIZE: usize = 4 * 1024 * 1024;

/// Maximum size for batch messages (Casp017Document)
///
/// Batch messages can legitimately be larger than individual messages, but
/// should still be bounded to prevent denial-of-service.
///
/// Current: Same as MAX_MESSAGE_SIZE (4MB)
/// Future: Consider separate limit if actual usage patterns justify it
pub const MAX_BATCH_MESSAGE_SIZE: usize = MAX_MESSAGE_SIZE;

/// Maximum size for cardholder data messages
///
/// Card data should never be excessively large. This provides defense-in-depth
/// against malformed or malicious card data.
///
/// Set to 16KB as card data typically contains:
/// - PAN: up to 19 digits
/// - Track data: up to ~200 chars
/// - Chip data: variable but typically < 1KB
pub const MAX_CARD_DATA_SIZE: usize = 16 * 1024;

/// Maximum size for security trailer messages
///
/// Security trailers contain cryptographic signatures and certificates.
/// Set to 64KB to accommodate RSA-4096 signatures + X.509 certificates.
pub const MAX_SECURITY_TRAILER_SIZE: usize = 64 * 1024;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_message_size_is_4mb() {
        assert_eq!(MAX_MESSAGE_SIZE, 4 * 1024 * 1024);
    }

    #[test]
    fn test_max_message_size_in_bytes() {
        // 4MB = 4,194,304 bytes
        assert_eq!(MAX_MESSAGE_SIZE, 4_194_304);
    }

    #[test]
    fn test_batch_limit_matches_default() {
        assert_eq!(MAX_BATCH_MESSAGE_SIZE, MAX_MESSAGE_SIZE);
    }

    #[test]
    fn test_card_data_limit_is_16kb() {
        assert_eq!(MAX_CARD_DATA_SIZE, 16 * 1024);
    }

    #[test]
    fn test_security_trailer_limit_is_64kb() {
        assert_eq!(MAX_SECURITY_TRAILER_SIZE, 64 * 1024);
    }
}
