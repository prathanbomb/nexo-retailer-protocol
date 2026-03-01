//! Tokio Server Example with Tracing for Nexo Retailer Protocol
//!
//! This example demonstrates how to use the NexoServer with structured logging
//! using the tracing crate. It shows proper initialization of tracing-subscriber
//! with environment variable support for log level configuration.
//!
//! # Usage
//!
//! ```bash
//! # Run with default INFO log level
//! cargo run --example tokio_server_with_logging --features std
//!
//! # Run with DEBUG log level
//! RUST_LOG=debug cargo run --example tokio_server_with_logging --features std
//!
//! # Run with TRACE log level (very verbose)
//! RUST_LOG=trace cargo run --example tokio_server_with_logging --features std
//!
//! # Run only nexo_retailer_protocol logs at DEBUG level
//! RUST_LOG=nexo_retailer_protocol=debug cargo run --example tokio_server_with_logging --features std
//! ```
//!
//! # Features Demonstrated
//!
//! - tracing-subscriber initialization with env-filter
//! - Environment variable based log level configuration (RUST_LOG)
//! - Structured logging with fields (addr, message_count, error, etc.)
//! - Connection lifecycle logging (accept, messages, close)
//! - Handler error logging with structured error field
//! - Heartbeat event logging (send, timeout)
//!
//! # Running the Example
//!
//! 1. Start this server:
//!    ```bash
//!    cargo run --example tokio_server_with_logging --features std
//!    ```
//!
//! 2. In another terminal, run a client:
//!    ```bash
//!    cargo run --example tokio_client --features std -- 127.0.0.1:8080
//!    ```
//!
//! # Expected Output
//!
//! With INFO level (default):
//! ```text
//! 2025-03-01T18:00:00.000Z INFO nexo_retailer_protocol::server::std_impl: Connection accepted addr=127.0.0.1:52341
//! 2025-03-01T18:00:00.100Z INFO nexo_retailer_protocol::server::std_impl: Connection closed addr=127.0.0.1:52341 message_count=1
//! ```
//!
//! With DEBUG level (RUST_LOG=debug):
//! ```text
//! 2025-03-01T18:00:00.000Z INFO nexo_retailer_protocol::server::std_impl: Connection accepted addr=127.0.0.1:52341
//! 2025-03-01T18:00:00.050Z DEBUG nexo_retailer_protocol::server::std_impl: Message received byte_count=128
//! 2025-03-01T18:00:00.055Z DEBUG nexo_retailer_protocol::server::std_impl: Dispatching to handler
//! 2025-03-01T18:00:00.100Z INFO nexo_retailer_protocol::server::std_impl: Connection closed addr=127.0.0.1:52341 message_count=1
//! ```

#[cfg(feature = "std")]
use nexo_retailer_protocol::NexoServer;

#[cfg(feature = "std")]
use tokio::main;

#[cfg(feature = "std")]
#[main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing-subscriber with environment variable support
    //
    // The env-filter reads the RUST_LOG environment variable to determine
    // the log level. If RUST_LOG is not set, it defaults to "info".
    //
    // Valid RUST_LOG values:
    // - "error", "warn", "info", "debug", "trace"
    // - "nexo_retailer_protocol=debug" (module-specific)
    // - "nexo_retailer_protocol=debug,tokio=info" (multiple modules)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    println!("Nexo Retailer Protocol - Tokio Server with Tracing Example");
    println!("Bind address: 127.0.0.1:8080");
    println!("\nLog level controlled by RUST_LOG environment variable:");
    println!("  RUST_LOG=info  (default - INFO and above)");
    println!("  RUST_LOG=debug (DEBUG and above - includes message details)");
    println!("  RUST_LOG=trace (TRACE and above - very verbose)");
    println!("\nPress Ctrl+C to stop the server");

    // Bind to address and create server
    let server = NexoServer::bind("127.0.0.1:8080").await?;

    println!("\nServer listening on 127.0.0.1:8080\n");

    // Run the server (accepts connections indefinitely)
    server.run().await?;

    Ok(())
}
