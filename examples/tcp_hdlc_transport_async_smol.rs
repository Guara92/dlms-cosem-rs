//! Asynchronous TCP + HDLC transport example with Smol runtime for DLMS/COSEM.
//!
//! This example demonstrates how to layer HDLC framing on top of async TCP transport
//! using Smol runtime for DLMS communication over TCP with HDLC encapsulation.
//!
//! Smol is a lightweight async runtime, perfect for embedded concentrators
//! and resource-constrained systems.
//!
//! Required features: `async-client`, `transport-tcp-async`, `transport-hdlc-async`, `smol`
//!
//! # Usage
//!
//! ```bash
//! cargo run --example tcp_hdlc_transport_async_smol --features async-client,transport-tcp-async,transport-hdlc-async,smol,encode,parse
//! ```
//!
//! # Note
//!
//! This is a demonstrative example showing API usage patterns.
//! For actual meter communication, replace the mock server with a real DLMS device.

#[cfg(all(
    feature = "async-client",
    feature = "transport-tcp-async",
    feature = "transport-hdlc-async",
    feature = "smol"
))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use dlms_cosem::async_client::AsyncClientBuilder;
    use dlms_cosem::client::ClientSettings;
    use dlms_cosem::transport::hdlc::AsyncHdlcTransport;
    use dlms_cosem::transport::tcp::SmolTcpTransport;
    use std::time::Duration;

    println!("=== DLMS Async TCP + HDLC Transport Example (Smol) ===\n");

    // Connection parameters
    let server_address = "192.168.1.100:4059";
    let hdlc_client_address = 0x01; // HDLC client address
    let hdlc_server_address = 0x10; // HDLC server physical address
    let client_sap = 16; // DLMS client SAP
    let server_sap = 1; // DLMS server SAP

    println!("1. Creating layered async transport (TCP + HDLC) with Smol...");
    println!("   TCP Server: {}", server_address);
    println!("   HDLC Client Address: 0x{:02X}", hdlc_client_address);
    println!("   HDLC Server Address: 0x{:02X}", hdlc_server_address);
    println!("   Runtime: Smol (lightweight)");

    // Run async code with Smol runtime
    smol::block_on(async {
        // Create base async TCP transport with Smol
        match SmolTcpTransport::connect(server_address).await {
            Ok(mut tcp_transport) => {
                println!("   ✓ Async TCP connection established");

                // Configure TCP timeouts
                tcp_transport.set_read_timeout(Some(Duration::from_secs(30)));
                tcp_transport.set_write_timeout(Some(Duration::from_secs(30)));

                println!("   ✓ TCP timeouts configured");

                // Wrap TCP transport with async HDLC framing
                let hdlc_transport = AsyncHdlcTransport::new(
                    tcp_transport,
                    hdlc_client_address,
                    hdlc_server_address,
                );

                println!("   ✓ Async HDLC wrapper added");
                println!("   Transport stack: DLMS -> HDLC -> TCP -> Network (Smol)");

                println!("\n2. Creating async DLMS client...");

                // Configure client settings
                let settings = ClientSettings {
                    client_address: client_sap,
                    server_address: server_sap,
                    ..Default::default()
                };

                // Create async DLMS client with layered transport using builder
                // Option 1: Heap-allocated buffer (runtime size)
                let mut client =
                    AsyncClientBuilder::new(hdlc_transport, settings).build_with_heap(2048);

                // Option 2: Stack-allocated buffer (compile-time size, for embedded/no_std)
                // let mut client = AsyncClientBuilder::new(hdlc_transport, settings)
                //     .build_with_heapless::<2048>();

                println!("   ✓ Async client created (Smol runtime)");
                println!("   DLMS Client SAP: {}", client_sap);
                println!("   DLMS Server SAP: {}", server_sap);

                println!("\n3. Establishing association...");
                // Note: This will fail without a real server
                match client.connect().await {
                    Ok(_) => {
                        println!("   ✓ Association established");

                        println!("\n4. Example async operations...");
                        println!("   (Operations would happen here with a real server)");
                        // Example async operations:
                        // - Read clock: client.read_clock().await
                        // - Read register: client.read(class_id, obis_code, attribute_id, None).await
                        // - Write data: client.write(class_id, obis_code, attribute_id, value).await

                        println!("\n5. Disconnecting...");
                        match client.disconnect().await {
                            Ok(_) => println!("   ✓ Disconnected successfully"),
                            Err(e) => println!("   ✗ Disconnect failed: {}", e),
                        }
                    }
                    Err(e) => {
                        println!("   ✗ Association failed: {}", e);
                        println!("   (This is expected without a real DLMS server)");
                    }
                }
            }
            Err(e) => {
                println!("   ✗ TCP connection failed: {}", e);
                println!("   (This is expected without a real DLMS server)");
                println!("\n   To use this example:");
                println!("   1. Configure a DLMS server with HDLC over TCP");
                println!("   2. Update the server_address and HDLC addresses");
                println!("   3. Run the example again");
            }
        }

        Ok::<(), Box<dyn std::error::Error>>(())
    })?;

    println!("\n=== Example complete ===\n");

    // Example usage patterns
    println!("API Usage Patterns (Smol + HDLC):");
    println!("----------------------------------");
    println!("use smol::block_on;");
    println!();
    println!("fn main() -> Result<(), Box<dyn std::error::Error>> {{");
    println!("    smol::block_on(async {{");
    println!("        // Create base async TCP transport with Smol");
    println!("        let mut tcp = SmolTcpTransport::connect(\"192.168.1.100:4059\").await?;");
    println!("        tcp.set_read_timeout(Some(Duration::from_secs(30)));");
    println!();
    println!("        // Wrap with async HDLC framing");
    println!("        let hdlc = AsyncHdlcTransport::new(");
    println!("            tcp,");
    println!("            0x01,  // HDLC client address");
    println!("            0x10,  // HDLC server address");
    println!("        );");
    println!();
    println!("        // Create async DLMS client with layered transport");
    println!("        let settings = ClientSettings {{");
    println!("            client_address: 16,  // DLMS client SAP");
    println!("            server_address: 1,   // DLMS server SAP");
    println!("            ..Default::default()");
    println!("        }};");
    println!("        // Build client with heap buffer (2048 bytes)");
    println!("        let mut client = AsyncClientBuilder::new(hdlc, settings)");
    println!("            .build_with_heap(2048);");
    println!();
    println!("        // Or with heapless buffer for embedded systems:");
    println!("        // let mut client = AsyncClientBuilder::new(hdlc, settings)");
    println!("        //     .build_with_heapless::<2048>();");
    println!();
    println!("        // Use client normally - HDLC framing is automatic and async");
    println!("        client.connect().await?;");
    println!("        let data = client.read(class_id, obis_code, attribute_id, None).await?;");
    println!("        client.disconnect().await?;");
    println!("        Ok(())");
    println!("    }})");
    println!("}}");
    println!();
    println!("HDLC Frame Structure (Async with Smol):");
    println!("----------------------------------------");
    println!("Flag | Format | Length | Dest | Src | Ctrl | HCS | LLC | APDU | FCS | Flag");
    println!("0x7E |   1B   |  1-2B  | 1-4B |1-4B | 1B   | 2B  | 3B  |  nB  | 2B  | 0x7E");
    println!();
    println!("Where:");
    println!("  - All framing/deframing happens automatically in async tasks");
    println!("  - Dest/Src: HDLC addresses (0x01, 0x10 in this example)");
    println!("  - LLC: Logical Link Control header (0xE6 0xE6 0x00)");
    println!("  - APDU: DLMS application protocol data");
    println!("  - HCS/FCS: Frame check sequences (CRC-16)");
    println!();
    println!("Benefits of Smol with HDLC:");
    println!("----------------------------");
    println!("✓ Lightweight runtime (minimal binary size)");
    println!("✓ Efficient HDLC frame handling in async tasks");
    println!("✓ Perfect for embedded concentrators (100-1,000 meters)");
    println!("✓ Low memory footprint for frame buffers");
    println!("✓ Automatic frame validation (FCS checks)");
    println!("✓ Simple API with block_on for embedded use cases");
    println!();
    println!("Use Cases:");
    println!("----------");
    println!("• Embedded Linux concentrators (ARM, x86)");
    println!("• Serial-over-IP gateways with moderate load");
    println!("• Edge devices managing multiple meters");
    println!("• Resource-constrained systems (< 100MB RAM)");

    Ok(())
}

#[cfg(not(all(
    feature = "async-client",
    feature = "transport-tcp-async",
    feature = "transport-hdlc-async",
    feature = "smol"
)))]
fn main() {
    eprintln!(
        "This example requires the 'async-client', 'transport-tcp-async', 'transport-hdlc-async', and 'smol' features."
    );
    eprintln!(
        "Run with: cargo run --example tcp_hdlc_transport_async_smol --features async-client,transport-tcp-async,transport-hdlc-async,smol,encode,parse"
    );
    std::process::exit(1);
}
