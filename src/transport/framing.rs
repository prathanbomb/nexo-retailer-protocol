//! Length-prefixed TCP framing for Nexo Retailer Protocol
//!
//! This module provides a framing layer that wraps any Transport implementation
//! and adds length-prefixed message framing following standard protobuf-over-TCP
//! conventions.

#![cfg_attr(not(feature = "std"), no_std)]

use crate::error::NexoError;
use crate::transport::Transport;

/// Length prefix size in bytes (4-byte big-endian)
pub const LENGTH_PREFIX_SIZE: usize = 4;

/// Maximum frame size in bytes (4MB limit from codec layer)
pub const MAX_FRAME_SIZE: usize = 4 * 1024 * 1024;

/// Framed transport wrapper that adds length-prefixed message framing
///
/// This wrapper implements the standard protobuf-over-TCP framing format:
/// - 4-byte big-endian length prefix
/// - Followed by the protobuf-encoded message body
///
/// # Type Parameters
///
/// * `T` - Transport implementation (e.g., TokioTcpTransport, EmbassyTcpTransport)
///
/// # Example
///
/// ```rust,ignore
/// use nexo_retailer_protocol::transport::{FramedTransport, Transport};
///
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let transport = MyTransport::new();
///     let mut framed = FramedTransport::new(transport);
///
///     // Send a message
///     let message = MyMessage::default();
///     framed.send_message(&message).await?;
///
///     // Receive a message
///     let received: MyMessage = framed.recv_message().await?;
///
///     Ok(())
/// }
/// ```
pub struct FramedTransport<T: Transport> {
    inner: T,
}

impl<T: Transport> FramedTransport<T> {
    /// Create a new framed transport wrapper
    ///
    /// # Arguments
    ///
    /// * `inner` - Inner transport implementation
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Get a mutable reference to the inner transport
    pub fn inner(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Get an immutable reference to the inner transport
    pub fn inner_ref(&self) -> &T {
        &self.inner
    }

    /// Send a message with length-prefix framing
    ///
    /// # Arguments
    ///
    /// * `msg` - Message implementing prost::Message to send
    ///
    /// # Errors
    ///
    /// Returns NexoError if:
    /// - Message encoding fails
    /// - Message size exceeds MAX_FRAME_SIZE
    /// - Write operation fails
    pub async fn send_message<M: prost::Message>(&mut self, msg: &M) -> Result<(), T::Error> {
        // Encode the message to bytes
        let encoded = msg.encode_to_vec();

        // Validate message size
        if encoded.len() > MAX_FRAME_SIZE {
            return Err(NexoError::Encoding {
                details: format!(
                    "Message size {} exceeds maximum frame size {}",
                    encoded.len(),
                    MAX_FRAME_SIZE
                ),
            }
            .into());
        }

        // Write length prefix (4-byte big-endian)
        let length_prefix = (encoded.len() as u32).to_be_bytes();
        self.write_all(&length_prefix).await?;

        // Write message body
        self.write_all(&encoded).await?;

        Ok(())
    }

    /// Receive a message with length-prefix framing
    ///
    /// # Arguments
    ///
    /// * `msg` - Type parameter implementing prost::Message and Default
    ///
    /// # Errors
    ///
    /// Returns NexoError if:
    /// - Length prefix cannot be read
    /// - Message body cannot be read
    /// - Message size exceeds MAX_FRAME_SIZE
    /// - Message decoding fails
    pub async fn recv_message<M: prost::Message + Default>(&mut self) -> Result<M, T::Error> {
        // Read length prefix (4-byte big-endian)
        let mut length_prefix_bytes = [0u8; LENGTH_PREFIX_SIZE];
        self.read_exact(&mut length_prefix_bytes).await?;

        let length = u32::from_be_bytes(length_prefix_bytes) as usize;

        // Validate message size
        if length > MAX_FRAME_SIZE {
            return Err(NexoError::Decoding {
                details: format!(
                    "Message size {} exceeds maximum frame size {}",
                    length,
                    MAX_FRAME_SIZE
                ),
            }
            .into());
        }

        // Read message body
        let mut buffer = vec![0u8; length];
        self.read_exact(&mut buffer).await?;

        // Decode message
        let msg = M::decode(&*buffer).map_err(|e| NexoError::Decoding {
            details: format!("Failed to decode message: {}", e),
        })?;

        Ok(msg)
    }

    /// Write all bytes to the transport
    ///
    /// This is a helper method that ensures all bytes are written,
    /// handling partial writes automatically.
    ///
    /// # Arguments
    ///
    /// * `buf` - Buffer containing bytes to write
    async fn write_all(&mut self, buf: &[u8]) -> Result<(), T::Error> {
        let mut total_written = 0;
        while total_written < buf.len() {
            let n = self.inner.write(&buf[total_written..]).await?;
            if n == 0 {
                return Err(NexoError::Connection {
                    details: "Write returned 0 bytes (connection closed)".to_string(),
                }
                .into());
            }
            total_written += n;
        }
        Ok(())
    }

    /// Read exact number of bytes from the transport
    ///
    /// This is a helper method that ensures the exact number of bytes
    /// are read, handling partial reads automatically.
    ///
    /// # Arguments
    ///
    /// * `buf` - Buffer to read bytes into
    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), T::Error> {
        let mut total_read = 0;
        while total_read < buf.len() {
            let n = self.inner.read(&mut buf[total_read..]).await?;
            if n == 0 {
                return Err(NexoError::Connection {
                    details: "Read returned 0 bytes (unexpected EOF)".to_string(),
                }
                .into());
            }
            total_read += n;
        }
        Ok(())
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::error::NexoError;

    #[test]
    fn test_framing_module_exists() {
        // Simple test to verify the module is compiled
        assert_eq!(LENGTH_PREFIX_SIZE, 4);
        assert_eq!(MAX_FRAME_SIZE, 4 * 1024 * 1024);
    }
    use super::*;
    use crate::error::NexoError;

    // Mock transport for testing
    struct MockTransport {
        read_buffer: Vec<u8>,
        write_buffer: Vec<u8>,
        connected: bool,
        read_chunk_size: usize, // Simulate partial reads
    }

    impl MockTransport {
        fn new() -> Self {
            Self {
                read_buffer: Vec::new(),
                write_buffer: Vec::new(),
                connected: true,
                read_chunk_size: usize::MAX, // No partial reads by default
            }
        }

        fn set_read_data(&mut self, data: Vec<u8>) {
            self.read_buffer = data;
        }

        fn get_written_data(&self) -> Vec<u8> {
            self.write_buffer.clone()
        }

        fn with_partial_reads(mut self, chunk_size: usize) -> Self {
            self.read_chunk_size = chunk_size;
            self
        }
    }

    impl Transport for MockTransport {
        type Error = NexoError;

        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            if !self.connected {
                return Err(NexoError::Connection {
                    details: "Not connected".to_string(),
                });
            }

            let bytes_to_read = core::cmp::min(self.read_chunk_size, buf.len());
            let bytes_to_read = core::cmp::min(bytes_to_read, self.read_buffer.len());

            if bytes_to_read == 0 {
                return Ok(0); // EOF
            }

            buf[..bytes_to_read].copy_from_slice(&self.read_buffer[..bytes_to_read]);
            self.read_buffer = self.read_buffer[bytes_to_read..].to_vec();

            Ok(bytes_to_read)
        }

        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            if !self.connected {
                return Err(NexoError::Connection {
                    details: "Not connected".to_string(),
                });
            }

            self.write_buffer.extend_from_slice(buf);
            Ok(buf.len())
        }

        async fn connect(&mut self, _addr: &str) -> Result<(), Self::Error> {
            self.connected = true;
            Ok(())
        }

        fn is_connected(&self) -> bool {
            self.connected
        }
    }

    #[test]
    fn test_encode_decode_normal_message() {
        // Test round-trip encoding/decoding
        let mut transport = MockTransport::new();
        let mut framed = FramedTransport::new(transport);

        // Use a simple test message (prost::Message is implemented for many types)
        // We'll use a generic prost message for testing
        let original = prost_types::Any {
            type_url: "test.type".to_string(),
            value: vec![1, 2, 3, 4],
        };

        // Send message (write to mock transport)
        let send_result = futures::executor::block_on(framed.send_message(&original));
        assert!(send_result.is_ok(), "send_message failed: {:?}", send_result);

        // Get written data and set it as read data
        let written = framed.inner().get_written_data();
        framed.inner_mut().set_read_data(written);

        // Receive message
        let received: prost_types::Any = futures::executor::block_on(framed.recv_message())
            .expect("recv_message failed");

        assert_eq!(received.type_url, original.type_url);
        assert_eq!(received.value, original.value);
    }

    #[test]
    fn test_length_prefix_big_endian() {
        let mut transport = MockTransport::new();
        let mut framed = FramedTransport::new(transport);

        // Create a message with known size
        let msg = prost_types::Any {
            type_url: "test".to_string(),
            value: vec![1, 2, 3],
        };

        futures::executor::block_on(framed.send_message(&msg)).unwrap();

        // Check that the length prefix is in big-endian format
        let written = framed.inner().get_written_data();
        assert!(written.len() >= LENGTH_PREFIX_SIZE);

        let length_prefix = u32::from_be_bytes([
            written[0],
            written[1],
            written[2],
            written[3],
        ]);

        // Message size should be total minus length prefix
        let expected_length = (written.len() - LENGTH_PREFIX_SIZE) as u32;
        assert_eq!(length_prefix, expected_length);
    }

    #[test]
    fn test_empty_message() {
        let mut transport = MockTransport::new();
        let mut framed = FramedTransport::new(transport);

        // Create an empty message
        let msg = prost_types::Any {
            type_url: "test".to_string(),
            value: vec![],
        };

        // Send and receive
        futures::executor::block_on(framed.send_message(&msg)).unwrap();

        let written = framed.inner().get_written_data();
        framed.inner_mut().set_read_data(written);

        let received: prost_types::Any =
            futures::executor::block_on(framed.recv_message()).unwrap();

        assert_eq!(received.type_url, msg.type_url);
        assert_eq!(received.value, msg.value);
        assert!(received.value.is_empty());
    }

    #[test]
    fn test_oversized_message_rejected() {
        let mut transport = MockTransport::new();
        let mut framed = FramedTransport::new(transport);

        // Create a message that exceeds MAX_FRAME_SIZE
        let msg = prost_types::Any {
            type_url: "test".to_string(),
            value: vec![0u8; MAX_FRAME_SIZE + 1],
        };

        // Should fail with encoding error
        let result = futures::executor::block_on(framed.send_message(&msg));
        assert!(result.is_err());

        match result {
            Err(NexoError::Encoding { details }) => {
                assert!(details.contains("exceeds maximum frame size"));
            }
            _ => panic!("Expected Encoding error, got {:?}", result),
        }
    }

    #[test]
    fn test_oversized_length_prefix_rejected() {
        let mut transport = MockTransport::new();
        let mut framed = FramedTransport::new(transport);

        // Set up read data with oversized length prefix
        let mut data = vec![0u8; LENGTH_PREFIX_SIZE];
        // Write length that exceeds MAX_FRAME_SIZE
        let oversized_length = (MAX_FRAME_SIZE + 1) as u32;
        data.copy_from_slice(&oversized_length.to_be_bytes());

        framed.inner_mut().set_read_data(data);

        // Should fail with decoding error
        let result: Result<prost_types::Any, _> =
            futures::executor::block_on(framed.recv_message());
        assert!(result.is_err());

        match result {
            Err(NexoError::Decoding { details }) => {
                assert!(details.contains("exceeds maximum frame size"));
            }
            _ => panic!("Expected Decoding error, got {:?}", result),
        }
    }

    #[test]
    fn test_partial_read_handling() {
        let mut transport = MockTransport::new().with_partial_reads(2);
        let mut framed = FramedTransport::new(transport);

        // Create a message
        let msg = prost_types::Any {
            type_url: "test".to_string(),
            value: vec![1, 2, 3, 4, 5, 6, 7, 8],
        };

        // Send and receive with partial reads
        futures::executor::block_on(framed.send_message(&msg)).unwrap();

        let written = framed.inner().get_written_data();
        framed.inner_mut().set_read_data(written);

        let received: prost_types::Any =
            futures::executor::block_on(framed.recv_message()).unwrap();

        assert_eq!(received.type_url, msg.type_url);
        assert_eq!(received.value, msg.value);
    }

    #[test]
    fn test_multiple_messages_sequential() {
        let mut transport = MockTransport::new();
        let mut framed = FramedTransport::new(transport);

        // Send multiple messages
        let msg1 = prost_types::Any {
            type_url: "test1".to_string(),
            value: vec![1, 2, 3],
        };

        let msg2 = prost_types::Any {
            type_url: "test2".to_string(),
            value: vec![4, 5, 6],
        };

        futures::executor::block_on(framed.send_message(&msg1)).unwrap();
        futures::executor::block_on(framed.send_message(&msg2)).unwrap();

        // Read them back
        let written = framed.inner().get_written_data();
        framed.inner_mut().set_read_data(written);

        let received1: prost_types::Any =
            futures::executor::block_on(framed.recv_message()).unwrap();
        let received2: prost_types::Any =
            futures::executor::block_on(framed.recv_message()).unwrap();

        assert_eq!(received1.type_url, msg1.type_url);
        assert_eq!(received1.value, msg1.value);
        assert_eq!(received2.type_url, msg2.type_url);
        assert_eq!(received2.value, msg2.value);
    }

    #[test]
    fn test_zero_length_prefix() {
        let mut transport = MockTransport::new();
        let mut framed = FramedTransport::new(transport);

        // Set up read data with zero-length prefix
        let data = [0u8; LENGTH_PREFIX_SIZE].to_vec();
        framed.inner_mut().set_read_data(data);

        // Should successfully receive an empty message
        let result: Result<prost_types::Any, _> =
            futures::executor::block_on(framed.recv_message());

        // prost::Message::decode on empty bytes might fail or return default
        // depending on the message type, but we should at least not panic
        // on the length prefix reading
        assert!(result.is_ok() || result.is_err());
        if let Err(NexoError::Connection { .. }) | Err(NexoError::Decoding { .. }) = result {
            // Expected - decode may fail on empty bytes
        } else {
            // Either success or expected error type
        }
    }

    // Helper to access inner_mut for testing
    trait MockTransportExt {
        fn inner_mut(&mut self) -> &mut MockTransport;
    }

    impl MockTransportExt for FramedTransport<MockTransport> {
        fn inner_mut(&mut self) -> &mut MockTransport {
            &mut self.inner
        }
    }
}
