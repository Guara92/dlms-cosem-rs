//! Basic DLMS Client Example
//!
//! This example demonstrates how to use the DLMS client to:
//! 1. Connect to a DLMS server
//! 2. Read an attribute (e.g., Register value)
//! 3. Write an attribute
//! 4. Invoke a method
//! 5. Disconnect
//!
//! Note: This is a demonstration example with a mock transport.
//! For real usage, replace MockTransport with TcpTransport or HdlcTransport.

use dlms_cosem::client::sync::{ClientBuilder, ClientSettings};
use dlms_cosem::transport::sync::Transport;
use dlms_cosem::{Data, ObisCode};

#[cfg(feature = "std")]
use std::cell::RefCell;

/// Mock transport for demonstration purposes.
/// In production, use a real transport like TcpTransport or HdlcTransport.
#[derive(Debug)]
struct MockTransport {
    recv_buffer: RefCell<Vec<u8>>,
}

impl MockTransport {
    fn new() -> Self {
        Self { recv_buffer: RefCell::new(Vec::new()) }
    }

    fn push_response(&self, data: Vec<u8>) {
        self.recv_buffer.borrow_mut().extend(data);
    }
}

impl Transport for MockTransport {
    type Error = std::io::Error;

    fn send(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        println!("→ Sent {} bytes: {:02X?}", data.len(), &data[..data.len().min(20)]);
        Ok(())
    }

    fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        let mut recv = self.recv_buffer.borrow_mut();
        if recv.is_empty() {
            return Ok(0);
        }
        let len = core::cmp::min(buffer.len(), recv.len());
        buffer[..len].copy_from_slice(&recv[..len]);
        *recv = recv.split_off(len);
        println!("← Received {} bytes", len);
        Ok(len)
    }
}

#[cfg(all(feature = "std", feature = "encode", feature = "parse"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DLMS Client Basic Example ===\n");

    // 1. Configure client settings
    let settings = ClientSettings {
        client_address: 16, // Public client
        server_address: 1,  // Management logical device
        max_pdu_size: 0xFFFF,
        ..ClientSettings::default()
    };

    println!("Client Settings:");
    println!("  Client Address: {}", settings.client_address);
    println!("  Server Address: {}", settings.server_address);
    println!("  Max PDU Size: {}\n", settings.max_pdu_size);

    // 2. Create transport (mock for this example)
    let transport = MockTransport::new();

    // Simulate server AARE response (Association Response - Accepted)
    let aare = dlms_cosem::association::AareApdu {
        protocol_version: 1,
        application_context_name:
            dlms_cosem::association::ApplicationContextName::LogicalNameReferencing,
        result: dlms_cosem::association::AssociationResult::Accepted,
        result_source_diagnostic: dlms_cosem::association::AcseServiceUserDiagnostics::Null,
        responding_ap_title: None,
        responding_ae_qualifier: None,
        responding_ap_invocation_id: None,
        responding_ae_invocation_id: None,
        responder_acse_requirements: None,
        mechanism_name: None,
        responding_authentication_value: None,
        user_information: None,
    };
    transport.push_response(aare.encode());

    // 3. Build client with heap-allocated buffer
    let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);

    println!("Buffer: Heap-allocated, 2048 bytes\n");

    // 4. Connect to server
    println!("Step 1: Connecting to DLMS server...");
    client.connect()?;
    println!("✓ Connected successfully!\n");

    // 5. Read a Register attribute (Active Energy - 1.0.1.8.0.255, attribute 2)
    println!("Step 2: Reading Register value (1.0.1.8.0.255, attribute 2)...");

    // Simulate server GET response
    let get_response = dlms_cosem::get::GetResponse::Normal(dlms_cosem::get::GetResponseNormal {
        invoke_id: 0,
        result: dlms_cosem::get::GetDataResult::Data(Data::DoubleLongUnsigned(123456)),
    });
    client.transport().push_response(get_response.encode());

    let obis = ObisCode::new(1, 0, 1, 8, 0, 255);
    let value = client.read(3, obis, 2, None)?;

    println!("✓ Read value: {:?}\n", value);

    // 6. Write a Data object attribute
    println!("Step 3: Writing Data object value (0.0.96.1.0.255, attribute 2)...");

    // Simulate server SET response
    let set_response = dlms_cosem::set::SetResponse::Normal(dlms_cosem::set::SetResponseNormal {
        invoke_id: 1,
        result: dlms_cosem::get::DataAccessResult::Success,
    });
    client.transport().push_response(set_response.encode());

    let obis = ObisCode::new(0, 0, 96, 1, 0, 255);
    let new_value = Data::OctetString(b"DLMS-RS".to_vec());
    client.write(1, obis, 2, new_value, None)?;

    println!("✓ Write successful!\n");

    // 7. Invoke a Clock method (adjust_to_quarter - method 1)
    println!("Step 4: Invoking Clock.adjust_to_quarter (0.0.1.0.0.255, method 1)...");

    // Simulate server ACTION response
    let action_response =
        dlms_cosem::action::ActionResponse::Normal(dlms_cosem::action::ActionResponseNormal {
            invoke_id: 2,
            result: dlms_cosem::action::ActionResult::Success(None),
        });
    client.transport().push_response(action_response.encode());

    let obis = ObisCode::new(0, 0, 1, 0, 0, 255);
    let result = client.method(8, obis, 1, None)?;

    println!("✓ Method invoked successfully!");
    if let Some(data) = result {
        println!("  Return data: {:?}", data);
    } else {
        println!("  No return data");
    }
    println!();

    // 8. Disconnect
    println!("Step 5: Disconnecting from server...");

    // Simulate server RLRE response
    let rlre = dlms_cosem::association::ReleaseResponseApdu {
        reason: Some(dlms_cosem::association::ReleaseResponseReason::Normal),
        user_information: None,
    };
    client.transport().push_response(rlre.encode());

    client.disconnect()?;
    println!("✓ Disconnected successfully!\n");

    println!("=== Example Complete ===");
    println!("\nFor real usage:");
    println!("  - Replace MockTransport with TcpTransport or HdlcTransport");
    println!("  - Handle errors appropriately");
    println!("  - Use proper authentication (LLS, HLS, HLS-GMAC)");
    println!("  - Enable encryption with security context (GLO/DED)");

    Ok(())
}

#[cfg(not(all(feature = "std", feature = "encode", feature = "parse")))]
fn main() {
    println!("This example requires 'std', 'encode', and 'parse' features.");
    println!("Run with: cargo run --example basic_client --all-features");
}
