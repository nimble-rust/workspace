/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use udp_client::UdpClient;

#[test]
fn it_works() {
    let client = UdpClient::new("localhost:23000").unwrap();
    client.send_datagram(&[0x18, 0x28]).unwrap();
}
