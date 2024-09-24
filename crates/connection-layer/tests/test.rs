/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use datagram::{DatagramDecoder, DatagramEncoder};
use flood_rs::prelude::*;
use nimble_connection_layer::datagram_builder::*;
use nimble_connection_layer::prelude::*;
use secure_random::SecureRandom;
use std::io;

#[test_log::test]
fn test_header() {
    let connection = ConnectionLayerMode::Connection(ConnectionLayer {
        connection_id: ConnectionId { value: 42 },
        murmur3_hash: 0xfe334411,
    });

    let mut writer = OutOctetStream::new();
    connection.to_stream(&mut writer).expect("should work");

    let buf = writer.octets_ref();
    assert_eq!(buf[0], 42);
    assert_eq!(&buf[1..=4], &[0xfe, 0x33, 0x44, 0x11]);

    let mut reader = InOctetStream::new(buf);
    assert_eq!(
        ConnectionLayerMode::from_stream(&mut reader).expect("should work"),
        connection
    );
}

#[derive(Debug)]
pub struct FakeRandom {
    pub counter: u64,
}

impl SecureRandom for FakeRandom {
    fn get_random_u64(&mut self) -> u64 {
        self.counter += 1;
        self.counter
    }
}

#[test_log::test]
fn codec() -> io::Result<()> {
    // Setup
    let request_id: RequestId = 0x0001020304050607;
    let mut client_codec = ConnectionLayerClientCodec::new(request_id);

    let random2 = FakeRandom { counter: 0 };
    let boxed_random2 = Box::new(random2);
    let mut host_codec = ConnectionLayerHostCodec::new(boxed_random2);

    // First message Client -> Host
    let test_octets = &[b'h', b'e', b'l', b'l', b'o'];
    let data_to_send = client_codec.encode(test_octets)?;

    // Verify
    #[rustfmt::skip]
    let expected_test_octets = &[
        0, // Connection ID. Zero is OOB
        0x05, // Connect Request
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, // Request ID
        0x00, 0x02, // Connection Layer Version
        b'h', b'e', b'l', b'l', b'o'];
    hexify::assert_eq_slices(&data_to_send, expected_test_octets);
    let (connection_id, decoded) = host_codec.decode(data_to_send.as_slice())?;
    assert_eq!(decoded, test_octets);

    // Host -> Client
    const EXPECTED_CONNECTION_ID: u8 = 1;
    assert_eq!(connection_id, EXPECTED_CONNECTION_ID);

    let test_reply_octets = &[b'w', b'o', b'r', b'l', b'd', b'!'];

    let host_to_client_reply = host_codec.encode(connection_id, test_reply_octets)?;
    #[rustfmt::skip]
    let expected_host_to_client_reply = &[
        0, // Connection Id.
        0x06, // Connect Response
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, // Request ID
        EXPECTED_CONNECTION_ID, // Created connection id
        0x00, 0x00, 0x00, 0x01,  // Secret seed
        b'w', b'o', b'r', b'l', b'd', b'!'];
    hexify::assert_eq_slices(&host_to_client_reply, expected_host_to_client_reply);

    let client_received_reply = client_codec.decode(&host_to_client_reply)?;
    hexify::assert_eq_slices(&client_received_reply, test_reply_octets);

    // Client -> Host
    let test_after_connected_octets = &[b'l', b'o', b'v', b'e', b'l', b'y'];
    let to_host_after_connected = client_codec.encode(test_after_connected_octets)?;

    // verify
    #[rustfmt::skip]
    let expected_to_host_after_connected = &[
        EXPECTED_CONNECTION_ID, // client should be connected now, so start using the client connection ID
        19, 215, 173, 162,  // Hash for this content
        b'l', b'o', b'v', b'e', b'l', b'y'];
    hexify::assert_eq_slices(&to_host_after_connected, expected_to_host_after_connected);

    let (found_connection_id, from_client_after_connected) =
        host_codec.decode(&*to_host_after_connected)?;
    hexify::assert_eq_slices(&*from_client_after_connected, test_after_connected_octets);
    assert_eq!(found_connection_id, EXPECTED_CONNECTION_ID);

    let host_to_client_msg_after_connected = &[b'w', b'o', b'r', b'k', b's'];
    let host_to_client_after_connected =
        host_codec.encode(connection_id, host_to_client_msg_after_connected)?;

    // verify
    #[rustfmt::skip]
    let expected_to_client_after_connected = &[
        EXPECTED_CONNECTION_ID, // host should know that client is connected, so it will use connection_id without any command
        129, 15, 178, 151, // Hash for this content
        b'w', b'o', b'r', b'k', b's'];
    hexify::assert_eq_slices(
        &host_to_client_after_connected,
        expected_to_client_after_connected,
    );

    Ok(())
}
