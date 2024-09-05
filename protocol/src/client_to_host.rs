/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use crate::{Nonce, ParticipantId, SessionConnectionSecret};
use flood_rs::{ReadOctetStream, WriteOctetStream};
use io::ErrorKind;
use std::{fmt, io};

#[repr(u8)]
enum ClientToHostCommand {
    JoinGame = 0x01,
    Steps = 0x02,
    DownloadGameState = 0x03,
}

impl TryFrom<u8> for ClientToHostCommand {
    type Error = io::Error;

    fn try_from(value: u8) -> io::Result<Self> {
        match value {
            0x01 => Ok(ClientToHostCommand::JoinGame),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Unknown command {}", value),
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DownloadGameStateRequest {
    pub request_id: u8,
}

impl DownloadGameStateRequest {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.request_id)
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        Ok(Self {
            request_id: stream.read_u8()?,
        })
    }
}

#[derive(Debug, Clone)]
pub enum ClientToHostCommands {
    JoinGameType(JoinGameRequest),
    Steps(StepsRequest),
    DownloadGameState(DownloadGameStateRequest),
}

impl ClientToHostCommands {
    pub fn to_octet(&self) -> u8 {
        match self {
            ClientToHostCommands::Steps(_) => ClientToHostCommand::Steps as u8,
            ClientToHostCommands::JoinGameType(_) => ClientToHostCommand::JoinGame as u8,
            ClientToHostCommands::DownloadGameState(_) => {
                ClientToHostCommand::DownloadGameState as u8
            }
        }
    }

    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.to_octet())?;
        match self {
            ClientToHostCommands::Steps(predicted_steps_and_ack) => {
                predicted_steps_and_ack.to_stream(stream)
            }
            ClientToHostCommands::JoinGameType(join_game_request) => {
                join_game_request.to_stream(stream)
            }
            ClientToHostCommands::DownloadGameState(download_game_state) => {
                download_game_state.to_stream(stream)
            }
        }
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        let command_value = stream.read_u8()?;
        let command = ClientToHostCommand::try_from(command_value)?;
        let x = match command {
            ClientToHostCommand::JoinGame => ClientToHostCommands::JoinGameType(JoinGameRequest::from_stream(stream)?),
            ClientToHostCommand::Steps => ClientToHostCommands::Steps(StepsRequest::from_stream(stream)?),
            ClientToHostCommand::DownloadGameState => ClientToHostCommands::DownloadGameState(DownloadGameStateRequest::from_stream(stream)?),
        };
        Ok(x)
    }
}

impl fmt::Display for ClientToHostCommands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientToHostCommands::JoinGameType(join) => write!(f, "join {:?}", join),
            ClientToHostCommands::Steps(predicted_steps_and_ack) => {
                write!(f, "steps {:?}", predicted_steps_and_ack)
            }
            ClientToHostCommands::DownloadGameState(download_game_state) => {
                write!(f, "download game state {:?}", download_game_state)
            }
        }
    }
}

// --- Individual commands ---


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
    pub fn to_stream(self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.to_octet())?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        let join_game_type_value_raw = stream.read_u8()?;
        JoinGameTypeValue::try_from(join_game_type_value_raw)
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

    fn try_from(value: u8) -> io::Result<Self> {
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

    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
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

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
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
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.local_index)
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
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
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.players.len() as u8)?;
        for player in self.players.iter() {
            player.to_stream(stream)?;
        }
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
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
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        self.nonce.to_stream(stream)?;
        self.join_game_type.to_stream(stream)?;
        // TODO: Add more for other join game types.
        self.player_requests.to_stream(stream)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
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
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
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
    pub participant_party_index: u8,
    pub first_step_id: u32,
    pub serialized_predicted_steps: Vec<Vec<u8>>,
}

impl PredictedStepsForPlayer {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.participant_party_index)?;
        stream.write_u32(self.first_step_id)?;
        stream.write_u8(self.serialized_predicted_steps.len() as u8)?;

        for serialized_predicted_step in self.serialized_predicted_steps.iter() {
            stream.write_u8(serialized_predicted_step.len() as u8)?;
            stream.write(serialized_predicted_step)?;
        }

        Ok(())
    }
    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        let participant_party_index = stream.read_u8()?;
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
            participant_party_index,
            first_step_id,
            serialized_predicted_steps: steps_vec,
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

        let mut predicted_steps_for_players =
            Vec::<PredictedStepsForPlayer>::with_capacity(player_count as usize);

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

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        Ok(Self {
            ack: StepsAck::from_stream(stream)?,
            combined_predicted_steps: PredictedStepsForPlayers::from_stream(stream)?,
        })
    }
}
