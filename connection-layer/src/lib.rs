use std::io;

use flood_rs::{ReadOctetStream, WriteOctetStream};
use murmur3::murmur3_32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConnectionSecretSeed(pub u32);


#[derive(PartialEq, Copy, Clone, Debug)]
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
    stream.write_u8(0x8a)?; // add marker
    stream.write_u32(0) // prepare hash value
}

pub fn write_to_stream(stream: &mut dyn WriteOctetStream, connection_id: ConnectionId, seed: ConnectionSecretSeed, payload: &[u8]) -> io::Result<()> {
    let cursor = &mut io::Cursor::new(payload);
    let calculated_hash = murmur3_32(cursor, seed.0)?;
    connection_id.to_stream(stream)?;
    stream.write_u8(0x8a)?; // add marker
    stream.write_u32(calculated_hash)
}

pub fn read_and_verify(stream: &mut dyn ReadOctetStream, payload: &[u8]) -> io::Result<()> {
    let expected_hash = stream.read_u32()?;
    let mut cursor = &mut io::Cursor::new(payload);
    let calculated_hash = murmur3_32(cursor, 0)?;
    if calculated_hash != expected_hash {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("hash mismatch: the data does not match the expected hash. {} vs {}", calculated_hash, expected_hash),
        ))
    } else {
        Ok(())
    }
}