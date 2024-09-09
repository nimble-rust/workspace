/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use std::fmt;
use std::fmt::Formatter;
use std::io::Result;

use flood_rs::{ReadOctetStream, WriteOctetStream};

pub mod client_to_host;
pub mod client_to_host_oob;
pub mod host_to_client;
pub mod host_to_client_oob;
pub mod prelude;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

#[derive(PartialEq, Copy, Clone, Eq)]
pub struct SessionConnectionSecret {
    pub value: u64,
}

impl SessionConnectionSecret {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> Result<()> {
        stream.write_u64(self.value)
    }
    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> Result<Self> {
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
