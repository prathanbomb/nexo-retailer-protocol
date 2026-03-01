//! Request handler trait for Nexo server
//!
//! This module defines the `RequestHandler` trait that applications implement
//! to receive and process incoming Nexo messages. The trait provides async methods
//! for common message types (PaymentRequest, AdminRequest, etc.) and returns
//! appropriate response types.
//!
//! # Usage
//!
//! Applications implement the `RequestHandler` trait to handle incoming messages:
//!
//! ```rust,no_run
//! use nexo_retailer_protocol::server::RequestHandler;
//! use nexo_retailer_protocol::{Casp001Document, Casp002Document, Casp003Document, Casp004Document, NexoError};
//!
//! struct MyHandler;
//!
//! #[async_trait::async_trait]
//! impl RequestHandler for MyHandler {
//!     async fn handle_payment_request(&self, req: Casp001Document) -> Result<Casp002Document, NexoError> {
//!         // Process payment request and return response
//!         Ok(Casp002Document::default())
//!     }
//!
//!     async fn handle_admin_request(&self, req: Casp003Document) -> Result<Casp004Document, NexoError> {
//!         // Process admin request and return response
//!         Ok(Casp004Document::default())
//!     }
//! }
//! ```

use crate::NexoError;

// Import Casp document types from generated protobuf code
use crate::{
    Casp001Document, Casp002Document, Casp003Document, Casp004Document,
    Casp005Document, Casp006Document, Casp007Document, Casp008Document,
};

/// Trait for handling incoming Nexo messages
///
/// Applications implement this trait to receive and process incoming Nexo messages.
/// Each method corresponds to a specific message type (Casp001, Casp003, etc.) and
/// returns the appropriate response type.
///
/// # Thread Safety
///
/// The trait requires `Send + Sync` bounds, allowing handlers to be shared across
/// concurrent connections using `Arc<dyn RequestHandler>`.
///
/// # Default Implementations
///
/// All methods have default implementations that return "unsupported message" errors.
/// Applications only need to implement the message types they support.
///
/// # Examples
///
/// ```rust,no_run
/// use nexo_retailer_protocol::server::RequestHandler;
/// use nexo_retailer_protocol::{Casp001Document, Casp002Document, NexoError};
/// use std::sync::Arc;
///
/// struct PaymentHandler;
///
/// #[async_trait::async_trait]
/// impl RequestHandler for PaymentHandler {
///     async fn handle_payment_request(&self, req: Casp001Document) -> Result<Casp002Document, NexoError> {
///         // Handle payment request
///         Ok(Casp002Document::default())
///     }
/// }
///
/// // Wrap in Arc for sharing across connections
/// let handler = Arc::new(PaymentHandler);
/// ```
#[async_trait::async_trait]
pub trait RequestHandler: Send + Sync {
    /// Handle a payment service request (Casp001Document)
    ///
    /// This method is called when the server receives a `SaleToPOIServiceRequestV06` message.
    /// The handler should process the request and return a `SaleToPOIServiceResponseV06`.
    ///
    /// # Arguments
    ///
    /// * `req` - The payment request message
    ///
    /// # Returns
    ///
    /// A `Casp002Document` response message
    ///
    /// # Errors
    ///
    /// Returns `NexoError` if the request cannot be processed
    async fn handle_payment_request(
        &self,
        req: Casp001Document,
    ) -> Result<Casp002Document, NexoError> {
        let _ = req;
        Err(NexoError::Validation {
            field: "message_type",
            reason: "payment requests not supported by this handler",
        })
    }

    /// Handle an admin request (Casp003Document)
    ///
    /// This method is called when the server receives a `SaleToPOIAdminRequestV06` message.
    /// The handler should process the request and return a `SaleToPOIAdminResponseV06`.
    ///
    /// # Arguments
    ///
    /// * `req` - The admin request message
    ///
    /// # Returns
    ///
    /// A `Casp004Document` response message
    ///
    /// # Errors
    ///
    /// Returns `NexoError` if the request cannot be processed
    async fn handle_admin_request(
        &self,
        req: Casp003Document,
    ) -> Result<Casp004Document, NexoError> {
        let _ = req;
        Err(NexoError::Validation {
            field: "message_type",
            reason: "admin requests not supported by this handler",
        })
    }

    /// Handle a payment response (Casp002Document)
    ///
    /// This method is called when the server receives a `SaleToPOIServiceResponseV06` message.
    /// This is typically used in bidirectional communication scenarios.
    ///
    /// # Arguments
    ///
    /// * `req` - The payment response message
    ///
    /// # Returns
    ///
    /// An optional acknowledgment or response
    ///
    /// # Errors
    ///
    /// Returns `NexoError` if the response cannot be processed
    async fn handle_payment_response(
        &self,
        req: Casp002Document,
    ) -> Result<Option<Casp001Document>, NexoError> {
        let _ = req;
        Err(NexoError::Validation {
            field: "message_type",
            reason: "payment responses not supported by this handler",
        })
    }

    /// Handle an admin response (Casp004Document)
    ///
    /// This method is called when the server receives a `SaleToPOIAdminResponseV06` message.
    ///
    /// # Arguments
    ///
    /// * `req` - The admin response message
    ///
    /// # Returns
    ///
    /// An optional acknowledgment or response
    ///
    /// # Errors
    ///
    /// Returns `NexoError` if the response cannot be processed
    async fn handle_admin_response(
        &self,
        req: Casp004Document,
    ) -> Result<Option<Casp003Document>, NexoError> {
        let _ = req;
        Err(NexoError::Validation {
            field: "message_type",
            reason: "admin responses not supported by this handler",
        })
    }

    /// Handle a login request (Casp005Document)
    ///
    /// This method is called when the server receives a login request message.
    ///
    /// # Arguments
    ///
    /// * `req` - The login request message
    ///
    /// # Returns
    ///
    /// A login response message
    ///
    /// # Errors
    ///
    /// Returns `NexoError` if the request cannot be processed
    async fn handle_login_request(
        &self,
        req: Casp005Document,
    ) -> Result<Casp006Document, NexoError> {
        let _ = req;
        Err(NexoError::Validation {
            field: "message_type",
            reason: "login requests not supported by this handler",
        })
    }

    /// Handle a login response (Casp006Document)
    ///
    /// This method is called when the server receives a login response message.
    ///
    /// # Arguments
    ///
    /// * `req` - The login response message
    ///
    /// # Returns
    ///
    /// An optional acknowledgment or response
    ///
    /// # Errors
    ///
    /// Returns `NexoError` if the response cannot be processed
    async fn handle_login_response(
        &self,
        req: Casp006Document,
    ) -> Result<Option<Casp005Document>, NexoError> {
        let _ = req;
        Err(NexoError::Validation {
            field: "message_type",
            reason: "login responses not supported by this handler",
        })
    }

    /// Handle a diagnosis request (Casp007Document)
    ///
    /// This method is called when the server receives a diagnosis request message.
    ///
    /// # Arguments
    ///
    /// * `req` - The diagnosis request message
    ///
    /// # Returns
    ///
    /// A diagnosis response message
    ///
    /// # Errors
    ///
    /// Returns `NexoError` if the request cannot be processed
    async fn handle_diagnosis_request(
        &self,
        req: Casp007Document,
    ) -> Result<Casp008Document, NexoError> {
        let _ = req;
        Err(NexoError::Validation {
            field: "message_type",
            reason: "diagnosis requests not supported by this handler",
        })
    }

    /// Handle a diagnosis response (Casp008Document)
    ///
    /// This method is called when the server receives a diagnosis response message.
    ///
    /// # Arguments
    ///
    /// * `req` - The diagnosis response message
    ///
    /// # Returns
    ///
    /// An optional acknowledgment or response
    ///
    /// # Errors
    ///
    /// Returns `NexoError` if the response cannot be processed
    async fn handle_diagnosis_response(
        &self,
        req: Casp008Document,
    ) -> Result<Option<Casp007Document>, NexoError> {
        let _ = req;
        Err(NexoError::Validation {
            field: "message_type",
            reason: "diagnosis responses not supported by this handler",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock request handler for testing
    ///
    /// This mock implementation tracks method calls and can be used to verify
    /// that the dispatcher routes messages correctly.
    pub struct MockRequestHandler {
        /// Track whether handle_payment_request was called
        pub payment_request_called: std::sync::atomic::AtomicBool,
        /// Track whether handle_admin_request was called
        pub admin_request_called: std::sync::atomic::AtomicBool,
        /// Track whether handle_payment_response was called
        pub payment_response_called: std::sync::atomic::AtomicBool,
        /// Track whether handle_admin_response was called
        pub admin_response_called: std::sync::atomic::AtomicBool,
    }

    impl MockRequestHandler {
        /// Create a new mock handler
        pub fn new() -> Self {
            Self {
                payment_request_called: std::sync::atomic::AtomicBool::new(false),
                admin_request_called: std::sync::atomic::AtomicBool::new(false),
                payment_response_called: std::sync::atomic::AtomicBool::new(false),
                admin_response_called: std::sync::atomic::AtomicBool::new(false),
            }
        }

        /// Reset all tracking flags
        pub fn reset(&self) {
            self.payment_request_called.store(false, std::sync::atomic::Ordering::SeqCst);
            self.admin_request_called.store(false, std::sync::atomic::Ordering::SeqCst);
            self.payment_response_called.store(false, std::sync::atomic::Ordering::SeqCst);
            self.admin_response_called.store(false, std::sync::atomic::Ordering::SeqCst);
        }
    }

    impl Default for MockRequestHandler {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait::async_trait]
    impl RequestHandler for MockRequestHandler {
        async fn handle_payment_request(
            &self,
            _req: Casp001Document,
        ) -> Result<Casp002Document, NexoError> {
            self.payment_request_called.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(Casp002Document::default())
        }

        async fn handle_admin_request(
            &self,
            _req: Casp003Document,
        ) -> Result<Casp004Document, NexoError> {
            self.admin_request_called.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(Casp004Document::default())
        }

        async fn handle_payment_response(
            &self,
            _req: Casp002Document,
        ) -> Result<Option<Casp001Document>, NexoError> {
            self.payment_response_called.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(None)
        }

        async fn handle_admin_response(
            &self,
            _req: Casp004Document,
        ) -> Result<Option<Casp003Document>, NexoError> {
            self.admin_response_called.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(None)
        }
    }

    #[tokio::test]
    async fn test_mock_handler_payment_request() {
        let handler = MockRequestHandler::new();
        let request = Casp001Document::default();

        let result = handler.handle_payment_request(request).await;
        assert!(result.is_ok());
        assert!(handler.payment_request_called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_mock_handler_admin_request() {
        let handler = MockRequestHandler::new();
        let request = Casp003Document::default();

        let result = handler.handle_admin_request(request).await;
        assert!(result.is_ok());
        assert!(handler.admin_request_called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_default_handler_returns_unsupported_error() {
        struct EmptyHandler;
        #[async_trait::async_trait]
        impl RequestHandler for EmptyHandler {}

        let handler = EmptyHandler;
        let request = Casp001Document::default();

        let result = handler.handle_payment_request(request).await;
        assert!(result.is_err());
        match result {
            Err(NexoError::Validation { field, reason }) => {
                assert_eq!(field, "message_type");
                assert!(reason.contains("not supported"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[tokio::test]
    async fn test_handler_is_send_sync() {
        // Verify that RequestHandler trait object is Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Box<dyn RequestHandler>>();
    }
}
