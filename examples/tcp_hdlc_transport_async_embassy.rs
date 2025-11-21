//! Asynchronous TCP + HDLC transport example with Embassy runtime for DLMS/COSEM.
//!
//! This example demonstrates how to layer HDLC framing on top of async TCP transport
//! using Embassy runtime (embedded-first, can run on std) for DLMS communication
//! over TCP with HDLC encapsulation.
//!
//! Required features: `async-client`, `transport-tcp-async`, `transport-hdlc-async`, `embassy`
//!
//! # Usage
//!
//! ```bash
//! cargo run --example tcp_hdlc_transport_async_embassy --features async-client,transport-tcp-async,transport-hdlc-async,embassy,encode,parse
//! ```
//!
//! # Note
//!
//! This is a demonstrative example showing API usage patterns.
//! For actual meter communication, replace the mock server with a real DLMS device.
//!
//! Embassy is designed for embedded systems but can run on std for testing/development.

#[cfg(all(
    feature = "async-client",
    feature = "transport-tcp-async",
    feature = "transport-hdlc-async",
    feature = "embassy"
))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use dlms_cosem::async_client::AsyncClientBuilder;
    use dlms_cosem::client::ClientSettings;
    use dlms_cosem::transport::hdlc::AsyncHdlcTransport;
    use dlms_cosem::transport::tcp::EmbassyTcpTransport;
    use std::time::Duration;

    println!("=== DLMS Async TCP + HDLC Transport Example (Embassy) ===\n");

    // Connection parameters
    let server_address = "192.168.1.100:4059";
    let hdlc_client_address = 0x01; // HDLC client address
    let hdlc_server_address = 0x10; // HDLC server physical address
    let client_sap = 16; // DLMS client SAP
    let server_sap = 1; // DLMS server SAP

    println!("1. Creating Embassy executor (embedded-first async runtime)...");
    println!("   Runtime: Embassy (executor-agnostic)");
    println!("   Mode: std (for testing; can run no_std on embedded)");

    // Embassy uses futures-executor for std environments
    futures_executor::block_on(async {
        println!("   ✓ Embassy executor running");

        println!("\n2. Creating layered async transport (TCP + HDLC)...");
        println!("   TCP Server: {}", server_address);
        println!("   HDLC Client Address: 0x{:02X}", hdlc_client_address);
        println!("   HDLC Server Address: 0x{:02X}", hdlc_server_address);

        // Create base async TCP transport with Embassy
        match EmbassyTcpTransport::connect(server_address).await {
            Ok(mut tcp_transport) => {
                println!("   ✓ Async TCP connection established (Embassy)");

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
                println!("   Transport stack: DLMS -> HDLC -> TCP -> Network (Embassy)");

                println!("\n3. Creating async DLMS client...");

                // Configure client settings
                let settings = ClientSettings {
                    client_address: client_sap,
                    server_address: server_sap,
                    ..Default::default()
                };

                // Create async DLMS client with layered transport using builder
                // Option 1: Heap-allocated buffer (runtime size, for std)
                let mut client =
                    AsyncClientBuilder::new(hdlc_transport, settings).build_with_heap(2048);

                // Option 2: Stack-allocated buffer (compile-time size, for embedded/no_std)
                // This is the PREFERRED option for embedded Embassy applications:
                // let mut client = AsyncClientBuilder::new(hdlc_transport, settings)
                //     .build_with_heapless::<2048>();

                println!("   ✓ Async client created (Embassy runtime)");
                println!("   DLMS Client SAP: {}", client_sap);
                println!("   DLMS Server SAP: {}", server_sap);

                println!("\n4. Establishing association...");
                // Note: This will fail without a real server
                match client.connect().await {
                    Ok(_) => {
                        println!("   ✓ Association established");

                        println!("\n5. Example async operations...");
                        println!("   (Operations would happen here with a real server)");
                        // Example async operations:
                        // - Read clock: client.read_clock().await
                        // - Read register: client.read(class_id, obis_code, attribute_id, None).await
                        // - Write data: client.write(class_id, obis_code, attribute_id, value).await

                        println!("\n6. Disconnecting...");
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
    });

    println!("\n=== Example complete ===\n");

    // Example usage patterns
    println!("API Usage Patterns (Embassy + HDLC):");
    println!("-------------------------------------");
    println!("use embassy_executor::Spawner;");
    println!("use futures_executor::block_on;");
    println!();
    println!("// For std environments (testing/development):");
    println!("fn main() -> Result<(), Box<dyn std::error::Error>> {{");
    println!("    futures_executor::block_on(async {{");
    println!("        // Create base async TCP transport with Embassy");
    println!("        let mut tcp = EmbassyTcpTransport::connect(\"192.168.1.100:4059\").await?;");
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
    println!();
    println!("        // For embedded/no_std, use heapless buffer (RECOMMENDED):");
    println!("        let mut client = AsyncClientBuilder::new(hdlc, settings)");
    println!("            .build_with_heapless::<2048>();");
    println!();
    println!("        // For std environments, heap allocation is also available:");
    println!("        // let mut client = AsyncClientBuilder::new(hdlc, settings)");
    println!("        //     .build_with_heap(2048);");
    println!();
    println!("        // Use client normally - HDLC framing is automatic and async");
    println!("        client.connect().await?;");
    println!("        let data = client.read(class_id, obis_code, attribute_id, None).await?;");
    println!("        client.disconnect().await?;");
    println!("        Ok(())");
    println!("    }})");
    println!("}}");
    println!();
    println!("// For embedded no_std environments:");
    println!("#[embassy_executor::main]");
    println!("async fn main(spawner: Spawner) {{");
    println!("    // Setup Embassy networking (embassy-net)");
    println!("    // ... network stack initialization ...");
    println!();
    println!("    // Create HDLC transport over TCP");
    println!("    let tcp = EmbassyTcpTransport::connect(\"192.168.1.100:4059\").await?;");
    println!("    let hdlc = AsyncHdlcTransport::new(tcp, 0x01, 0x10);");
    println!();
    println!("    // Use heapless buffer for no_std");
    println!("    let mut client = AsyncClientBuilder::new(hdlc, settings)");
    println!("        .build_with_heapless::<2048>();");
    println!();
    println!("    client.connect().await?;");
    println!("    // ... operations ...");
    println!("}}");
    println!();
    println!("HDLC Frame Structure (Async with Embassy):");
    println!("--------------------------------------------");
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
    println!("Embassy Architecture Benefits:");
    println!("------------------------------");
    println!("✓ Designed for embedded systems (ARM Cortex-M, RISC-V, etc.)");
    println!("✓ no_std compatible with heapless buffers");
    println!("✓ Minimal memory footprint (perfect for MCU-based meters)");
    println!("✓ Automatic frame validation (FCS checks) in async tasks");
    println!("✓ Can run on std for development/testing");
    println!("✓ Executor-agnostic design");
    println!();
    println!("Use Cases:");
    println!("----------");
    println!("- Embedded DLMS meter communication modules");
    println!("- Low-power IoT gateways with HDLC/TCP");
    println!("- ARM Cortex-M based data concentrators");
    println!("- Battery-powered remote terminal units (RTUs)");
    println!("- Cost-optimized smart meter head-end units");
    println!();
    println!("Memory Characteristics:");
    println!("-----------------------");
    println!("- Stack-based buffers (no heap allocation needed)");
    println!("- Fixed-size HDLC frame buffers (2048 bytes typical)");
    println!("- Predictable memory usage for embedded systems");
    println!("- Suitable for MCUs with 32KB+ RAM");

    Ok(())
}

#[cfg(not(all(
    feature = "async-client",
    feature = "transport-tcp-async",
    feature = "transport-hdlc-async",
    feature = "embassy"
)))]
fn main() {
    eprintln!(
        "This example requires the 'async-client', 'transport-tcp-async', 'transport-hdlc-async', and 'embassy' features."
    );
    eprintln!(
        "Run with: cargo run --example tcp_hdlc_transport_async_embassy --features async-client,transport-tcp-async,transport-hdlc-async,embassy,encode,parse"
    );
    eprintln!("\nNote: Embassy is optimized for embedded systems but can run on std for testing");
    std::process::exit(1);
}
