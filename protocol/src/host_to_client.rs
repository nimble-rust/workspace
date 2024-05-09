/*----------------------------------------------------------------------------------------------------------
 *  Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------------------*/
use std::io;
use std::io::ErrorKind;

use flood_rs::{ReadOctetStream, WriteOctetStream};

use crate::{ConnectionId, Nonce, SessionConnectionSecret};

#[repr(u8)]
enum HostToClientCommand {
    Challenge = 0x11,
    Connect = 0x0d,
    Packet = 0x13,
}

impl TryFrom<u8> for HostToClientCommand {
    type Error = io::Error;

    fn try_from(value: u8) -> std::io::Result<Self> {
        match value {
            0x11 => Ok(HostToClientCommand::Challenge),
            0x0d => Ok(HostToClientCommand::Connect),
            0x13 => Ok(HostToClientCommand::Packet),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Unknown command {}", value),
            )),
        }
    }
}

#[derive(Debug)]
pub enum HostToClientCommands {
    ConnectType(ConnectionAccepted),
}

impl HostToClientCommands {
    pub fn to_octet(&self) -> u8 {
        match self {
            HostToClientCommands::ConnectType(_) => HostToClientCommand::Connect as u8,
        }
    }

    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        stream.write_u8(self.to_octet())?;
        match self {
            HostToClientCommands::ConnectType(connect_command) => connect_command.to_stream(stream),
        }
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        let command_value = stream.read_u8()?;
        let command = HostToClientCommand::try_from(command_value)?;
        let x = match command {
            HostToClientCommand::Connect => {
                HostToClientCommands::ConnectType(ConnectionAccepted::from_stream(stream)?)
            }
            _ => {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("unknown command {}", command_value),
                ));
            }
        };
        Ok(x)
    }
}

#[derive(Debug, PartialEq)]
pub struct ConnectionAccepted {
    pub flags: u8,
    pub response_to_nonce: Nonce,
    pub host_assigned_connection_id: ConnectionId,
    pub host_assigned_connection_secret: SessionConnectionSecret,
}

//const SECRET_MARKER: u8 = 0x65;

impl ConnectionAccepted {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        stream.write_u8(self.flags)?;
        self.response_to_nonce.to_stream(stream)?;
        self.host_assigned_connection_id.to_stream(stream)?;
        self.host_assigned_connection_secret.to_stream(stream)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        Ok(Self {
            flags: stream.read_u8()?,
            response_to_nonce: Nonce::from_stream(stream)?,
            host_assigned_connection_id: ConnectionId::from_stream(stream)?,
            host_assigned_connection_secret: SessionConnectionSecret::from_stream(stream)?,
        })
    }
}
