/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
pub mod prelude;

use std::io::{Error, ErrorKind, Result};

use flood_rs::prelude::*;
use hexify::format_hex_u32_be;
use mash_rs::murmur3_32;

/// A seed used for generating a [Murmur3 hash](https://en.wikipedia.org/wiki/MurmurHash#MurmurHash3) for connection validation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConnectionSecretSeed(pub u32);

/// Represents a unique connection identifier for the session.
#[derive(Eq, PartialEq, Copy, Clone, Default, Debug)]
pub struct ConnectionId {
    pub value: u8,
}

impl ConnectionId {
    /// Writes the connection identifier to the provided output stream.
    ///
    /// # Arguments
    ///
    /// * `stream` - A mutable reference to a stream implementing `WriteOctetStream`.
    ///
    /// # Errors
    ///
    /// Returns an `io::Result` error if writing to the stream fails.
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> Result<()> {
        stream.write_u8(self.value)
    }

    /// Reads a connection identifier from the provided input stream.
    ///
    /// # Arguments
    ///
    /// * `stream` - A mutable reference to a stream implementing `ReadOctetStream`.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `ConnectionId` if successful, or an `io::Result` error if reading fails.
    pub fn from_stream(stream: &mut impl ReadOctetStream) -> Result<Self> {
        Ok(Self {
            value: stream.read_u8()?,
        })
    }
}

/// Prepares an output stream by writing a default connection ID and reserving space for a hash.
///
/// This function is typically used before writing the actual payload to the stream.
///
/// # Arguments
///
/// * `stream` - A mutable reference to a stream implementing `WriteOctetStream`.
///
/// # Errors
///
/// Returns an `io::Result` error if writing to the stream fails.
pub fn prepare_out_stream(stream: &mut impl WriteOctetStream) -> Result<()> {
    let connection_id = ConnectionId { value: 0 };
    connection_id.to_stream(stream)?; // connection id must be outside the hashing
    stream.write_u32(0) // prepare hash value
}

/// Represents the header of a connection with an ID and a Murmur3 hash.
#[derive(Eq, PartialEq, Debug)]
pub struct ConnectionLayer {
    pub connection_id: ConnectionId,
    pub murmur3_hash: u32,
}

/// Represents the mode of a connection layer, which can be either [Out-Of-Band (OOB)](https://en.wikipedia.org/wiki/Out-of-band_data) or an active connection.
#[derive(Eq, PartialEq, Debug)]
pub enum ConnectionLayerMode {
    OOB,
    Connection(ConnectionLayer),
}

impl ConnectionLayerMode {
    /// Serializes the `ConnectionLayerMode` into the provided output stream.
    ///
    /// # Arguments
    ///
    /// * `stream` - A mutable reference to a stream implementing [`WriteOctetStream`].
    ///
    /// # Errors
    ///
    /// Returns an `io::Result` error if writing to the stream fails.
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> Result<()> {
        match self {
            ConnectionLayerMode::OOB => ConnectionId::default().to_stream(stream),
            ConnectionLayerMode::Connection(layer) => {
                layer.connection_id.to_stream(stream)?;
                stream.write_u32(layer.murmur3_hash)
            }
        }
    }

    /// Deserializes a `ConnectionLayerMode` from the provided input stream.
    ///
    /// # Arguments
    ///
    /// * `stream` - A mutable reference to a stream implementing [`ReadOctetStream`].
    ///
    /// # Returns
    ///
    /// A `Result` containing the `ConnectionLayerMode` if successful, or an `io::Result` error if reading fails.
    pub fn from_stream(stream: &mut impl ReadOctetStream) -> Result<Self> {
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

/// Writes a connection header and a payload to the provided stream, including a Murmur3 hash for validation.
///
/// # Arguments
///
/// * `stream` - A mutable reference to a stream implementing `WriteOctetStream`.
/// * `connection_id` - The `ConnectionId` to write to the stream.
/// * `seed` - A `ConnectionSecretSeed` used for generating the Murmur3 hash.
/// * `payload` - The payload data to be written and hashed.
///
/// # Errors
///
/// Returns an `io::Result` error if writing to the stream fails.
pub fn write_to_stream(
    stream: &mut impl WriteOctetStream,
    connection_id: ConnectionId,
    seed: ConnectionSecretSeed,
    payload: &[u8],
) -> Result<()> {
    let calculated_hash = murmur3_32(payload, seed.0);
    ConnectionLayerMode::Connection(ConnectionLayer {
        connection_id,
        murmur3_hash: calculated_hash,
    })
    .to_stream(stream)
}

/// Verifies the integrity of a payload against an expected Murmur3 hash.
///
/// # Arguments
///
/// * `expected_hash` - The expected Murmur3 hash value.
/// * `seed` - The `ConnectionSecretSeed` used for generating the hash.
/// * `payload` - The payload data to be hashed and compared.
///
/// # Errors
///
/// Returns an `io::Result` error if the calculated hash does not match the expected hash.
pub fn verify_hash(expected_hash: u32, seed: ConnectionSecretSeed, payload: &[u8]) -> Result<()> {
    let calculated_hash = murmur3_32(payload, seed.0);
    if calculated_hash != expected_hash {
        Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "hash mismatch: the data does not match the expected hash. calculated {} but payload provided hash {}",
                format_hex_u32_be(calculated_hash), format_hex_u32_be(expected_hash),
            ),
        ))
    } else {
        Ok(())
    }
}
