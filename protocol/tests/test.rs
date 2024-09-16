/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use flood_rs::prelude::*;

use nimble_protocol::client_to_host_oob::ConnectRequest;
use nimble_protocol::{Nonce, Version};

#[test]
fn check_version() {
    let mut out_stream = OutOctetStream::new();
    let version = Version {
        major: 4,
        minor: 3,
        patch: 2,
    };
    version.to_stream(&mut out_stream).unwrap()
}

#[test]
fn check_connect() {
    let mut out_stream = OutOctetStream::new();
    let version = Version {
        major: 4,
        minor: 3,
        patch: 2,
    };
    let nimble_version = Version {
        major: 99,
        minor: 66,
        patch: 33,
    };
    let connect = ConnectRequest {
        nimble_version,
        use_debug_stream: false,
        application_version: version,
        nonce: Nonce(0xff4411ff),
    };
    connect.to_stream(&mut out_stream).unwrap();

    let mut in_stream = InOctetStream::new(out_stream.octets_ref());

    let received_connect = ConnectRequest::from_stream(&mut in_stream).unwrap();

    assert_eq!(received_connect, connect);
}
