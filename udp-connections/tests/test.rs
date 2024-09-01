/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use secure_random::SecureRandom;

use udp_connections::{Client, DatagramProcessor};

pub struct FakeRandom {
    pub counter: u64,
}

impl SecureRandom for FakeRandom {
    fn get_random_u64(&mut self) -> u64 {
        self.counter += 1;
        self.counter
    }
}

#[test]
fn simple_connection() {
    let random = FakeRandom { counter: 2 };

    let mut client = Client::new(Box::new(random));

    let example = vec![0x18, 0x24, 0x32];

    let datagram_to_send = client
        .send_datagram(example.as_slice())
        .expect("TODO: panic message");

    let expected = vec![
        1, // Challenge command 0x01
        0, 0, 0, 0, 0, 0, 0, 3, // Nonce in network order.
        0x18, 0x24, 0x32,
    ];
    assert_eq!(datagram_to_send, expected, "upd-connections-was wrong")
}
