/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use crate::{Nonce, SessionConnectionSecret};
use connection_layer::ConnectionId;
use flood_rs::{ReadOctetStream, WriteOctetStream};
use std::io;
use std::io::ErrorKind;


#[repr(u8)]
pub enum HostToClientOobCommand {
    Connect = 0x0D,
}
impl TryFrom<u8> for HostToClientOobCommand {
    type Error = io::Error;

    fn try_from(value: u8) -> io::Result<Self> {
        match value {
            0x0D => Ok(HostToClientOobCommand::Connect),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Unknown host to client command {}", value),
            )),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ConnectionAccepted {
    pub flags: u8,
    pub response_to_nonce: Nonce,
    pub host_assigned_connection_id: ConnectionId,
    pub host_assigned_connection_secret: SessionConnectionSecret,
}

#[derive(Debug)]
pub enum HostToClientOobCommands {
    ConnectType(ConnectionAccepted),
}

impl ConnectionAccepted {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.flags)?;
        self.response_to_nonce.to_stream(stream)?;
        self.host_assigned_connection_id.to_stream(stream)?;
        self.host_assigned_connection_secret.to_stream(stream)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        Ok(Self {
            flags: stream.read_u8()?,
            response_to_nonce: Nonce::from_stream(stream)?,
            host_assigned_connection_id: ConnectionId::from_stream(stream)?,
            host_assigned_connection_secret: SessionConnectionSecret::from_stream(stream)?,
        })
    }
}


impl HostToClientOobCommands {
    pub fn to_octet(&self) -> u8 {
        match self {
            HostToClientOobCommands::ConnectType(_) => HostToClientOobCommand::Connect as u8,
        }
    }

    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.to_octet())?;
        match self {
            HostToClientOobCommands::ConnectType(connect_command) => connect_command.to_stream(stream),
        }
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        let command_value = stream.read_u8()?;
        let command = HostToClientOobCommand::try_from(command_value)?;
        let x = match command {
            HostToClientOobCommand::Connect => {
                HostToClientOobCommands::ConnectType(ConnectionAccepted::from_stream(stream)?)
            }
        };
        Ok(x)
    }
}