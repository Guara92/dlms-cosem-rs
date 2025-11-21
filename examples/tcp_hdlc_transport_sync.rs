//! Synchronous TCP + HDLC transport example for DLMS/COSEM.
//!
//! This example demonstrates how to layer HDLC framing on top of TCP transport
//! for DLMS communication over TCP with HDLC encapsulation.
//!
//! Required features: `client`, `transport-tcp`, `transport-hdlc`
//!
//! # Usage
//!
//! ```bash
//! cargo run --example tcp_hdlc_transport_sync --features client,transport-tcp,transport-hdlc
//! ```
//!
//! # Note
//!
//! This is a demonstrative example showing API usage patterns.
//! For actual meter communication, replace the mock server with a real DLMS device.

#[cfg(all(feature = "client", feature = "transport-tcp", feature = "transport-hdlc"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use dlms_cosem::client::sync::{ClientBuilder, ClientSettings};
    use dlms_cosem::transport::hdlc::HdlcTransport;
    use dlms_cosem::transport::tcp::TcpTransport;
    use std::time::Duration;

    println!("=== DLMS TCP + HDLC Transport Example ===\n");

    // Connection parameters
    let server_address = "192.168.1.100:4059";
    let hdlc_client_address = 0x01; // HDLC client address
    let hdlc_server_address = 0x10; // HDLC server physical address
    let client_sap = 16; // DLMS client SAP
    let server_sap = 1; // DLMS server SAP

    println!("1. Creating layered transport (TCP + HDLC)...");
    println!("   TCP Server: {}", server_address);
    println!("   HDLC Client Address: 0x{:02X}", hdlc_client_address);
    println!("   HDLC Server Address: 0x{:02X}", hdlc_server_address);

    // Create base TCP transport
    match TcpTransport::connect(server_address) {
        Ok(mut tcp_transport) => {
            println!("   ✓ TCP connection established");

            // Configure TCP timeouts
            tcp_transport
                .set_read_timeout(Some(Duration::from_secs(30)))
                .expect("Failed to set read timeout");
            tcp_transport
                .set_write_timeout(Some(Duration::from_secs(30)))
                .expect("Failed to set write timeout");

            println!("   ✓ TCP timeouts configured");

            // Wrap TCP transport with HDLC framing
            let hdlc_transport =
                HdlcTransport::new(tcp_transport, hdlc_client_address, hdlc_server_address);

            println!("   ✓ HDLC wrapper added");
            println!("   Transport stack: DLMS -> HDLC -> TCP -> Network");

            println!("\n2. Creating DLMS client...");

            // Configure client settings
            let settings = ClientSettings {
                client_address: client_sap,
                server_address: server_sap,
                ..Default::default()
            };

            // Create DLMS client with layered transport
            let mut client = ClientBuilder::new(hdlc_transport, settings).build_with_heap(2048);

            println!("   ✓ Client created");
            println!("   DLMS Client SAP: {}", client_sap);
            println!("   DLMS Server SAP: {}", server_sap);

            println!("\n3. Establishing association...");
            // Note: This will fail without a real server
            match client.connect() {
                Ok(_) => {
                    println!("   ✓ Association established");

                    println!("\n4. Example operations...");
                    println!("   (Operations would happen here with a real server)");
                    // Example operations:
                    // - Read clock: client.read_clock()
                    // - Read register: client.read(class_id, obis_code, attribute_id)
                    // - Write data: client.write(class_id, obis_code, attribute_id, value)

                    println!("\n5. Disconnecting...");
                    match client.disconnect() {
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

    println!("\n=== Example complete ===\n");

    // Example usage patterns
    println!("API Usage Patterns:");
    println!("-------------------");
    println!("// Create base TCP transport");
    println!("let tcp = TcpTransport::connect(\"192.168.1.100:4059\")?;");
    println!();
    println!("// Wrap with HDLC framing");
    println!("let hdlc = HdlcTransport::new(");
    println!("    tcp,");
    println!("    0x01,  // HDLC client address");
    println!("    0x10,  // HDLC server address");
    println!(");");
    println!();
    println!("// Create DLMS client with layered transport");
    println!("let settings = ClientSettings {{");
    println!("    client_address: 16,  // DLMS client SAP");
    println!("    server_address: 1,   // DLMS server SAP");
    println!("    ..Default::default()");
    println!("}};");
    println!("let mut client = ClientBuilder::new(hdlc, settings)");
    println!("    .build_with_heap(2048);");
    println!();
    println!("// Use client normally - HDLC framing is automatic");
    println!("client.connect()?;");
    println!("let data = client.read(class_id, obis_code, attribute_id)?;");
    println!("client.disconnect()?;");
    println!();
    println!("HDLC Frame Structure:");
    println!("---------------------");
    println!("Flag | Format | Length | Dest | Src | Ctrl | HCS | LLC | APDU | FCS | Flag");
    println!("0x7E |   1B   |  1-2B  | 1-4B |1-4B | 1B   | 2B  | 3B  |  nB  | 2B  | 0x7E");
    println!();
    println!("Where:");
    println!("  - Dest/Src: HDLC addresses (0x01, 0x10 in this example)");
    println!("  - LLC: Logical Link Control header (0xE6 0xE6 0x00)");
    println!("  - APDU: DLMS application protocol data");
    println!("  - HCS/FCS: Frame check sequences (CRC-16)");

    Ok(())
}

#[cfg(not(all(feature = "client", feature = "transport-tcp", feature = "transport-hdlc")))]
fn main() {
    eprintln!(
        "This example requires the 'client', 'transport-tcp', and 'transport-hdlc' features."
    );
    eprintln!(
        "Run with: cargo run --example tcp_hdlc_transport_sync --features client,transport-tcp,transport-hdlc"
    );
    std::process::exit(1);
}
