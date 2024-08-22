mod test;

use std::io;

use flood_rs::{ReadOctetStream, WriteOctetStream};

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

fn murmur3_x86_32(data: &[u8], seed: u32) -> u32 {
    let mut h1 = seed;
    let c1 = 0xcc9e2d51;
    let c2 = 0x1b873593;

    // Process each 4-byte block of the input
    let len = data.len();
    let num_blocks = len / 4;

    for i in 0..num_blocks {
        let start = i * 4;
        let k1 = u32::from_le_bytes(data[start..start + 4].try_into().unwrap());

        let k1 = {
            let mut k1 = k1;
            k1 = k1.wrapping_mul(c1);
            k1 = k1.rotate_left(15);
            k1 = k1.wrapping_mul(c2);
            k1
        };

        h1 = h1 ^ k1;
        h1 = h1.rotate_left(13);
        h1 = h1.wrapping_mul(5).wrapping_add(0xe6546b64);
    }

    // Handle the remaining bytes
    let tail = &data[num_blocks * 4..];
    let mut k1 = 0u32;

    match tail.len() {
        3 => {
            k1 ^= (tail[2] as u32) << 16;
            k1 ^= (tail[1] as u32) << 8;
            k1 ^= tail[0] as u32;
            k1 = k1.wrapping_mul(c1);
            k1 = k1.rotate_left(15);
            k1 = k1.wrapping_mul(c2);
            h1 ^= k1;
        }
        2 => {
            k1 ^= (tail[1] as u32) << 8;
            k1 ^= tail[0] as u32;
            k1 = k1.wrapping_mul(c1);
            k1 = k1.rotate_left(15);
            k1 = k1.wrapping_mul(c2);
            h1 ^= k1;
        }
        1 => {
            k1 ^= tail[0] as u32;
            k1 = k1.wrapping_mul(c1);
            k1 = k1.rotate_left(15);
            k1 = k1.wrapping_mul(c2);
            h1 ^= k1;
        }
        _ => (),
    }

    // Finalization
    h1 ^= len as u32;
    h1 = h1 ^ (h1 >> 16);
    h1 = h1.wrapping_mul(0x85ebca6b);
    h1 = h1 ^ (h1 >> 13);
    h1 = h1.wrapping_mul(0xc2b2ae35);
    h1 = h1 ^ (h1 >> 16);

    h1
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
    let calculated_hash = murmur3_x86_32(payload, seed.0);
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
    let calculated_hash = murmur3_x86_32(payload, seed.0);
    if calculated_hash != expected_hash {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "hash mismatch: the data does not match the expected hash. {} vs {}",
                calculated_hash, expected_hash
            ),
        ))
    } else {
        Ok(())
    }
}
