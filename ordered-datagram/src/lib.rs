/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
mod test;

use flood_rs::{ReadOctetStream, WriteOctetStream};
use std::io::ErrorKind;
use std::{fmt, io};

#[derive(Debug, Default, Copy, Clone)]
pub struct DatagramId(u16);

impl DatagramId {
    fn to_stream(self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u16(self.0)
    }

    #[allow(unused)]
    fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<DatagramId> {
        Ok(Self(stream.read_u16()?))
    }

    fn diff(self, after: DatagramId) -> i32 {
        after.0.wrapping_sub(self.0) as i32
    }

    #[allow(unused)]
    fn is_valid_successor(self, after: DatagramId) -> bool {
        const ORDERED_DATAGRAM_ID_ACCEPTABLE_DIFF: i32 = 625; // 10 datagrams / tick * tickFrequency (62.5) * 1 second latency
        let diff = self.diff(after);
        diff > 0 && diff <= ORDERED_DATAGRAM_ID_ACCEPTABLE_DIFF
    }

    fn is_equal_or_successor(self, after: DatagramId) -> bool {
        const ORDERED_DATAGRAM_ID_ACCEPTABLE_DIFF: i32 = 625; // 10 datagrams / tick * tickFrequency (62.5) * 1 second latency
        let diff = self.diff(after);
        (0..=ORDERED_DATAGRAM_ID_ACCEPTABLE_DIFF).contains(&diff)
    }
}

impl fmt::Display for DatagramId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DatagramId({:X})", self.0)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct OrderedOut {
    pub sequence_to_send: DatagramId,
}

impl OrderedOut {
    pub fn new() -> Self {
        Self {
            sequence_to_send: DatagramId(0),
        }
    }
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        self.sequence_to_send.to_stream(stream)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct OrderedIn {
    expected_sequence: DatagramId,
}

impl OrderedIn {
    pub fn read_and_verify(&mut self, stream: &mut dyn ReadOctetStream) -> io::Result<()> {
        let potential_expected_or_successor = DatagramId::from_stream(stream)?;

        if self
            .expected_sequence
            .is_equal_or_successor(potential_expected_or_successor)
        {
            self.expected_sequence = potential_expected_or_successor;
            Ok(())
        } else {
            Err(io::Error::new(
                ErrorKind::InvalidData,
                format!(
                    "wrong datagram order. expected {} but received {}",
                    self.expected_sequence, potential_expected_or_successor
                ),
            ))
        }
    }
}
