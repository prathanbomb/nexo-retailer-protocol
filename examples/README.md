# Nexo Retailer Protocol Examples

This directory contains example programs demonstrating how to use the Nexo Retailer Protocol library in both standard (std) and embedded (no_std) environments.

## Overview

The examples demonstrate:

- **Tokio Transport** (std environment): Client and server examples using Tokio async runtime
- **Embassy Transport** (no_std environment): Embedded client example for bare-metal devices
- **Message Framing**: Length-prefixed TCP framing for protocol message transmission
- **Error Handling**: Comprehensive error handling for connection, timeout, and encoding/decoding errors

## Examples

### 1. Tokio Client Example (`tokio_client.rs`)

Demonstrates how to create a client connection in a standard environment using Tokio.

**Features:**
- Command-line argument parsing for host and port
- `TokioTransport::connect()` with timeout configuration
- `FramedTransport` wrapping for length-prefixed messaging
- Sending and receiving CASP messages
- Error handling for connection, timeout, and encoding/decoding

**Running the example:**

```bash
# Default address (127.0.0.1:8080)
cargo run --example tokio_client --features std

# Custom address
cargo run --example tokio_client --features std -- 192.168.1.100:8080
```

**Expected output:**

```
Nexo Retailer Protocol - Tokio Client Example
Connecting to: 127.0.0.1:8080

[1] Connecting to server...
✓ Connected successfully

[2] Setting up framed transport...
✓ Framed transport ready

[3] Creating CASP message...
Message created:
  - Message Function: Some("DREQ")
  - Protocol Version: Some("6.0")
  - Transaction ID: Some("EXAMPLE-TX-001")

[4] Sending message to server...
✓ Message sent successfully

[5] Waiting for response...
✓ Response received:
  - Message Function: Some("DREQ")
  - Protocol Version: Some("6.0")
  - Transaction ID: Some("EXAMPLE-TX-001")

[6] Shutting down...
✓ Client exiting normally
```

### 2. Embassy Client Example (`embassy_client.rs`)

Demonstrates how to create a client connection in an embedded environment using Embassy.

**Features:**
- Embassy executor setup with entry macro
- Static buffer allocation for Embassy TCP socket
- `EmbassyTransport` creation with buffer lifetime management
- Timeout configuration with `embassy_time::Duration`
- Embedded-specific error handling
- Buffer size considerations for resource-constrained devices

**Building the example:**

```bash
# Install the target (if not already installed)
rustup target add thumbv7em-none-eabihf

# Build for bare-metal target
cargo build --example embassy_client --features embassy --target thumbv7em-none-eabihf
```

**Hardware Requirements:**

This example requires:
- Embassy-compatible hardware (STM32, ESP32, nRF, etc.)
- Network hardware (Ethernet/WiFi) supported by `embassy-net`
- Hardware-specific network stack initialization

**Note:** The Embassy example demonstrates usage patterns but cannot run without actual hardware or QEMU emulation due to `embassy-net` dependencies. See the Embassy repository for platform-specific examples.

### 3. Echo Server Example (`echo_server.rs`)

Demonstrates how to create a TCP echo server using Tokio for testing client implementations.

**Features:**
- `TcpListener` binding to specified address
- Accept loop for handling incoming connections
- Creating `TokioTransport` from accepted connections
- Concurrent connection handling with `tokio::spawn`
- Echoing received messages back to clients
- Connection counting and detailed logging

**Running the server:**

```bash
# Default address (127.0.0.1:8080)
cargo run --example echo_server --features std

# Custom bind address
cargo run --example echo_server --features std -- 0.0.0.0:9000
```

**Testing the server:**

1. Start the echo server:
   ```bash
   cargo run --example echo_server --features std
   ```

2. In another terminal, run the Tokio client:
   ```bash
   cargo run --example tokio_client --features std -- 127.0.0.1:8080
   ```

**Expected server output:**

```
Nexo Retailer Protocol - Echo Server Example
Binding to: 127.0.0.1:8080

[1] Creating TCP listener...
✓ Server listening on 127.0.0.1:8080
✓ Actual bound address: 127.0.0.1:8080

[2] Server ready. Press Ctrl+C to stop.

Waiting for connections...

--- Connection #1 ---
Accepted connection from 127.0.0.1:54321
[127.0.0.1:54321] Client connected
[127.0.0.1:54321] Received message: msg_fctn=Some("DREQ"), tx_id=Some("EXAMPLE-TX-001")
[127.0.0.1:54321] Echoed message back to client
[127.0.0.1:54321] Client disconnected
[127.0.0.1:54321] Client handler finished
[127.0.0.1:54321] Client handler task completed
```

## Usage Patterns

### Creating a Client Connection (Tokio)

```rust
use nexo_retailer_protocol::{TokioTransport, FramedTransport, Header4};
use core::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to server
    let transport = TokioTransport::connect("127.0.0.1:8080", Duration::from_secs(10)).await?;

    // Wrap in framed transport
    let mut framed = FramedTransport::new(transport);

    // Create and send message
    let message = Header4 {
        msg_fctn: Some("DREQ".to_string()),
        ..Default::default()
    };
    framed.send_message(&message).await?;

    // Receive response
    let response: Header4 = framed.recv_message().await?;

    Ok(())
}
```

### Creating a Server (Tokio)

```rust
use nexo_retailer_protocol::{TokioTransport, FramedTransport};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (stream, addr) = listener.accept().await?;
        tokio::spawn(async move {
            let transport = TokioTransport::new(stream);
            let mut framed = FramedTransport::new(transport);

            // Handle client messages...
        });
    }
}
```

### Creating a Client (Embassy)

```rust
use nexo_retailer_protocol::{EmbassyTransport, FramedTransport};
use embassy_net::TcpSocket;
use embassy_time::Duration;

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    // Allocate static buffers
    static mut RX_BUFFER: [u8; 4096] = [0u8; 4096];
    static mut TX_BUFFER: [u8; 4096] = [0u8; 4096];

    // Create socket (requires network stack initialization)
    let rx_buffer = unsafe { &mut RX_BUFFER };
    let tx_buffer = unsafe { &mut TX_BUFFER };
    let socket = TcpSocket::new(stack, rx_buffer, tx_buffer);

    // Create transport
    let transport = EmbassyTransport::new(socket, rx_buffer, tx_buffer)
        .with_timeouts(Duration::from_secs(30), Duration::from_secs(10));

    // Connect and use...
    transport.connect("192.168.1.100:8080").await?;
}
```

## Key Concepts

### Length-Prefixed Framing

All examples use `FramedTransport`, which adds length-prefixed framing to messages:

- **4-byte big-endian length prefix** followed by message body
- Automatic handling of partial reads/writes
- 4MB maximum frame size enforcement
- Compatible with standard protobuf-over-TCP conventions

### Timeout Configuration

Both Tokio and Embassy transports support configurable timeouts:

- **Connect timeout**: Maximum time to establish connection
- **Read timeout**: Maximum time to wait for incoming data
- **Write timeout**: Maximum time to wait for write completion

Default timeouts:
- Connect: 10 seconds
- Read: 30 seconds
- Write: 10 seconds

### Error Handling

The library provides comprehensive error types via `NexoError`:

- `NexoError::Connection`: Connection failures and errors
- `NexoError::Timeout`: Operation timeout
- `NexoError::Encoding`: Message encoding failures
- `NexoError::Decoding`: Message decoding failures

## Troubleshooting

### Port Already in Use

If you get an "address already in use" error:

```bash
# On Linux/macOS
lsof -i :8080
kill -9 <PID>

# Or use a different port
cargo run --example echo_server --features std -- 127.0.0.1:9000
```

### Connection Refused

Ensure the server is running before starting the client:

```bash
# Terminal 1: Start server
cargo run --example echo_server --features std

# Terminal 2: Start client
cargo run --example tokio_client --features std -- 127.0.0.1:8080
```

### Embassy Build Errors

Embassy examples require specific targets and may have dependency issues:

```bash
# Install the target
rustup target add thumbv7em-none-eabihf

# Build (may fail due to embassy-net dependencies)
cargo build --example embassy_client --features embassy --target thumbv7em-none-eabihf
```

Note: Embassy examples are primarily for documentation purposes. Actual embedded usage requires hardware-specific setup.

## Further Reading

- **Tokio Documentation**: https://tokio.rs/
- **Embassy Documentation**: https://embassy.dev/
- **Nexo Retailer Protocol Specification**: See project documentation
- **Prost Documentation**: https://docs.rs/prost/

## Contributing

When adding new examples:

1. Add the example to `Cargo.toml` with appropriate `required-features`
2. Document the example in this README
3. Include comprehensive comments in the example code
4. Test the example compiles and runs successfully
5. Follow the existing code style and patterns
