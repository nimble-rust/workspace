/*----------------------------------------------------------------------------------------------------------
 *  Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------------------*/
use std::{fmt, io};
use std::fmt::Pointer;
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
    Steps(StepsRequest),
}

impl ClientToHostCommands {
    pub fn to_octet(&self) -> u8 {
        match self {
            ClientToHostCommands::ConnectType(_) => ClientToHostCommand::Connect as u8,
            ClientToHostCommands::Steps(_) => ClientToHostCommand::Steps as u8,
            ClientToHostCommands::JoinGameType(_) => ClientToHostCommand::JoinGame as u8
        }
    }

    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        stream.write_u8(self.to_octet())?;
        match self {
            ClientToHostCommands::ConnectType(connect_command) => connect_command.to_stream(stream),
            ClientToHostCommands::Steps(predicted_steps_and_ack) => predicted_steps_and_ack.to_stream(stream),
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

impl fmt::Display for ClientToHostCommands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientToHostCommands::ConnectType(connect) => write!(f, "connect {:?}", connect),
            ClientToHostCommands::JoinGameType(join) => write!(f, "join {:?}", join),
            ClientToHostCommands::Steps(predicted_steps_and_ack) => write!(f, "steps {:?}", predicted_steps_and_ack)
        }
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
        // TODO: Add more for other join game types.
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


#[derive(Debug, PartialEq, Clone)]
pub struct StepsAck {
    pub latest_received_step_tick_id: u32,
    pub lost_steps_mask_after_last_received: u64,
}

impl StepsAck {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        stream.write_u32(self.latest_received_step_tick_id)?;
        stream.write_u64(self.lost_steps_mask_after_last_received)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        Ok(Self {
            latest_received_step_tick_id: stream.read_u32()?,
            lost_steps_mask_after_last_received: stream.read_u64()?,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PredictedStepsForPlayer {
    pub first_step_id: u32,
    pub combined_steps_octets: Vec<Vec<u8>>,
}

impl PredictedStepsForPlayer {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u32(self.first_step_id)?;
        stream.write_u8(self.combined_steps_octets.len() as u8)?;

        for combined_predicted_step in self.combined_steps_octets.iter() {
            stream.write_u8(combined_predicted_step.len() as u8)?;
            stream.write(combined_predicted_step)?;
        }

        Ok(())
    }
    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        let first_step_id = stream.read_u32()?;
        let step_count = stream.read_u8()?;

        let mut temp = vec![0u8; 256];
        let mut steps_vec = Vec::with_capacity(step_count as usize);

        for _ in 0..step_count {
            let octet_count = stream.read_u8()? as usize;
            stream.read(&mut temp[0..octet_count])?;
            steps_vec.push(temp[0..octet_count].to_vec());
        }

        Ok(Self {
            first_step_id,
            combined_steps_octets: steps_vec,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PredictedStepsForPlayers {
    pub predicted_steps_for_players: Vec<PredictedStepsForPlayer>,
}

impl PredictedStepsForPlayers {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.predicted_steps_for_players.len() as u8)?;

        for predicted_steps_for_player in self.predicted_steps_for_players.iter() {
            predicted_steps_for_player.to_stream(stream)?;
        }

        Ok(())
    }
    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        let player_count = stream.read_u8()?;

        let mut predicted_steps_for_players = Vec::<PredictedStepsForPlayer>::with_capacity(player_count as usize);

        for _ in 0..player_count {
            let predicted_steps_for_player = PredictedStepsForPlayer::from_stream(stream)?;
            predicted_steps_for_players.push(predicted_steps_for_player);
        }

        Ok(Self {
            predicted_steps_for_players,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct StepsRequest {
    pub ack: StepsAck,
    pub combined_predicted_steps: PredictedStepsForPlayers,
}

impl StepsRequest {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        self.ack.to_stream(stream)?;
        self.combined_predicted_steps.to_stream(stream)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        Ok(Self {
            ack: StepsAck::from_stream(stream)?,
            combined_predicted_steps: PredictedStepsForPlayers::from_stream(stream)?,
        })
    }
}
