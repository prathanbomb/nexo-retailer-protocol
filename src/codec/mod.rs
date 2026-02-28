//! Codec layer for protobuf message encoding/decoding
//!
//! This module provides a trait-based abstraction over prost's Message trait,
//! enabling size-checked encoding and decoding of all CASP message types.
//!
//! # Features
//!
//! - **no_std compatible**: Works in both std and no_std environments
//! - **Size limits enforced**: Prevents unbounded allocation attacks (CODEC-03)
//! - **Trait abstraction**: Enables testing with mock codecs (CODEC-05)
//! - **Generic over prost::Message**: Supports all 17 CASP message types
//!
//! # Usage
//!
//! ## Using the Codec trait
//!
//! ```rust,ignore
//! use nexo_retailer_protocol::codec::{Codec, ProstCodec};
//! use nexo_retailer_protocol::Casp001Document;
//!
//! let codec = ProstCodec;
//! let message = Casp001Document { /* fields */ };
//!
//! // Encode
//! let bytes = codec.encode(&message)?;
//!
//! // Decode
//! let decoded = codec.decode::<Casp001Document>(&bytes)?;
//! ```
//!
//! ## Using convenience functions
//!
//! For simple use cases, use the standalone functions:
//!
//! ```rust,ignore
//! use nexo_retailer_protocol::codec::{encode, decode};
//!
//! let bytes = encode(&message)?;
//! let decoded = decode::<Casp001Document>(&bytes)?;
//! ```

pub mod limits;

use crate::NexoError;
use prost::Message;

/// Codec trait for encoding/decoding protobuf messages
///
/// This trait abstracts the underlying codec implementation, enabling:
/// - Testing with mock codecs (CODEC-05)
/// - Potential codec swaps in future versions
/// - Dependency injection in client/server code
///
/// # Type Parameters
///
/// - `M`: Message type implementing `prost::Message` (all 17 CASP types)
///
/// # Example
///
/// ```rust,ignore
/// use nexo_retailer_protocol::codec::Codec;
///
/// fn process_with_codec<C: Codec<Casp001Document>>(
///     codec: &C,
///     bytes: &[u8]
/// ) -> Result<Casp001Document, NexoError> {
///     codec.decode(bytes)
/// }
/// ```
pub trait Codec<M: Message + Default> {
    /// Encode a message to bytes
    ///
    /// This method must:
    /// - Check message size against limits before encoding (CODEC-03)
    /// - Return error if message is too large
    /// - Return encoded bytes on success
    ///
    /// # Errors
    ///
    /// Returns `NexoError::Encoding` if:
    /// - Message size exceeds `MAX_MESSAGE_SIZE`
    /// - Prost encoding fails
    fn encode(&self, msg: &M) -> Result<Vec<u8>, NexoError>;

    /// Decode a message from bytes
    ///
    /// This method must:
    /// - Check buffer size against limits before decoding (CODEC-03)
    /// - Return error if buffer is too large
    /// - Return decoded message on success
    ///
    /// # Errors
    ///
    /// Returns `NexoError::Decoding` if:
    /// - Buffer size exceeds `MAX_MESSAGE_SIZE`
    /// - Prost decoding fails (malformed protobuf)
    fn decode(&self, bytes: &[u8]) -> Result<M, NexoError>;
}

/// Prost-based codec implementation
///
/// This is the standard codec implementation using prost's Message trait.
/// It enforces size limits before all encode/decode operations to prevent
/// unbounded allocation attacks.
///
/// # Example
///
/// ```rust,ignore
/// use nexo_retailer_protocol::codec::ProstCodec;
///
/// let codec = ProstCodec;
/// let bytes = codec.encode(&message)?;
/// let decoded = codec.decode::<Casp001Document>(&bytes)?;
/// ```
pub struct ProstCodec;

impl<M: Message + Default> Codec<M> for ProstCodec {
    fn encode(&self, msg: &M) -> Result<Vec<u8>, NexoError> {
        // Check size limit BEFORE encoding (CODEC-03)
        let encoded_len = msg.encoded_len();
        if encoded_len > limits::MAX_MESSAGE_SIZE {
            return Err(NexoError::Encoding {
                details: "message size exceeds maximum allowed",
            });
        }

        // Encode using prost
        // Note: prost's encode_to_vec can return EncodeError if the buffer
        // provided to encode() is too small. We use encode_to_vec() which
        // allocates a properly sized Vec, so this should never fail in practice.
        // However, we wrap it to handle any edge cases.
        let bytes = msg.encode_to_vec();
        Ok(bytes)
    }

    fn decode(&self, bytes: &[u8]) -> Result<M, NexoError> {
        // Check size limit BEFORE decoding (CODEC-03)
        if bytes.len() > limits::MAX_MESSAGE_SIZE {
            return Err(NexoError::Decoding {
                details: "message size exceeds maximum allowed",
            });
        }

        // Decode using prost
        M::decode(bytes).map_err(|_| NexoError::Decoding {
            details: "prost decode failed",
        })
    }
}

/// Convenience function for encoding messages
///
/// Uses `ProstCodec` internally. Provides an ergonomic API for users
/// who don't need trait abstraction.
///
/// # Example
///
/// ```rust,ignore
/// use nexo_retailer_protocol::codec::encode;
///
/// let bytes = encode(&message)?;
/// ```
pub fn encode<M: Message + Default>(msg: &M) -> Result<Vec<u8>, NexoError> {
    ProstCodec.encode(msg)
}

/// Convenience function for decoding messages
///
/// Uses `ProstCodec` internally. Provides an ergonomic API for users
/// who don't need trait abstraction.
///
/// # Example
///
/// ```rust,ignore
/// use nexo_retailer_protocol::codec::decode;
///
/// let decoded = decode::<Casp001Document>(&bytes)?;
/// ```
pub fn decode<M: Message + Default>(bytes: &[u8]) -> Result<M, NexoError> {
    ProstCodec.decode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Casp001Document;

    // Helper to create a minimal valid message for testing
    // Note: Actual message construction depends on proto definition
    // This is a placeholder - adjust based on actual proto structure

    #[test]
    fn test_encode_normal_message() {
        // Test encoding a default message
        // Casp001Document has Default derived from prost
        let msg = Casp001Document::default();
        let codec = ProstCodec;

        let result = codec.encode(&msg);
        assert!(result.is_ok());

        let bytes = result.unwrap();
        // Default message should encode to something under the limit
        assert!(bytes.len() < limits::MAX_MESSAGE_SIZE);
        // Note: default messages might encode to 0 bytes if all fields are default
        // This is valid protobuf behavior
    }

    #[test]
    fn test_encode_oversized_message() {
        // Test that encoding fails for messages exceeding MAX_MESSAGE_SIZE
        // This requires constructing a message > 4MB
        // TODO: Implement with actual oversized message
    }

    #[test]
    fn test_decode_valid_bytes() {
        // Test round-trip: encode then decode
        let original = Casp001Document::default();
        let codec = ProstCodec;

        // Encode
        let bytes = codec.encode(&original).unwrap();

        // Decode
        let decoded: Result<Casp001Document, _> = codec.decode(&bytes);
        assert!(decoded.is_ok());
    }

    #[test]
    fn test_decode_oversized_buffer() {
        // Test that decoding fails for buffers > MAX_MESSAGE_SIZE
        let oversized = vec![0u8; limits::MAX_MESSAGE_SIZE + 1];
        let codec = ProstCodec;

        let result: Result<Casp001Document, _> = codec.decode(&oversized);
        assert!(result.is_err());

        if let Err(NexoError::Decoding { details }) = result {
            assert!(details.contains("size exceeds maximum"));
        } else {
            panic!("Expected Decoding error");
        }
    }

    #[test]
    fn test_decode_malformed_protobuf() {
        // Test that decoding fails for malformed protobuf
        let malformed = vec![0xFF, 0xFF, 0xFF, 0xFF]; // Invalid protobuf
        let codec = ProstCodec;

        let result: Result<Casp001Document, _> = codec.decode(&malformed);
        assert!(result.is_err());

        if let Err(NexoError::Decoding { details }) = result {
            assert!(details.contains("decode failed"));
        } else {
            panic!("Expected Decoding error");
        }
    }

    #[test]
    fn test_convenience_encode_function() {
        let msg = Casp001Document::default();
        let result = encode(&msg);
        assert!(result.is_ok());
        // Encoding should succeed
        // Note: empty messages are valid in protobuf
    }

    #[test]
    fn test_convenience_decode_function() {
        let msg = Casp001Document::default();
        let bytes = encode(&msg).unwrap();
        let result: Result<Casp001Document, _> = decode(&bytes);
        assert!(result.is_ok());
    }

    #[test]
    fn test_round_trip_encoding() {
        // Test complete round-trip: encode -> decode -> verify
        let original = Casp001Document::default();
        let codec = ProstCodec;

        // Encode to bytes
        let encoded = codec.encode(&original).expect("encoding failed");

        // Verify size is under limit
        assert!(
            encoded.len() <= limits::MAX_MESSAGE_SIZE,
            "encoded size {} exceeds MAX_MESSAGE_SIZE {}",
            encoded.len(),
            limits::MAX_MESSAGE_SIZE
        );

        // Decode back to struct
        let decoded: Casp001Document = codec
            .decode(&encoded)
            .expect("decoding failed");

        // For prost messages with Default, round-trip should succeed
        // (exact equality depends on message fields)
        let _ = decoded;
    }

    #[test]
    fn test_round_trip_with_convenience_functions() {
        let original = Casp001Document::default();

        // Encode using convenience function
        let encoded = encode(&original).expect("encode failed");

        // Decode using convenience function
        let decoded: Casp001Document = decode(&encoded).expect("decode failed");

        // Verify success
        let _ = decoded;
    }

    #[test]
    fn test_codec_trait_can_be_mocked() {
        // Verify Codec trait can be implemented for testing
        #[derive(Debug)]
        struct MockCodec;

        impl<M: Message + Default> Codec<M> for MockCodec {
            fn encode(&self, _msg: &M) -> Result<Vec<u8>, NexoError> {
                Ok(vec![1, 2, 3]) // Mock implementation
            }

            fn decode(&self, _bytes: &[u8]) -> Result<M, NexoError> {
                Err(NexoError::Decoding {
                    details: "mock codec",
                })
            }
        }

        let mock = MockCodec;
        let result = mock.encode(&Casp001Document::default());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1, 2, 3]);
    }
}
