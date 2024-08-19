/*----------------------------------------------------------------------------------------------------------
 *  Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------------------*/
use std::fmt;
use std::fmt::Formatter;
use std::io::{Error, ErrorKind, Result};

use flood_rs::{ReadOctetStream, WriteOctetStream};

pub mod client_to_host;
pub mod host_to_client;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Nonce(pub u64);

impl fmt::Display for Nonce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Nonce({:X})", self.0)
    }
}

impl Nonce {
    pub fn new(value: u64) -> Nonce {
        Self(value)
    }
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> Result<()> {
        stream.write_u64(self.0)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> Result<Self> {
        let x = stream.read_u64()?;
        Ok(Self(x))
    }
}

pub fn hex_output(data: &[u8]) -> String {
    let mut hex_string = String::new();
    for byte in data {
        hex_string.push_str(&format!("{:02X} ", byte));
    }
    hex_string.trim_end().to_string() // Remove the trailing space and convert to String
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl Version {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> Result<()> {
        stream.write_u16(self.major)?;
        stream.write_u16(self.minor)?;
        stream.write_u16(self.patch)?;

        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> Result<Self> {
        Ok(Self {
            major: stream.read_u16()?,
            minor: stream.read_u16()?,
            patch: stream.read_u16()?,
        })
    }
}


#[derive(PartialEq, Copy, Clone, Debug)]
pub struct ParticipantId {
    pub value: u8,
}

impl ParticipantId {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> Result<()> {
        stream.write_u8(self.value)
    }
    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> Result<Self> {
        Ok(Self {
            value: stream.read_u8()?,
        })
    }
}

pub fn write_marker(stream: &mut dyn WriteOctetStream, marker: u8) -> Result<()> {
    stream.write_u8(marker)
}

pub fn read_marker(stream: &mut dyn ReadOctetStream, expected_marker: u8) -> Result<()> {
    let found_marker = stream.read_u8()?;

    if found_marker == expected_marker {
        return Ok(());
    }

    Err(Error::new(
        ErrorKind::InvalidData,
        "Encountered wrong marker",
    ))
}

#[derive(PartialEq, Copy, Clone)]
pub struct SessionConnectionSecret {
    pub value: u64,
}

impl SessionConnectionSecret {
    const MARKER: u8 = 0x68;
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> Result<()> {
        stream.write_debug_marker(SessionConnectionSecret::MARKER)?;
        stream.write_u64(self.value)
    }
    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> Result<Self> {
        stream.verify_debug_marker(SessionConnectionSecret::MARKER)?;
        Ok(Self {
            value: stream.read_u64()?,
        })
    }
}

impl fmt::Display for SessionConnectionSecret {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "session_secret: {:X}", self.value)
    }
}

impl fmt::Debug for SessionConnectionSecret {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "session_secret: {:X}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use flood_rs::{InOctetStream, OutOctetStream};

    use crate::{Nonce, Version};
    use crate::client_to_host::ConnectRequest;

    #[test]
    fn check_version() {
        let mut out_stream = OutOctetStream::new();
        let version = Version {
            major: 4,
            minor: 3,
            patch: 2,
        };
        version.to_stream(&mut out_stream).unwrap()
    }

    #[test]
    fn check_connect() {
        let mut out_stream = OutOctetStream::new();
        let version = Version {
            major: 4,
            minor: 3,
            patch: 2,
        };
        let nimble_version = Version {
            major: 99,
            minor: 66,
            patch: 33,
        };
        let connect = ConnectRequest {
            nimble_version,
            use_debug_stream: false,
            application_version: version,
            nonce: Nonce(0xff4411ff),
        };
        connect.to_stream(&mut out_stream).unwrap();

        let mut in_stream = InOctetStream::new(Vec::from(out_stream.data));

        let received_connect = ConnectRequest::from_stream(&mut in_stream).unwrap();

        assert_eq!(received_connect, connect);
    }
}
