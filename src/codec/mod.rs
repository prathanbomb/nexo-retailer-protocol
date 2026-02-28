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
pub trait Codec<M: Message> {
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

impl<M: Message> Codec<M> for ProstCodec {
    fn encode(&self, msg: &M) -> Result<Vec<u8>, NexoError> {
        // Check size limit BEFORE encoding (CODEC-03)
        let encoded_len = msg.encoded_len();
        if encoded_len > limits::MAX_MESSAGE_SIZE {
            return Err(NexoError::Encoding {
                details: "message size exceeds maximum allowed",
            });
        }

        // Encode using prost
        msg.encode_to_vec()
            .map_err(|_| NexoError::Encoding {
                details: "prost encode failed",
            })
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
pub fn encode<M: Message>(msg: &M) -> Result<Vec<u8>, NexoError> {
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
pub fn decode<M: Message>(bytes: &[u8]) -> Result<M, NexoError> {
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
        // This test will need actual message construction
        // once we have valid proto message builders
        // For now, just verify the API compiles
        let _codec = ProstCodec;
        // TODO: Construct actual Casp001Document and test encoding
    }

    #[test]
    fn test_encode_oversized_message() {
        // Test that encoding fails for messages exceeding MAX_MESSAGE_SIZE
        // This requires constructing a message > 4MB
        // TODO: Implement with actual oversized message
    }

    #[test]
    fn test_decode_valid_bytes() {
        // Test decoding valid protobuf bytes
        // TODO: Implement with actual encoded bytes
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
        // Test that convenience function works
        // TODO: Implement with actual message
    }

    #[test]
    fn test_convenience_decode_function() {
        // Test that convenience function works
        // TODO: Implement with actual bytes
    }

    #[test]
    fn test_codec_trait_can_be_mocked() {
        // Verify Codec trait can be implemented for testing
        #[derive(Debug)]
        struct MockCodec;

        impl<M: Message> Codec<M> for MockCodec {
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
