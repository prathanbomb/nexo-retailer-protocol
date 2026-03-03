//! Mock Nexo Server for Integration Testing
//!
//! This module provides a mock server implementation that follows the Nexo TCP
//! framing protocol. It supports echo responses, connection close simulation,
//! and delayed response simulation for testing client behavior.
//!
//! The server can handle all 17 CASP message types by receiving raw bytes and
//! sending back appropriate response documents.

use std::sync::Arc;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use nexo_retailer_protocol::transport::FramedTransport;
use nexo_retailer_protocol::transport::TokioTransport;
use prost::Message;

/// Mock Nexo Server that implements the Nexo TCP framing protocol
///
/// The server supports:
/// - Echo responses: Returns the received message
/// - Connection close simulation: For testing reconnection logic
/// - Delayed response simulation: For testing timeout handling
/// - Failure simulation: For testing exponential backoff
#[derive(Clone)]
pub struct MockNexoServer {
    listener: Arc<Mutex<TcpListener>>,
    addr: SocketAddr,
    shutdown_signal: Arc<Mutex<bool>>,
    close_on_connect: Arc<Mutex<bool>>,
    delay_response_ms: Arc<Mutex<u64>>,
    reject_attempts: Arc<Mutex<u32>>,
    connection_count: Arc<Mutex<u32>>,
}

impl MockNexoServer {
    /// Start a new mock server on a random available port
    ///
    /// Returns the server instance bound to the local address
    pub async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        // Bind to port 0 to get a random available port
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;

        Ok(Self {
            listener: Arc::new(Mutex::new(listener)),
            addr,
            shutdown_signal: Arc::new(Mutex::new(false)),
            close_on_connect: Arc::new(Mutex::new(false)),
            delay_response_ms: Arc::new(Mutex::new(0)),
            reject_attempts: Arc::new(Mutex::new(0)),
            connection_count: Arc::new(Mutex::new(0)),
        })
    }

    /// Get the server address
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Set the flag to close connection immediately after accepting
    ///
    /// Useful for testing reconnection logic
    pub async fn set_close_on_connect(&self, close: bool) {
        let mut flag = self.close_on_connect.lock().await;
        *flag = close;
    }

    /// Set the delay before sending response (in milliseconds)
    ///
    /// Useful for testing timeout handling
    pub async fn set_delay_response(&self, delay_ms: u64) {
        let mut delay = self.delay_response_ms.lock().await;
        *delay = delay_ms;
    }

    /// Set the number of connection attempts to reject
    ///
    /// Useful for testing exponential backoff
    pub async fn set_reject_attempts(&self, count: u32) {
        let mut reject = self.reject_attempts.lock().await;
        *reject = count;
    }

    /// Get the number of connections accepted
    pub async fn connection_count(&self) -> u32 {
        let count = self.connection_count.lock().await;
        *count
    }

    /// Run the server - accepts connections and handles them
    ///
    /// This should be spawned as a background task
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = self.listener.clone();
        let shutdown = self.shutdown_signal.clone();
        let close_on_connect = self.close_on_connect.clone();
        let delay_response_ms = self.delay_response_ms.clone();
        let reject_attempts = self.reject_attempts.clone();
        let connection_count = self.connection_count.clone();

        loop {
            // Check shutdown signal
            {
                let shutdown_flag = shutdown.lock().await;
                if *shutdown_flag {
                    break;
                }
            }

            // Accept connection with timeout
            let accept_result = tokio::time::timeout(
                tokio::time::Duration::from_secs(5),
                listener.lock().await.accept()
            ).await;

            let (socket, _peer_addr) = match accept_result {
                Ok(Ok(conn)) => conn,
                Ok(Err(_)) => continue, // Accept failed, try again
                Err(_) => {
                    // Timeout - check shutdown and continue
                    continue;
                }
            };

            // Increment connection count
            {
                let mut count = connection_count.lock().await;
                *count += 1;
            }

            // Check if we should reject this connection
            let should_reject = {
                let mut reject = reject_attempts.lock().await;
                if *reject > 0 {
                    *reject -= 1;
                    true
                } else {
                    false
                }
            };

            if should_reject {
                drop(socket);
                continue;
            }

            // Check if we should close immediately
            let should_close = {
                let flag = close_on_connect.lock().await;
                *flag
            };

            if should_close {
                drop(socket);
                continue;
            }

            // Handle the connection
            let delay_ms = {
                let delay = delay_response_ms.lock().await;
                *delay
            };

            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(socket, delay_ms).await {
                    eprintln!("Mock server connection handler error: {:?}", e);
                }
            });
        }

        Ok(())
    }

    /// Handle a single client connection
    ///
    /// This handler receives raw bytes (as Vec<u8>) and echoes back a Casp002Document
    /// response. The raw bytes approach allows handling any CASP message type without
    /// needing to decode the specific request type.
    async fn handle_connection(
        socket: TcpStream,
        delay_ms: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let transport = TokioTransport::new(socket);
        let mut framed = FramedTransport::new(transport);

        // Keep handling messages until the connection is closed
        loop {
            // Receive request as raw bytes to handle any CASP message type
            let receive_result = framed.recv_raw().await;

            match receive_result {
                Ok(_request_bytes) => {
                    // Add delay if configured
                    if delay_ms > 0 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    }

                    // Send back a Casp002Document response for any request
                    // This allows testing all CASP types without complex routing logic
                    let response = nexo_retailer_protocol::Casp002Document {
                        document: Some(nexo_retailer_protocol::Casp002DocumentDocument {
                            sale_to_poi_svc_rsp: Some(nexo_retailer_protocol::SaleToPoiServiceResponseV06::default()),
                        }),
                    };

                    // Encode and send the response
                    let response_bytes = response.encode_to_vec();
                    if let Err(e) = framed.send_raw(&response_bytes).await {
                        eprintln!("Mock server send error: {:?}", e);
                        break;
                    }
                }
                Err(e) => {
                    // Connection closed or error - this is expected when client disconnects
                    eprintln!("Mock server connection ended: {:?}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Stop the server gracefully
    pub async fn stop(self) {
        let mut shutdown = self.shutdown_signal.lock().await;
        *shutdown = true;

        // Give time for the run loop to exit
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
