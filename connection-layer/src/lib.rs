use std::io;

use flood_rs::{ReadOctetStream, WriteOctetStream};
use murmur3::murmur3_32;

pub fn prepare_out_stream(stream: &mut dyn WriteOctetStream) -> io::Result<()> {
    stream.write_u8(0x8a)?; // add marker
    stream.write_u32(0) // prepare hash value
}

pub fn write_to_stream(stream: &mut dyn WriteOctetStream, payload: &[u8]) -> io::Result<()> {
    let cursor = &mut io::Cursor::new(payload);
    let calculated_hash = murmur3_32(cursor, 0)?;
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