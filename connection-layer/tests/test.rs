/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use connection_layer::prelude::*;
use flood_rs::{InOctetStream, OutOctetStream};

#[test]
fn test_header() {
    let connection = ConnectionLayerMode::Connection(ConnectionLayer {
        connection_id: ConnectionId { value: 42 },
        murmur3_hash: 0xfe334411,
    });

    let mut writer = OutOctetStream::new();
    connection.to_stream(&mut writer).expect("should work");

    let buf = writer.data;
    assert_eq!(buf[0], 42);
    assert_eq!(&buf[1..=4], &[0xfe, 0x33, 0x44, 0x11]);

    let mut reader = InOctetStream::new(buf);
    assert_eq!(
        ConnectionLayerMode::from_stream(&mut reader).expect("should work"),
        connection
    );
}
