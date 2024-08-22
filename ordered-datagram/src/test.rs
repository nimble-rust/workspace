#[cfg(test)]
use crate::{DatagramId, OrderedOut};

#[test]
fn ordered_out() {
    let out = OrderedOut {
        sequence_to_send: DatagramId(32),
    };
    assert_eq!(out.sequence_to_send.0, 32);
}

#[test]
fn test_valid() {
    assert!(DatagramId(u16::MAX).is_valid_successor(DatagramId(0)));
}

#[test]
fn test_valid_wraparound() {
    assert!(DatagramId(u16::MAX).is_valid_successor(DatagramId(80)));
}

#[test]
fn test_wrong_order() {
    assert!(!DatagramId(0).is_valid_successor(DatagramId(u16::MAX)));
}
