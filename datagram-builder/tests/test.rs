/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use datagram_builder::deserialize::{deserialize_datagrams, DatagramParser};
use datagram_builder::{serialize_datagrams, DatagramBuilder};
use flood_rs::prelude::*;
use std::io;

#[derive(Debug, PartialEq)]
enum TestItem {
    Name { input: String },
    Amount(i16),
}

impl Serialize for TestItem {
    fn serialize(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        match self {
            TestItem::Name { input } => {
                stream.write_u8(0x01)?;
                stream.write_u8(input.len() as u8)?;
                stream.write(input.as_bytes())?;
            }
            TestItem::Amount(amount) => {
                stream.write_u8(0x02)?;
                stream.write_i16(*amount)?;
            }
        }
        Ok(())
    }
}

impl Deserialize for TestItem {
    fn deserialize(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        let type_id = stream.read_u8()?;
        let cmd = match type_id {
            0x01 => {
                let len = stream.read_u8()? as usize;
                let mut vec = vec![0u8; len];
                stream.read(&mut vec)?;
                Self::Name {
                    input: String::from_utf8(vec)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
                }
            }

            0x02 => {
                let amount = stream.read_i16()?;
                Self::Amount(amount)
            }

            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "unknown type"))?,
        };
        Ok(cmd)
    }
}

struct ExampleDatagramBuilder;

impl DatagramBuilder for ExampleDatagramBuilder {
    fn start_datagram(&mut self, buffer: &mut [u8]) -> io::Result<(usize, usize)> {
        buffer[0] = 0x01;
        buffer[1] = 0xff;
        Ok((2, buffer.len() - 1))
    }

    fn end_datagram(&mut self, buffer: &mut [u8], payload_len: usize) -> io::Result<usize> {
        buffer[0] = 0x02;
        buffer[payload_len + 2] = 0x00;

        Ok(3 + payload_len)
    }
}

struct ExampleDatagramParser;

impl DatagramParser for ExampleDatagramParser {
    fn parse<'a>(&mut self, datagram: &'a [u8]) -> io::Result<&'a [u8]> {
        assert_eq!(datagram[0], 0x02);
        assert_eq!(datagram[1], 0xff);

        Ok(datagram[2..datagram.len() - 1].as_ref())
    }
}

#[test]
fn serialize_single_datagram() {
    let items = [
        TestItem::Name {
            input: "foo".into(),
        },
        TestItem::Amount(42),
        TestItem::Name {
            input: "bar".into(),
        },
    ];
    const EXPECTED_DATAGRAM: &[u8] = &[
        0x02, 0xff, 0x01, 3, b'f', b'o', b'o', 0x02, 0x00, 42, 0x01, 3, b'b', b'a', b'r', 0x00,
    ];
    let mut builder = ExampleDatagramBuilder;
    let datagrams = serialize_datagrams(&items, 1024, &mut builder).expect("serialization failed");
    assert_eq!(datagrams.len(), 1);
    assert_eq!(datagrams[0], EXPECTED_DATAGRAM);

    let mut parser = ExampleDatagramParser;
    let deserialized_items: Vec<TestItem> =
        deserialize_datagrams(datagrams, &mut parser).expect("deserialization failed");
    assert_eq!(items.len(), deserialized_items.len());
    assert_eq!(*deserialized_items, items);
}

#[test]
fn serialize_and_deserialize_multiple_datagrams() {
    const MAX_PACKET_SIZE: usize = 8;
    let items = [
        TestItem::Name {
            input: "foo".into(),
        },
        TestItem::Amount(42),
        TestItem::Name {
            input: "bar".into(),
        },
    ];

    const EXPECTED_DATAGRAMS: [&[u8]; 3] = [
        &[
            0x02, 0xff, // header
            0x01, 3, 'f' as u8, 'o' as u8, 'o' as u8, 0x00, // end of datagram
        ],
        &[
            0x02, 0xff, // header
            0x02, 0x00, 42, 0x00, // end of datagram
        ],
        &[
            0x02, 0xff, // header
            0x01, 3, 'b' as u8, 'a' as u8, 'r' as u8, 0x00, // end of datagram
        ],
    ];

    const EXPECTED_DATAGRAM_COUNT: usize = 3;

    let mut builder = ExampleDatagramBuilder;
    let datagrams =
        serialize_datagrams(&items, MAX_PACKET_SIZE, &mut builder).expect("serialization failed");
    assert_eq!(datagrams.len(), EXPECTED_DATAGRAM_COUNT);

    for i in 0..EXPECTED_DATAGRAM_COUNT {
        assert_eq!(datagrams[i].len(), EXPECTED_DATAGRAMS[i].len());
        assert_eq!(datagrams[i], EXPECTED_DATAGRAMS[i]);
    }

    let mut parser = ExampleDatagramParser;
    let deserialized_items: Vec<TestItem> =
        deserialize_datagrams(datagrams, &mut parser).expect("deserialization failed");
    assert_eq!(items.len(), deserialized_items.len());
}
