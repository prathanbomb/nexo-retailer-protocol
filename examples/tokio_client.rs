//! Tokio Client Example for Nexo Retailer Protocol
//!
//! This example demonstrates how to use the Tokio transport implementation
//! to connect to a Nexo server and exchange CASP messages.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example tokio_client --features std -- 127.0.0.1:8080
//! ```
//!
//! # Features Demonstrated
//!
//! - Command-line argument parsing for host and port
//! - TokioTransport::connect() with timeout configuration
//! - FramedTransport wrapping for length-prefixed messaging
//! - Sending/receiving CASP messages
//! - Error handling for connection, timeout, encoding/decoding
//! - Proper resource cleanup
//!
//! # Running the Example
//!
//! 1. Start an echo server (see echo_server.rs example):
//!    ```bash
//!    cargo run --example echo_server --features std
//!    ```
//!
//! 2. In another terminal, run this client:
//!    ```bash
//!    cargo run --example tokio_client --features std -- 127.0.0.1:8080
//!    ```

#[cfg(feature = "std")]
use core::time::Duration;

#[cfg(feature = "std")]
use nexo_retailer_protocol::{
    FramedTransport, Header4,
};

#[cfg(feature = "std")]
use tokio::main;

#[cfg(feature = "std")]
#[main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();

    let addr = if args.len() > 1 {
        // User provided address
        args[1].clone()
    } else {
        // Default address for local testing
        println!("Usage: {} [host:port]", args[0]);
        println!("Using default address: 127.0.0.1:8080");
        "127.0.0.1:8080".to_string()
    };

    println!("Nexo Retailer Protocol - Tokio Client Example");
    println!("Connecting to: {}", addr);

    // Step 1: Connect to server using TokioTransport
    // The connect() method takes an address string and timeout duration
    println!("\n[1] Connecting to server...");
    let transport = match TokioTransport::connect(&addr, Duration::from_secs(10)).await {
        Ok(t) => {
            println!("✓ Connected successfully");
            t
        }
        Err(e) => {
            eprintln!("✗ Connection failed: {}", e);
            eprintln!("  Make sure the server is running at {}", addr);
            return Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>);
        }
    };

    // Step 2: Wrap the transport in FramedTransport
    // This adds length-prefixed framing for message boundaries
    println!("\n[2] Setting up framed transport...");
    let mut framed = FramedTransport::new(transport);
    println!("✓ Framed transport ready");

    // Step 3: Create a CASP message to send
    // We'll create a simple Header4 message for demonstration
    println!("\n[3] Creating CASP message...");
    let header = Header4 {
        msg_fctn: Some("DREQ".to_string()), // Diagnostic Request
        proto_vrsn: Some("6.0".to_string()),
        tx_id: Some("EXAMPLE-TX-001".to_string()),
        ..Default::default()
    };

    println!("Message created:");
    println!("  - Message Function: {:?}", header.msg_fctn);
    println!("  - Protocol Version: {:?}", header.proto_vrsn);
    println!("  - Transaction ID: {:?}", header.tx_id);

    // Step 4: Send the message through the framed transport
    // send_message() handles:
    //   - Encoding the message to protobuf format
    //   - Adding 4-byte big-endian length prefix
    //   - Writing all bytes to the transport
    println!("\n[4] Sending message to server...");
    if let Err(e) = framed.send_message(&header).await {
        eprintln!("✗ Failed to send message: {}", e);
        return Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>);
    }
    println!("✓ Message sent successfully");

    // Step 5: Receive a response from the server
    // recv_message() handles:
    //   - Reading the 4-byte length prefix
    //   - Reading the exact message body
    //   - Decoding the protobuf message
    println!("\n[5] Waiting for response...");
    match framed.recv_message::<Header4>().await {
        Ok(response) => {
            println!("✓ Response received:");
            println!("  - Message Function: {:?}", response.msg_fctn);
            println!("  - Protocol Version: {:?}", response.proto_vrsn);
            println!("  - Transaction ID: {:?}", response.tx_id);
        }
        Err(e) => {
            eprintln!("✗ Failed to receive response: {}", e);
            eprintln!("  This is expected if the server doesn't echo properly");
            return Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>);
        }
    }

    // Step 6: Clean shutdown
    // The transport will be automatically dropped when it goes out of scope
    println!("\n[6] Shutting down...");
    println!("✓ Client exiting normally");

    Ok(())
}

// Re-import for the example code
#[cfg(feature = "std")]
use nexo_retailer_protocol::TokioTransport;
