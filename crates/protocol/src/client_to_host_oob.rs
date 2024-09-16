/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use crate::{Nonce, Version};
use flood_rs::{ReadOctetStream, WriteOctetStream};
use std::io::ErrorKind;
use std::{fmt, io};

#[repr(u8)]
enum ClientToHostOobCommand {
    Connect = 0x05,
}

impl TryFrom<u8> for ClientToHostOobCommand {
    type Error = io::Error;

    fn try_from(value: u8) -> io::Result<Self> {
        match value {
            0x05 => Ok(ClientToHostOobCommand::Connect),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Unknown command {}", value),
            )),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ConnectRequest {
    pub nimble_version: Version,
    pub use_debug_stream: bool,
    pub application_version: Version,
    pub nonce: Nonce,
}

impl ConnectRequest {
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        self.nimble_version.to_stream(stream)?;
        stream.write_u8(if self.use_debug_stream { 0x01 } else { 0x00 })?;
        self.application_version.to_stream(stream)?;
        self.nonce.to_stream(stream)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        Ok(Self {
            nimble_version: Version::from_stream(stream)?,
            use_debug_stream: stream.read_u8()? != 0,
            application_version: Version::from_stream(stream)?,
            nonce: Nonce::from_stream(stream)?,
        })
    }
}

#[derive(Debug, Clone)]
pub enum ClientToHostOobCommands {
    ConnectType(ConnectRequest),
}

impl ClientToHostOobCommands {
    pub fn to_octet(&self) -> u8 {
        match self {
            ClientToHostOobCommands::ConnectType(_) => ClientToHostOobCommand::Connect as u8,
        }
    }

    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.to_octet())?;
        match self {
            ClientToHostOobCommands::ConnectType(connect_command) => {
                connect_command.to_stream(stream)
            }
        }
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        let command_value = stream.read_u8()?;
        let command = ClientToHostOobCommand::try_from(command_value)?;
        let x = match command {
            ClientToHostOobCommand::Connect => {
                ClientToHostOobCommands::ConnectType(ConnectRequest::from_stream(stream)?)
            }
        };
        Ok(x)
    }
}

impl fmt::Display for ClientToHostOobCommands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientToHostOobCommands::ConnectType(connect) => write!(f, "connect {:?}", connect),
        }
    }
}
