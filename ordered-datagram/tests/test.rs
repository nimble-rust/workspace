/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
#[cfg(test)]
use ordered_datagram::{DatagramId, OrderedOut};

#[test]
fn ordered_out() {
    let out = OrderedOut {
        sequence_to_send: DatagramId::new(32),
    };
    assert_eq!(out.sequence_to_send.inner(), 32);
}

#[test]
fn test_valid() {
    assert!(DatagramId::new(u16::MAX).is_valid_successor(DatagramId::new(0)));
}

#[test]
fn test_valid_wraparound() {
    assert!(DatagramId::new(u16::MAX).is_valid_successor(DatagramId::new(80)));
}

#[test]
fn test_wrong_order() {
    assert!(!DatagramId::new(0).is_valid_successor(DatagramId::new(u16::MAX)));
}
