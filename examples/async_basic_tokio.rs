//! Async DLMS Client Example with Tokio
//!
//! This example demonstrates how to use the async DLMS client with Tokio runtime.
//! It shows the API patterns and usage, ready to be adapted for real TCP or serial connections.
//!
//! To use with a real device:
//! 1. Replace the commented sections with actual transport (TokioTcpTransport or TokioSerialTransport)
//! 2. Configure the correct IP address or serial port
//! 3. Set appropriate authentication and encryption keys
//!
//! ## What this example demonstrates:
//! - Creating an async DLMS client with Tokio
//! - Connecting to a server (AARQ/AARE)
//! - Reading attributes asynchronously
//! - Writing attributes asynchronously
//! - Invoking methods asynchronously
//! - Proper disconnection
//!
//! ## Features required:
//! - async-client
//! - tokio
//! - encode
//! - parse

use dlms_cosem::ObisCode;
use dlms_cosem::client::ClientSettings;

#[cfg(all(feature = "async-client", feature = "tokio", feature = "encode", feature = "parse"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DLMS Async Client Example (Tokio) ===\n");

    // ============================================================================
    // STEP 1: Configure Client Settings
    // ============================================================================
    let settings = ClientSettings {
        client_address: 16,   // Public client address
        server_address: 1,    // Management logical device
        max_pdu_size: 0xFFFF, // Maximum PDU size
        ..ClientSettings::default()
    };

    println!("Client Configuration:");
    println!("  Client Address: {}", settings.client_address);
    println!("  Server Address: {}", settings.server_address);
    println!("  Max PDU Size: {}", settings.max_pdu_size);
    println!("  Runtime: Tokio (async)\n");

    // ============================================================================
    // STEP 2: Create Transport
    // ============================================================================
    println!("Step 1: Creating async transport...");

    // For real usage with TCP:
    // ```
    // use dlms_cosem::transport::async::tokio::TokioTcpTransport;
    // let transport = TokioTcpTransport::connect("192.168.1.100:4059").await?;
    // ```

    // For real usage with Serial (HDLC):
    // ```
    // use dlms_cosem::transport::async::tokio::TokioSerialTransport;
    // let transport = TokioSerialTransport::open("/dev/ttyUSB0", 9600).await?;
    // ```

    println!("  Transport type: TCP or Serial (see source code for examples)");
    println!("  Note: This example shows API usage patterns\n");

    // Uncomment when using real transport:
    // let mut client = AsyncClientBuilder::new(transport, settings)
    //     .build_with_heap(2048);

    // ============================================================================
    // STEP 3: Connect to Server
    // ============================================================================
    println!("Step 2: Connecting to DLMS server...");
    println!("  Sending AARQ (Association Request)");

    // client.connect().await?;

    println!("  ✓ Connection established (AARE received)");
    println!("  ✓ Association successful\n");

    // ============================================================================
    // STEP 4: Read Attributes
    // ============================================================================
    println!("Step 3: Reading register values asynchronously...\n");

    // Example 1: Read Active Energy Import (1.0.1.8.0.255)
    let _active_energy_obis = ObisCode::new(1, 0, 1, 8, 0, 255);
    println!("  Reading: Active Energy Import (1.0.1.8.0.255)");
    println!("  Usage:");
    println!("    let value = client.read(");
    println!("        3,                    // class_id (Register)");
    println!("        active_energy_obis,   // OBIS code");
    println!("        2,                    // attribute_id (value)");
    println!("        None                  // access_selector");
    println!("    ).await?;");

    // let value = client.read(3, active_energy_obis, 2, None).await?;
    // println!("  Value: {:?}\n", value);

    // Example 2: Read Reactive Energy Import (1.0.2.8.0.255)
    let _reactive_energy_obis = ObisCode::new(1, 0, 2, 8, 0, 255);
    println!("\n  Reading: Reactive Energy Import (1.0.2.8.0.255)");
    // let value = client.read(3, reactive_energy_obis, 2, None).await?;
    // println!("  Value: {:?}\n", value);

    // Example 3: Read Voltage (1.0.32.7.0.255)
    let _voltage_obis = ObisCode::new(1, 0, 32, 7, 0, 255);
    println!("\n  Reading: Instantaneous Voltage (1.0.32.7.0.255)");
    // let value = client.read(3, voltage_obis, 2, None).await?;
    // println!("  Value: {:?}\n", value);

    println!("\n  ✓ All reads completed asynchronously");

    // ============================================================================
    // STEP 5: Write Attributes
    // ============================================================================
    println!("\nStep 4: Writing data object value...");

    let _data_obis = ObisCode::new(0, 0, 96, 1, 0, 255);
    println!("  Target: Data object (0.0.96.1.0.255)");
    println!("  Usage:");
    println!("    use dlms_cosem::Data;");
    println!("    let new_value = Data::OctetString(b\"DLMS-ASYNC\".to_vec());");
    println!("    client.write(");
    println!("        1,            // class_id (Data)");
    println!("        data_obis,    // OBIS code");
    println!("        2,            // attribute_id (value)");
    println!("        new_value,    // new value");
    println!("        None          // access_selector");
    println!("    ).await?;");

    // use dlms_cosem::Data;
    // let new_value = Data::OctetString(b"DLMS-ASYNC".to_vec());
    // client.write(1, data_obis, 2, new_value, None).await?;

    println!("\n  ✓ Write successful");

    // ============================================================================
    // STEP 6: Invoke Methods
    // ============================================================================
    println!("\nStep 5: Invoking Clock.adjust_to_quarter method...");

    let _clock_obis = ObisCode::new(0, 0, 1, 0, 0, 255);
    println!("  Target: Clock (0.0.1.0.0.255)");
    println!("  Method: 1 (adjust_to_quarter)");
    println!("  Usage:");
    println!("    let result = client.method(");
    println!("        8,           // class_id (Clock)");
    println!("        clock_obis,  // OBIS code");
    println!("        1,           // method_id (adjust_to_quarter)");
    println!("        None         // parameters");
    println!("    ).await?;");

    // let result = client.method(8, clock_obis, 1, None).await?;
    // if let Some(data) = result {
    //     println!("  Return data: {:?}", data);
    // } else {
    //     println!("  No return data");
    // }

    println!("\n  ✓ Method invoked successfully");

    // ============================================================================
    // STEP 7: Disconnect
    // ============================================================================
    println!("\nStep 6: Disconnecting from server...");
    println!("  Sending RLRQ (Release Request)");

    // client.disconnect().await?;

    println!("  ✓ Disconnected (RLRE received)");
    println!("  ✓ Association released\n");

    // ============================================================================
    // Summary
    // ============================================================================
    println!("=== Example Complete ===\n");
    println!("Key benefits of async client with Tokio:");
    println!("  ✓ Non-blocking I/O operations");
    println!("  ✓ Efficient handling of multiple concurrent connections");
    println!("  ✓ Better resource utilization for high-concurrency scenarios");
    println!("  ✓ Native async/await syntax");
    println!("  ✓ Rich ecosystem (tokio::time, tokio::select!, etc.)");

    println!("\nNext steps for production use:");
    println!("  1. Add real transport (TCP or Serial)");
    println!("  2. Configure authentication:");
    println!(
        "     settings.authentication = Some(Authentication::LowL
evelSecurity {{ ... }});"
    );
    println!("  3. Enable encryption:");
    println!("     settings.use_ciphering = true;");
    println!("     settings.encryption_key = Some([0u8; 16]);");
    println!("  4. Add error handling and retry logic");
    println!("  5. Use connection pooling for multiple meters");
    println!("  6. Add timeouts with tokio::time::timeout()");

    println!("\nBuffer allocation options:");
    println!("  - Heap allocation (runtime size):");
    println!("      AsyncClientBuilder::new(transport, settings).build_with_heap(2048)");
    println!("  - Stack allocation (compile-time size, no_std):");
    println!("      AsyncClientBuilder::new(transport, settings).build_with_heapless::<2048>()");
    println!();
    println!("Related examples:");
    println!("  - tcp_transport_async_tokio.rs - Real TCP transport");
    println!("  - tcp_transport_async_smol.rs  - Lightweight async with smol runtime");

    Ok(())
}

#[cfg(not(all(
    feature = "async-client",
    feature = "tokio",
    feature = "encode",
    feature = "parse"
)))]
fn main() {
    println!("This example requires 'async-client', 'tokio', 'encode', and 'parse' features.");
    println!();
    println!("Run with:");
    println!("  cargo run --example async_basic_tokio --features async-client,tokio,encode,parse");
    println!();
    println!("Or with all features:");
    println!("  cargo run --example async_basic_tokio --all-features");
}
