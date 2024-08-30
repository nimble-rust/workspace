/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use std::io;

use flood_rs::{ReadOctetStream, WriteOctetStream};
use mash_rs::murmur3_32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConnectionSecretSeed(pub u32);

#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub struct ConnectionId {
    pub value: u8,
}

impl ConnectionId {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        stream.write_u8(self.value)
    }
    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        Ok(Self {
            value: stream.read_u8()?,
        })
    }
}

pub fn prepare_out_stream(stream: &mut dyn WriteOctetStream) -> io::Result<()> {
    let connection_id = ConnectionId { value: 0 };
    connection_id.to_stream(stream)?; // connection id must be outside the hashing
    stream.write_u32(0) // prepare hash value
}

pub struct ConnectionLayer {
    pub connection_id: ConnectionId,
    pub murmur3_hash: u32,
}

pub enum ConnectionLayerMode {
    OOB,
    Connection(ConnectionLayer),
}

impl ConnectionLayerMode {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        match self {
            ConnectionLayerMode::OOB => ConnectionId::default().to_stream(stream),
            ConnectionLayerMode::Connection(layer) => {
                layer.connection_id.to_stream(stream)?;
                stream.write_u32(layer.murmur3_hash)
            }
        }
    }
    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        let connection_id = ConnectionId::from_stream(stream)?;
        let mode = match connection_id.value {
            0 => ConnectionLayerMode::OOB,
            _ => ConnectionLayerMode::Connection(ConnectionLayer {
                connection_id,
                murmur3_hash: stream.read_u32()?,
            }),
        };

        Ok(mode)
    }
}

pub fn write_to_stream(
    stream: &mut dyn WriteOctetStream,
    connection_id: ConnectionId,
    seed: ConnectionSecretSeed,
    payload: &[u8],
) -> io::Result<()> {
    let calculated_hash = murmur3_32(payload, seed.0);
    ConnectionLayerMode::Connection(ConnectionLayer {
        connection_id,
        murmur3_hash: calculated_hash,
    })
    .to_stream(stream)
}

pub fn verify_hash(
    expected_hash: u32,
    seed: ConnectionSecretSeed,
    payload: &[u8],
) -> io::Result<()> {
    let calculated_hash = murmur3_32(payload, seed.0);
    if calculated_hash != expected_hash {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "hash mismatch: the data does not match the expected hash. {:x} vs {:x}",
                calculated_hash, expected_hash
            ),
        ))
    } else {
        Ok(())
    }
}
