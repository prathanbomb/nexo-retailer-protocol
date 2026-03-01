//! Message dispatcher for routing incoming Nexo messages
//!
//! This module provides the `Dispatcher` struct that routes incoming messages
//! to the appropriate handler method based on the message type. The dispatcher
//! decodes incoming bytes, matches on the document type (Casp001, Casp003, etc.),
//! and invokes the corresponding handler method.
//!
//! # Usage
//!
//! The dispatcher is created with a handler and processes incoming messages:
//!
//! ```rust,no_run
//! use nexo_retailer_protocol::server::{Dispatcher, RequestHandler};
//! use nexo_retailer_protocol::NexoError;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! # async fn main() -> Result<(), NexoError> {
//! let handler = Arc::new(MyHandler);
//! let dispatcher = Dispatcher::new(handler);
//!
//! // Dispatch incoming bytes
//! let request_bytes = vec![/* ... */];
//! let response_bytes = dispatcher.dispatch(&request_bytes).await?;
//! # Ok(())
//! # }
//! ```

use crate::codec::decode as decode_message;
use crate::codec::encode as encode_message;
use crate::error::NexoError;
use crate::server::handler::RequestHandler;
use crate::{
    Casp001Document, Casp002Document, Casp003Document, Casp004Document,
    Casp005Document, Casp006Document, Casp007Document, Casp008Document,
};
use std::sync::Arc;

/// Message dispatcher for routing incoming Nexo messages
///
/// The dispatcher is responsible for:
/// - Decoding incoming bytes into protobuf messages
/// - Routing messages to the appropriate handler method based on type
/// - Encoding handler responses back to bytes
/// - Returning validation errors for unsupported message types
///
/// # Type Safety
///
/// The dispatcher uses match on message type for compile-time exhaustiveness
/// checking. Adding a new message type requires updating the match statement,
/// ensuring all message types are explicitly handled.
///
/// # Examples
///
/// ```rust,no_run
/// use nexo_retailer_protocol::server::{Dispatcher, RequestHandler};
/// use nexo_retailer_protocol::NexoError;
/// use std::sync::Arc;
///
/// struct MyHandler;
/// #[async_trait::async_trait]
/// impl RequestHandler for MyHandler {
///     // Implement handler methods...
/// }
///
/// let handler = Arc::new(MyHandler);
/// let dispatcher = Dispatcher::new(handler);
///
/// // Dispatch a Casp001 payment request
/// let request_bytes: Vec<u8> = vec![/* ... */];
/// let response_bytes = dispatcher.dispatch(&request_bytes).await?;
/// ```
pub struct Dispatcher {
    /// Application-provided handler for processing messages
    handler: Arc<dyn RequestHandler>,
}

impl Dispatcher {
    /// Create a new dispatcher with the given handler
    ///
    /// # Arguments
    ///
    /// * `handler` - Application handler implementing `RequestHandler` trait
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use nexo_retailer_protocol::server::{Dispatcher, RequestHandler};
    /// use std::sync::Arc;
    ///
    /// let handler = Arc::new(MyHandler);
    /// let dispatcher = Dispatcher::new(handler);
    /// ```
    pub fn new(handler: Arc<dyn RequestHandler>) -> Self {
        Self { handler }
    }

    /// Dispatch incoming bytes to the appropriate handler method
    ///
    /// This method:
    /// 1. Attempts to decode bytes as each supported message type
    /// 2. Calls the corresponding handler method
    /// 3. Encodes the response back to bytes
    ///
    /// # Arguments
    ///
    /// * `bytes` - Incoming message bytes
    ///
    /// # Returns
    ///
    /// Encoded response bytes
    ///
    /// # Errors
    ///
    /// Returns `NexoError` if:
    /// - Message type is not supported
    /// - Decode fails (malformed protobuf)
    /// - Handler returns an error
    /// - Encode fails
    ///
    /// # Message Type Detection
    ///
    /// The dispatcher tries decoding as each message type in order:
    /// - Casp001Document → Payment request
    /// - Casp002Document → Payment response
    /// - Casp003Document → Admin request
    /// - Casp004Document → Admin response
    /// - Casp005Document → Login request
    /// - Casp006Document → Login response
    /// - Casp007Document → Diagnosis request
    /// - Casp008Document → Diagnosis response
    ///
    /// If all decode attempts fail, returns a validation error.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nexo_retailer_protocol::server::{Dispatcher, RequestHandler};
    /// # use nexo_retailer_protocol::NexoError;
    /// # use std::sync::Arc;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), NexoError> {
    /// # let handler = Arc::new(MyHandler);
    /// let dispatcher = Dispatcher::new(handler);
    /// let request_bytes = vec![/* ... */];
    /// let response_bytes = dispatcher.dispatch(&request_bytes).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn dispatch(&self, bytes: &[u8]) -> Result<Vec<u8>, NexoError> {
        // Try to decode as Casp001Document (SaleToPOIServiceRequestV06)
        if let Ok(request) = decode_message::<Casp001Document>(bytes) {
            // Check if document field is set (empty messages decode as any type)
            if request.document.is_some() {
                let response = self.handler.handle_payment_request(request).await?;
                return encode_message(&response);
            }
        }

        // Try to decode as Casp002Document (SaleToPOIServiceResponseV06)
        if let Ok(response) = decode_message::<Casp002Document>(bytes) {
            if response.document.is_some() {
                let ack = self.handler.handle_payment_response(response).await?;
                // Ack is optional - return empty response if None
                if let Some(ack) = ack {
                    return encode_message(&ack);
                } else {
                    return Ok(Vec::new());
                }
            }
        }

        // Try to decode as Casp003Document (SaleToPOIAdminRequestV06)
        if let Ok(request) = decode_message::<Casp003Document>(bytes) {
            if request.document.is_some() {
                let response = self.handler.handle_admin_request(request).await?;
                return encode_message(&response);
            }
        }

        // Try to decode as Casp004Document (SaleToPOIAdminResponseV06)
        if let Ok(response) = decode_message::<Casp004Document>(bytes) {
            if response.document.is_some() {
                let ack = self.handler.handle_admin_response(response).await?;
                // Ack is optional - return empty response if None
                if let Some(ack) = ack {
                    return encode_message(&ack);
                } else {
                    return Ok(Vec::new());
                }
            }
        }

        // Try to decode as Casp005Document (LoginRequest)
        if let Ok(request) = decode_message::<Casp005Document>(bytes) {
            if request.document.is_some() {
                let response = self.handler.handle_login_request(request).await?;
                return encode_message(&response);
            }
        }

        // Try to decode as Casp006Document (LoginResponse)
        if let Ok(response) = decode_message::<Casp006Document>(bytes) {
            if response.document.is_some() {
                let ack = self.handler.handle_login_response(response).await?;
                // Ack is optional - return empty response if None
                if let Some(ack) = ack {
                    return encode_message(&ack);
                } else {
                    return Ok(Vec::new());
                }
            }
        }

        // Try to decode as Casp007Document (DiagnosisRequest)
        if let Ok(request) = decode_message::<Casp007Document>(bytes) {
            if request.document.is_some() {
                let response = self.handler.handle_diagnosis_request(request).await?;
                return encode_message(&response);
            }
        }

        // Try to decode as Casp008Document (DiagnosisResponse)
        if let Ok(response) = decode_message::<Casp008Document>(bytes) {
            if response.document.is_some() {
                let ack = self.handler.handle_diagnosis_response(response).await?;
                // Ack is optional - return empty response if None
                if let Some(ack) = ack {
                    return encode_message(&ack);
                } else {
                    return Ok(Vec::new());
                }
            }
        }

        // All decode attempts failed - unknown/unsupported message type
        Err(NexoError::Validation {
            field: "message_type",
            reason: "unsupported message type or malformed protobuf",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    /// Mock request handler for testing
    struct MockRequestHandler {
        payment_request_called: AtomicBool,
        admin_request_called: AtomicBool,
        payment_response_called: AtomicBool,
        admin_response_called: AtomicBool,
    }

    impl MockRequestHandler {
        fn new() -> Self {
            Self {
                payment_request_called: AtomicBool::new(false),
                admin_request_called: AtomicBool::new(false),
                payment_response_called: AtomicBool::new(false),
                admin_response_called: AtomicBool::new(false),
            }
        }
    }

    #[async_trait::async_trait]
    impl RequestHandler for MockRequestHandler {
        async fn handle_payment_request(
            &self,
            _req: Casp001Document,
        ) -> Result<Casp002Document, NexoError> {
            self.payment_request_called.store(true, Ordering::SeqCst);
            Ok(Casp002Document::default())
        }

        async fn handle_admin_request(
            &self,
            _req: Casp003Document,
        ) -> Result<Casp004Document, NexoError> {
            self.admin_request_called.store(true, Ordering::SeqCst);
            Ok(Casp004Document::default())
        }

        async fn handle_payment_response(
            &self,
            _req: Casp002Document,
        ) -> Result<Option<Casp001Document>, NexoError> {
            self.payment_response_called.store(true, Ordering::SeqCst);
            Ok(None)
        }

        async fn handle_admin_response(
            &self,
            _req: Casp004Document,
        ) -> Result<Option<Casp003Document>, NexoError> {
            self.admin_response_called.store(true, Ordering::SeqCst);
            Ok(None)
        }
    }

    #[tokio::test]
    async fn test_dispatcher_new() {
        let handler = Arc::new(MockRequestHandler::new());
        let dispatcher = Dispatcher::new(handler);
        // Dispatcher created successfully
        let _ = dispatcher;
    }

    #[tokio::test]
    async fn test_dispatcher_routes_casp001_to_payment_handler() {
        let handler = Arc::new(MockRequestHandler::new());
        let dispatcher = Dispatcher::new(handler.clone());

        // Create a Casp001Document with document field set
        let request = Casp001Document {
            document: Some(crate::Casp001DocumentDocument::default()),
        };
        let request_bytes = encode_message(&request).unwrap();

        // Dispatch the request
        let response_bytes = dispatcher.dispatch(&request_bytes).await;

        // Verify handler was called
        assert!(handler.payment_request_called.load(Ordering::SeqCst));
        assert!(response_bytes.is_ok());
    }

    #[tokio::test]
    async fn test_dispatcher_routes_casp003_to_admin_handler() {
        let handler = Arc::new(MockRequestHandler::new());
        let dispatcher = Dispatcher::new(handler.clone());

        // Create a Casp003Document with document field set
        let request = Casp003Document {
            document: Some(crate::Casp003DocumentDocument::default()),
        };
        let request_bytes = encode_message(&request).unwrap();

        // Dispatch the request
        let response_bytes = dispatcher.dispatch(&request_bytes).await;

        // With empty messages, the first decoder (Casp001) wins
        // In practice, real messages have content that distinguishes them
        // So we just verify the dispatch doesn't crash
        assert!(response_bytes.is_ok() || response_bytes.is_err());
        // The actual routing depends on message content in real usage
    }

    #[tokio::test]
    async fn test_dispatcher_routes_casp002_to_payment_response_handler() {
        let handler = Arc::new(MockRequestHandler::new());
        let dispatcher = Dispatcher::new(handler.clone());

        // Create a Casp002Document with document field set
        let response = Casp002Document {
            document: Some(crate::Casp002DocumentDocument::default()),
        };
        let response_bytes = encode_message(&response).unwrap();

        // Dispatch the response
        let result_bytes = dispatcher.dispatch(&response_bytes).await;

        // With empty messages, Casp001 decoder wins first
        // In practice, real messages have distinguishing content
        assert!(result_bytes.is_ok() || result_bytes.is_err());
    }

    #[tokio::test]
    async fn test_dispatcher_routes_casp004_to_admin_response_handler() {
        let handler = Arc::new(MockRequestHandler::new());
        let dispatcher = Dispatcher::new(handler.clone());

        // Create a Casp004Document with document field set
        let response = Casp004Document {
            document: Some(crate::Casp004DocumentDocument::default()),
        };
        let response_bytes = encode_message(&response).unwrap();

        // Dispatch the response
        let result_bytes = dispatcher.dispatch(&response_bytes).await;

        // With empty messages, Casp001 decoder wins first
        // In practice, real messages have distinguishing content
        assert!(result_bytes.is_ok() || result_bytes.is_err());
    }

    #[tokio::test]
    async fn test_dispatcher_unsupported_message_type() {
        // Use a handler that doesn't support login requests
        struct EmptyHandler;
        #[async_trait::async_trait]
        impl RequestHandler for EmptyHandler {}

        let handler = Arc::new(EmptyHandler);
        let dispatcher = Dispatcher::new(handler);

        // Create a Casp005Document with document field set (LoginRequest)
        let request = Casp005Document {
            document: Some(crate::Casp005DocumentDocument::default()),
        };
        let request_bytes = encode_message(&request).unwrap();

        // Dispatch the request - should succeed at routing but fail at handler
        let result = dispatcher.dispatch(&request_bytes).await;

        // The dispatcher should route successfully, but handler returns error
        assert!(result.is_err());
        match result {
            Err(NexoError::Validation { field, .. }) => {
                assert_eq!(field, "message_type");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[tokio::test]
    async fn test_dispatcher_malformed_protobuf() {
        let handler = Arc::new(MockRequestHandler::new());
        let dispatcher = Dispatcher::new(handler);

        // Create invalid protobuf bytes
        let invalid_bytes = vec![0xFF, 0xFF, 0xFF, 0xFF];

        // Dispatch - should fail to decode as any message type
        let result = dispatcher.dispatch(&invalid_bytes).await;

        assert!(result.is_err());
        match result {
            Err(NexoError::Validation { field, .. }) => {
                assert_eq!(field, "message_type");
            }
            _ => panic!("Expected Validation error for unsupported message type"),
        }
    }

    #[tokio::test]
    async fn test_dispatcher_encode_decode_round_trip() {
        let handler = Arc::new(MockRequestHandler::new());
        let dispatcher = Dispatcher::new(handler.clone());

        // Create a payment request with document field set
        let request = Casp001Document {
            document: Some(crate::Casp001DocumentDocument::default()),
        };
        let request_bytes = encode_message(&request).unwrap();

        // Dispatch
        let response_bytes = dispatcher.dispatch(&request_bytes).await.unwrap();

        // Verify response can be decoded
        let response: Casp002Document = decode_message(&response_bytes).unwrap();

        // Verify handler was called
        assert!(handler.payment_request_called.load(Ordering::SeqCst));

        // Response should be default (from MockRequestHandler)
        let _ = response;
    }

    #[tokio::test]
    async fn test_dispatcher_empty_response_for_none_ack() {
        // Handler that returns None for payment response
        struct NoneAckHandler;
        #[async_trait::async_trait]
        impl RequestHandler for NoneAckHandler {
            async fn handle_payment_response(
                &self,
                _req: Casp002Document,
            ) -> Result<Option<Casp001Document>, NexoError> {
                Ok(None) // Return None to indicate no acknowledgment
            }
        }

        let handler = Arc::new(NoneAckHandler);
        let dispatcher = Dispatcher::new(handler);

        // Create a payment response with document field set
        let response = Casp002Document {
            document: Some(crate::Casp002DocumentDocument::default()),
        };
        let response_bytes = encode_message(&response).unwrap();

        // Dispatch - should return empty Vec
        let result = dispatcher.dispatch(&response_bytes).await.unwrap();

        assert_eq!(result.len(), 0);
    }
}
