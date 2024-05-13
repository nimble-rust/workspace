/*----------------------------------------------------------------------------------------------------------
 *  Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------------------*/
use std::io;
use std::io::ErrorKind;

use flood_rs::{ReadOctetStream, WriteOctetStream};

use crate::{Nonce, ParticipantId, SessionConnectionSecret, Version};

#[repr(u8)]
enum ClientToHostCommand {
    Connect = 0x05,
    JoinGame = 0x01,
    Steps = 0x02,
}

impl TryFrom<u8> for ClientToHostCommand {
    type Error = io::Error;

    fn try_from(value: u8) -> std::io::Result<Self> {
        match value {
            0x05 => Ok(ClientToHostCommand::Connect),
            0x01 => Ok(ClientToHostCommand::JoinGame),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Unknown command {}", value),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ClientToHostCommands {
    ConnectType(ConnectRequest),
    JoinGameType(JoinGameRequest),
    Steps,
}

impl ClientToHostCommands {
    pub fn to_octet(&self) -> u8 {
        match self {
            ClientToHostCommands::ConnectType(_) => ClientToHostCommand::Connect as u8,
            ClientToHostCommands::Steps => ClientToHostCommand::Steps as u8,
            ClientToHostCommands::JoinGameType(_) => ClientToHostCommand::JoinGame as u8
        }
    }

    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        stream.write_u8(self.to_octet())?;
        match self {
            ClientToHostCommands::ConnectType(connect_command) => connect_command.to_stream(stream),
            ClientToHostCommands::Steps => Ok(()),
            ClientToHostCommands::JoinGameType(join_game_request) => join_game_request.to_stream(stream),
        }
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        let command_value = stream.read_u8()?;
        let command = ClientToHostCommand::try_from(command_value)?;
        let x = match command {
            ClientToHostCommand::Connect => {
                ClientToHostCommands::ConnectType(ConnectRequest::from_stream(stream)?)
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

// --- Individual commands ---

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ConnectRequest {
    pub nimble_version: Version,
    pub use_debug_stream: bool,
    pub application_version: Version,
    pub nonce: Nonce,
}

impl ConnectRequest {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        self.nimble_version.to_stream(stream)?;
        stream.write_u8(if self.use_debug_stream { 0x01 } else { 0x00 })?;
        self.application_version.to_stream(stream)?;
        self.nonce.to_stream(stream)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        Ok(Self {
            nimble_version: Version::from_stream(stream)?,
            use_debug_stream: stream.read_u8()? != 0,
            application_version: Version::from_stream(stream)?,
            nonce: Nonce::from_stream(stream)?,
        })
    }
}

#[repr(u8)]
pub enum JoinGameTypeValue {
    NoSecret,
    SessionSecret,
    HostMigrationParticipantId,
}

impl JoinGameTypeValue {
    pub fn to_octet(&self) -> u8 {
        match self {
            JoinGameTypeValue::NoSecret => JoinGameTypeValue::NoSecret as u8,
            JoinGameTypeValue::SessionSecret => JoinGameTypeValue::SessionSecret as u8,
            JoinGameTypeValue::HostMigrationParticipantId => {
                JoinGameTypeValue::HostMigrationParticipantId as u8
            }
        }
    }
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        stream.write_u8(self.to_octet())?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        let join_game_type_value_raw = stream.read_u8()?;
        Ok(JoinGameTypeValue::try_from(join_game_type_value_raw)?)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum JoinGameType {
    NoSecret,
    UseSessionSecret(SessionConnectionSecret),
    HostMigrationParticipantId(ParticipantId),
}

impl TryFrom<u8> for JoinGameTypeValue {
    type Error = io::Error;

    fn try_from(value: u8) -> std::io::Result<Self> {
        match value {
            0x00 => Ok(JoinGameTypeValue::NoSecret),
            0x01 => Ok(JoinGameTypeValue::SessionSecret),
            0x02 => Ok(JoinGameTypeValue::HostMigrationParticipantId),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Unknown join game type {}", value),
            )),
        }
    }
}

impl JoinGameType {
    pub fn to_octet(&self) -> u8 {
        match self {
            JoinGameType::NoSecret => JoinGameTypeValue::NoSecret as u8,
            JoinGameType::UseSessionSecret(_) => JoinGameTypeValue::SessionSecret as u8,
            JoinGameType::HostMigrationParticipantId(_) => {
                JoinGameTypeValue::HostMigrationParticipantId as u8
            }
        }
    }

    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        stream.write_u8(self.to_octet())?;
        match self {
            JoinGameType::NoSecret => {}
            JoinGameType::UseSessionSecret(session_secret) => session_secret.to_stream(stream)?,
            JoinGameType::HostMigrationParticipantId(participant_id) => {
                participant_id.to_stream(stream)?
            }
        }
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        let join_game_type_value_raw = stream.read_u8()?;
        let value = JoinGameTypeValue::try_from(join_game_type_value_raw)?;
        let join_game_type = match value {
            JoinGameTypeValue::NoSecret => JoinGameType::NoSecret,
            JoinGameTypeValue::SessionSecret => {
                JoinGameType::UseSessionSecret(SessionConnectionSecret::from_stream(stream)?)
            }
            JoinGameTypeValue::HostMigrationParticipantId => {
                JoinGameType::HostMigrationParticipantId(ParticipantId::from_stream(stream)?)
            }
        };
        Ok(join_game_type)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct JoinPlayerRequest {
    pub local_index: u8,
}

impl JoinPlayerRequest {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        stream.write_u8(self.local_index)
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        Ok(Self {
            local_index: stream.read_u8()?,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct JoinPlayerRequests {
    pub players: Vec<JoinPlayerRequest>,
}

impl JoinPlayerRequests {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        stream.write_u8(self.players.len() as u8)?;
        for player in self.players.iter() {
            player.to_stream(stream)?;
        }
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        let count = stream.read_u8()?;
        let mut vec = Vec::<JoinPlayerRequest>::with_capacity(count as usize);
        for v in vec.iter_mut() {
            *v = JoinPlayerRequest::from_stream(stream)?;
        }

        Ok(Self { players: vec })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct JoinGameRequest {
    pub nonce: Nonce,
    pub join_game_type: JoinGameType,
    pub player_requests: JoinPlayerRequests,
}

impl JoinGameRequest {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        self.nonce.to_stream(stream)?;
        self.join_game_type.to_stream(stream)?;
        self.player_requests.to_stream(stream)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        Ok(Self {
            nonce: Nonce::from_stream(stream)?,
            join_game_type: JoinGameType::from_stream(stream)?,
            player_requests: JoinPlayerRequests::from_stream(stream)?,
        })
    }
}
