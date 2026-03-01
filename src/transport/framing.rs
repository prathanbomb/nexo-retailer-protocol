//! Length-prefixed TCP framing for Nexo Retailer Protocol
//!
//! This module provides a framing layer that wraps any Transport implementation
//! and adds length-prefixed message framing following standard protobuf-over-TCP
//! conventions.

#![cfg_attr(not(feature = "std"), no_std)]

use crate::error::NexoError;
use crate::transport::Transport;

// Import Vec for alloc builds
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;

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
                details: "Message size exceeds maximum frame size (4MB)",
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
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub async fn recv_message<M: prost::Message + Default>(&mut self) -> Result<M, T::Error> {
        // Read length prefix (4-byte big-endian)
        let mut length_prefix_bytes = [0u8; LENGTH_PREFIX_SIZE];
        self.read_exact(&mut length_prefix_bytes).await?;

        let length = u32::from_be_bytes(length_prefix_bytes) as usize;

        // Validate message size
        if length > MAX_FRAME_SIZE {
            return Err(NexoError::Decoding {
                details: "Message size exceeds maximum frame size (4MB)",
            }
            .into());
        }

        // Read message body
        let mut buffer = vec![0u8; length];
        self.read_exact(&mut buffer).await?;

        // Decode message
        let msg = M::decode(&*buffer).map_err(|_| NexoError::Decoding {
            details: "Failed to decode protobuf message",
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
                    details: "Write returned 0 bytes (connection closed)",
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
                    details: "Read returned 0 bytes (unexpected EOF)",
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

    // Import futures executor for async test execution
    use futures_executor::block_on;

    // Import prost::Message trait for encode/decode
    use prost::Message;

    // Import message types for testing
    use crate::{Header4, ActiveCurrencyAndAmount};

    #[test]
    fn test_framing_module_exists() {
        // Simple test to verify the module is compiled
        assert_eq!(LENGTH_PREFIX_SIZE, 4);
        assert_eq!(MAX_FRAME_SIZE, 4 * 1024 * 1024);
    }

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
                    details: "Not connected",
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
                    details: "Not connected",
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
        // First, test that prost encoding/decoding works
        let original = ActiveCurrencyAndAmount {
            ccy: "USD".to_string(),
            units: 100,
            nanos: 500000000,
        };

        // Test prost encode/decode directly
        let encoded = original.encode_to_vec();
        let decoded_direct = ActiveCurrencyAndAmount::decode(&*encoded);
        assert!(decoded_direct.is_ok(), "Direct prost decode should work: {:?}", decoded_direct);

        // Test round-trip encoding/decoding
        let mut transport = MockTransport::new();
        let mut framed = FramedTransport::new(transport);
        // Test round-trip encoding/decoding
        let mut transport = MockTransport::new();
        let mut framed = FramedTransport::new(transport);

        // Use a simple test message that's easy to encode/decode
        // ActiveCurrencyAndAmount is a simple message with required fields
        let original = ActiveCurrencyAndAmount {
            ccy: "USD".to_string(),
            units: 100,
            nanos: 500000000,
        };

        // Verify direct encode/decode works
        let direct_encoded = original.encode_to_vec();
        println!("Direct encoded length: {}", direct_encoded.len());
        let direct_decoded = ActiveCurrencyAndAmount::decode(&*direct_encoded);
        println!("Direct decode result: {:?}", direct_decoded.is_ok());
        assert!(direct_decoded.is_ok(), "Direct decode should work");

        // Send message (write to mock transport)
        let send_result = block_on(framed.send_message(&original));
        assert!(send_result.is_ok(), "send_message failed: {:?}", send_result);

        // Get written data and set it as read data
        let written = framed.inner().get_written_data();
        println!("Written data length: {}", written.len());
        println!("Length prefix bytes: {:?}", &written[..4]);
        println!("Message body length: {}", written.len() - 4);
        println!("Message body: {:?}", &written[4..]);
        assert!(written.len() > LENGTH_PREFIX_SIZE, "Should have written data with length prefix");

        framed.inner_mut().set_read_data(written);

        // Receive message
        println!("About to call recv_message...");
        let received_result: Result<ActiveCurrencyAndAmount, _> = block_on(framed.recv_message());
        assert!(received_result.is_ok(), "recv_message failed: {:?}", received_result);
        let received = received_result.unwrap();

        assert_eq!(received.ccy, original.ccy);
        assert_eq!(received.units, original.units);
        assert_eq!(received.nanos, original.nanos);
    }

    #[test]
    fn test_length_prefix_big_endian() {
        let mut transport = MockTransport::new();
        let mut framed = FramedTransport::new(transport);

        // Create a message with known size
        let msg = ActiveCurrencyAndAmount {
            ccy: "USD".to_string(),
            units: 100,
            nanos: 0,
        };

        block_on(framed.send_message(&msg)).unwrap();

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

        // Create a simple message (not empty, but minimal)
        let msg = ActiveCurrencyAndAmount {
            ccy: "USD".to_string(),
            units: 0,
            nanos: 0,
        };

        // Send and receive
        block_on(framed.send_message(&msg)).unwrap();

        let written = framed.inner().get_written_data();
        framed.inner_mut().set_read_data(written);

        let received: ActiveCurrencyAndAmount =
            block_on(framed.recv_message()).unwrap();

        assert_eq!(received.ccy, msg.ccy);
        assert_eq!(received.units, msg.units);
        assert_eq!(received.nanos, msg.nanos);
    }

    #[test]
    fn test_oversized_message_rejected() {
        let mut transport = MockTransport::new();
        let mut framed = FramedTransport::new(transport);

        // Create a message that exceeds MAX_FRAME_SIZE
        // We can't actually create a message this large with Header4, so we'll
        // test this by manually creating oversized encoded data
        let oversized_data = vec![0u8; MAX_FRAME_SIZE + 1];

        // Try to encode it manually (simulating what send_message does internally)
        let length_prefix = (oversized_data.len() as u32).to_be_bytes();
        framed.inner_mut().set_read_data(length_prefix.to_vec());

        // recv_message should fail when reading the length prefix
        let result: Result<Header4, _> = block_on(framed.recv_message());
        assert!(result.is_err());

        match result {
            Err(NexoError::Decoding { details }) => {
                assert!(details.contains("exceeds maximum frame size"));
            }
            _ => panic!("Expected Decoding error, got {:?}", result),
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
        let result: Result<Header4, _> =
            block_on(framed.recv_message());
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
        let msg = ActiveCurrencyAndAmount {
            ccy: "EUR".to_string(),
            units: 200,
            nanos: 250000000,
        };

        // Send and receive with partial reads
        block_on(framed.send_message(&msg)).unwrap();

        let written = framed.inner().get_written_data();
        framed.inner_mut().set_read_data(written);

        let received: ActiveCurrencyAndAmount =
            block_on(framed.recv_message()).unwrap();

        assert_eq!(received.ccy, msg.ccy);
        assert_eq!(received.units, msg.units);
        assert_eq!(received.nanos, msg.nanos);
    }

    #[test]
    fn test_multiple_messages_sequential() {
        let mut transport = MockTransport::new();
        let mut framed = FramedTransport::new(transport);

        // Send multiple messages
        let msg1 = ActiveCurrencyAndAmount {
            ccy: "USD".to_string(),
            units: 100,
            nanos: 0,
        };

        let msg2 = ActiveCurrencyAndAmount {
            ccy: "EUR".to_string(),
            units: 200,
            nanos: 0,
        };

        block_on(framed.send_message(&msg1)).unwrap();
        block_on(framed.send_message(&msg2)).unwrap();

        // Read them back
        let written = framed.inner().get_written_data();
        framed.inner_mut().set_read_data(written);

        let received1: ActiveCurrencyAndAmount =
            block_on(framed.recv_message()).unwrap();
        let received2: ActiveCurrencyAndAmount =
            block_on(framed.recv_message()).unwrap();

        assert_eq!(received1.ccy, msg1.ccy);
        assert_eq!(received1.units, msg1.units);
        assert_eq!(received2.ccy, msg2.ccy);
        assert_eq!(received2.units, msg2.units);
    }

    #[test]
    fn test_zero_length_prefix() {
        let mut transport = MockTransport::new();
        let mut framed = FramedTransport::new(transport);

        // Set up read data with zero-length prefix
        let data = [0u8; LENGTH_PREFIX_SIZE].to_vec();
        framed.inner_mut().set_read_data(data);

        // Should successfully receive an empty message
        let result: Result<ActiveCurrencyAndAmount, _> =
            block_on(framed.recv_message());

        // prost::Message::decode on empty bytes should return default message
        assert!(result.is_ok(), "Zero-length message should decode successfully");
        let received = result.unwrap();
        assert_eq!(received.ccy, "");
        assert_eq!(received.units, 0);
        assert_eq!(received.nanos, 0);
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
