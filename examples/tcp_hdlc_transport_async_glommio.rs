//! Asynchronous TCP + HDLC transport example with Glommio runtime for DLMS/COSEM.
//!
//! This example demonstrates how to layer HDLC framing on top of async TCP transport
//! using Glommio runtime (Linux io_uring, thread-per-core) for DLMS communication
//! over TCP with HDLC encapsulation.
//!
//! Required features: `async-client`, `transport-tcp-async`, `transport-hdlc-async`, `glommio`
//!
//! # Usage
//!
//! ```bash
//! cargo run --example tcp_hdlc_transport_async_glommio --features async-client,transport-tcp-async,transport-hdlc-async,glommio,encode,parse
//! ```
//!
//! # Note
//!
//! This is a demonstrative example showing API usage patterns.
//! For actual meter communication, replace the mock server with a real DLMS device.
//!
//! Glommio is optimized for Linux with io_uring and uses a thread-per-core architecture.

#[cfg(all(
    feature = "async-client",
    feature = "transport-tcp-async",
    feature = "transport-hdlc-async",
    feature = "glommio"
))]
fn main() -> Result<(), std::io::Error> {
    use dlms_cosem::async_client::AsyncClientBuilder;
    use dlms_cosem::client::ClientSettings;
    use dlms_cosem::transport::hdlc::AsyncHdlcTransport;
    use dlms_cosem::transport::tcp::GlommioTcpTransport;
    use glommio::LocalExecutorBuilder;
    use std::time::Duration;

    println!("=== DLMS Async TCP + HDLC Transport Example (Glommio) ===\n");

    // Connection parameters
    let server_address = "192.168.1.100:4059";
    let hdlc_client_address = 0x01; // HDLC client address
    let hdlc_server_address = 0x10; // HDLC server physical address
    let client_sap = 16; // DLMS client SAP
    let server_sap = 1; // DLMS server SAP

    println!("1. Creating Glommio executor (thread-per-core architecture)...");
    println!("   Runtime: Glommio (Linux io_uring)");
    println!("   Architecture: Thread-per-core (single-threaded tasks)");

    // Create Glommio local executor
    let local_ex = LocalExecutorBuilder::default().spawn(move || async move {
        println!("   ✓ Glommio executor spawned");

        println!("\n2. Creating layered async transport (TCP + HDLC)...");
        println!("   TCP Server: {}", server_address);
        println!("   HDLC Client Address: 0x{:02X}", hdlc_client_address);
        println!("   HDLC Server Address: 0x{:02X}", hdlc_server_address);

        // Create base async TCP transport with Glommio
        match GlommioTcpTransport::connect(server_address).await {
            Ok(mut tcp_transport) => {
                println!("   ✓ Async TCP connection established (io_uring)");

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
                println!("   Transport stack: DLMS -> HDLC -> TCP -> io_uring (Glommio)");

                println!("\n3. Creating async DLMS client...");

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

                println!("   ✓ Async client created (Glommio runtime)");
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
                println!("   3. Run the example again on Linux with io_uring support");
            }
        }

        Ok::<(), std::io::Error>(())
    })?;

    // Run the executor
    let _ = local_ex.join()?;

    println!("\n=== Example complete ===\n");

    // Example usage patterns
    println!("API Usage Patterns (Glommio + HDLC):");
    println!("-------------------------------------");
    println!("use glommio::LocalExecutorBuilder;");
    println!();
    println!("fn main() -> Result<(), std::io::Error> {{");
    println!("    let local_ex = LocalExecutorBuilder::default()");
    println!("        .spawn(move || async move {{");
    println!("            // Create base async TCP transport with Glommio");
    println!(
        "            let mut tcp = GlommioTcpTransport::connect(\"192.168.1.100:4059\").await?;"
    );
    println!("            tcp.set_read_timeout(Some(Duration::from_secs(30)));");
    println!();
    println!("            // Wrap with async HDLC framing");
    println!("            let hdlc = AsyncHdlcTransport::new(");
    println!("                tcp,");
    println!("                0x01,  // HDLC client address");
    println!("                0x10,  // HDLC server address");
    println!("            );");
    println!();
    println!("            // Create async DLMS client with layered transport");
    println!("            let settings = ClientSettings {{");
    println!("                client_address: 16,  // DLMS client SAP");
    println!("                server_address: 1,   // DLMS server SAP");
    println!("                ..Default::default()");
    println!("            }};");
    println!("            let mut client = AsyncClientBuilder::new(hdlc, settings)");
    println!("                .build_with_heap(2048);");
    println!();
    println!("            // For no_std/embedded, use heapless buffer:");
    println!("            // let mut client = AsyncClientBuilder::new(hdlc, settings)");
    println!("            //     .build_with_heapless::<2048>();");
    println!();
    println!("            // Use client normally - HDLC framing is automatic and async");
    println!("            client.connect().await?;");
    println!("            let data = client.read(class_id, obis_code, attribute_id, None).await?;");
    println!("            client.disconnect().await?;");
    println!("            Ok(())");
    println!("        }})?;");
    println!("    local_ex.join()");
    println!("}}");
    println!();
    println!("HDLC Frame Structure (Async with Glommio):");
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
    println!("Glommio Architecture Benefits:");
    println!("------------------------------");
    println!("✓ Linux io_uring for zero-copy I/O");
    println!("✓ Thread-per-core architecture (no Send bounds needed)");
    println!("✓ Ultra-low latency HDLC frame processing");
    println!("✓ Perfect for high-performance HES systems (1,000-10,000 meters)");
    println!("✓ Automatic frame validation (FCS checks) with io_uring efficiency");
    println!("✓ Minimal syscall overhead for serial-over-IP gateways");
    println!();
    println!("Performance Characteristics:");
    println!("---------------------------");
    println!("- Single-threaded per executor (no lock contention)");
    println!("- Direct io_uring integration for TCP operations");
    println!("- Optimized for batch meter reading scenarios");
    println!("- Ideal for dedicated polling threads in data concentrators");

    Ok(())
}

#[cfg(not(all(
    feature = "async-client",
    feature = "transport-tcp-async",
    feature = "transport-hdlc-async",
    feature = "glommio"
)))]
fn main() {
    eprintln!(
        "This example requires the 'async-client', 'transport-tcp-async', 'transport-hdlc-async', and 'glommio' features."
    );
    eprintln!(
        "Run with: cargo run --example tcp_hdlc_transport_async_glommio --features async-client,transport-tcp-async,transport-hdlc-async,glommio,encode,parse"
    );
    eprintln!("\nNote: Glommio requires Linux with io_uring support (kernel 5.8+)");
    std::process::exit(1);
}
