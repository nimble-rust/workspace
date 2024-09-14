/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use crate::host_to_client::TickIdUtil;
use crate::{Nonce, SessionConnectionSecret};
use blob_stream::prelude::ReceiverToSenderFrontCommands;
use flood_rs::{Deserialize, ReadOctetStream, Serialize, WriteOctetStream};
use io::ErrorKind;
use nimble_participant::ParticipantId;
use std::collections::HashMap;
use std::fmt::Debug;
use std::{fmt, io};
use tick_id::TickId;

#[repr(u8)]
enum ClientToHostCommand {
    JoinGame = 0x01,
    Steps = 0x02,
    DownloadGameState = 0x03,
    BlobStreamChannel = 0x04,
}

impl TryFrom<u8> for ClientToHostCommand {
    type Error = io::Error;

    fn try_from(value: u8) -> io::Result<Self> {
        match value {
            0x01 => Ok(ClientToHostCommand::JoinGame),
            0x04 => Ok(ClientToHostCommand::BlobStreamChannel),
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
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.request_id)
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        Ok(Self {
            request_id: stream.read_u8()?,
        })
    }
}

#[derive(Debug, Clone)]
pub enum ClientToHostCommands<
    StepT: std::clone::Clone + Debug + Eq + PartialEq + Serialize + Deserialize,
> {
    JoinGameType(JoinGameRequest),
    Steps(StepsRequest<StepT>),
    DownloadGameState(DownloadGameStateRequest),
    BlobStreamChannel(ReceiverToSenderFrontCommands),
}

impl<StepT: std::clone::Clone + Debug + Eq + PartialEq + Serialize + Deserialize>
    ClientToHostCommands<StepT>
{
    pub fn to_octet(&self) -> u8 {
        match self {
            ClientToHostCommands::Steps(_) => ClientToHostCommand::Steps as u8,
            ClientToHostCommands::JoinGameType(_) => ClientToHostCommand::JoinGame as u8,
            ClientToHostCommands::DownloadGameState(_) => {
                ClientToHostCommand::DownloadGameState as u8
            }
            ClientToHostCommands::BlobStreamChannel(_) => {
                ClientToHostCommand::BlobStreamChannel as u8
            }
        }
    }

    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
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
            ClientToHostCommands::BlobStreamChannel(blob_stream_command) => {
                blob_stream_command.to_stream(stream)
            }
        }
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        let command_value = stream.read_u8()?;
        let command = ClientToHostCommand::try_from(command_value)?;
        let x = match command {
            ClientToHostCommand::JoinGame => {
                ClientToHostCommands::JoinGameType(JoinGameRequest::from_stream(stream)?)
            }
            ClientToHostCommand::Steps => {
                ClientToHostCommands::Steps(StepsRequest::from_stream(stream)?)
            }
            ClientToHostCommand::DownloadGameState => ClientToHostCommands::DownloadGameState(
                DownloadGameStateRequest::from_stream(stream)?,
            ),
            ClientToHostCommand::BlobStreamChannel => ClientToHostCommands::BlobStreamChannel(
                ReceiverToSenderFrontCommands::from_stream(stream)?,
            ),
        };
        Ok(x)
    }
}

impl<StepT: Clone + Debug + Eq + PartialEq + Serialize + Deserialize> fmt::Display
    for ClientToHostCommands<StepT>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientToHostCommands::JoinGameType(join) => write!(f, "join {:?}", join),
            ClientToHostCommands::Steps(predicted_steps_and_ack) => {
                write!(f, "steps {:?}", predicted_steps_and_ack)
            }
            ClientToHostCommands::DownloadGameState(download_game_state) => {
                write!(f, "download game state {:?}", download_game_state)
            }
            ClientToHostCommands::BlobStreamChannel(blob_command) => {
                write!(f, "blob stream channel {:?}", blob_command)
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
    pub fn to_stream(self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.to_octet())?;
        Ok(())
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
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

    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
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

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
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
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.local_index)
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
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
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.players.len() as u8)?;
        for player in self.players.iter() {
            player.to_stream(stream)?;
        }
        Ok(())
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
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
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        self.nonce.to_stream(stream)?;
        self.join_game_type.to_stream(stream)?;
        // TODO: Add more for other join game types.
        self.player_requests.to_stream(stream)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
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
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        stream.write_u32(self.latest_received_step_tick_id)?;
        stream.write_u64(self.lost_steps_mask_after_last_received)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        Ok(Self {
            latest_received_step_tick_id: stream.read_u32()?,
            lost_steps_mask_after_last_received: stream.read_u64()?,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct AuthoritativeCombinedStepForAllParticipants<
    StepT: Serialize + Deserialize + Debug + Clone + Eq + PartialEq,
> {
    pub authoritative_participants: HashMap<ParticipantId, StepT>,
}

impl<StepT: Serialize + Deserialize + Debug + Clone + Eq + PartialEq> Serialize
    for AuthoritativeCombinedStepForAllParticipants<StepT>
{
    fn serialize(&self, stream: &mut impl WriteOctetStream) -> io::Result<()>
    where
        Self: Sized,
    {
        stream.write_u8(self.authoritative_participants.len() as u8)?;

        for (participant_id, authoritative_step_for_one_participant) in
            self.authoritative_participants.iter()
        {
            participant_id.to_stream(stream)?;
            authoritative_step_for_one_participant.serialize(stream)?;
        }
        Ok(())
    }
}

impl<StepT: Serialize + Deserialize + Debug + Clone + Eq + PartialEq> Deserialize
    for AuthoritativeCombinedStepForAllParticipants<StepT>
{
    fn deserialize(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        let participant_count_in_combined_step = stream.read_u8()?;

        let mut authoritative_steps_map =
            HashMap::with_capacity(participant_count_in_combined_step as usize);

        for _ in 0..participant_count_in_combined_step {
            let participant_id = ParticipantId::from_stream(stream)?;
            let authoritative_step = StepT::deserialize(stream)?;
            authoritative_steps_map.insert(participant_id, authoritative_step);
        }

        Ok(Self {
            authoritative_participants: authoritative_steps_map,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PredictedStep<StepT: Serialize + Deserialize + Debug + Clone + Eq + PartialEq> {
    pub predicted_players: HashMap<LocalIndex, StepT>,
}

type LocalIndex = u8;

#[derive(Debug, PartialEq, Clone)]
pub struct PredictedStepsForAllPlayers<
    StepT: Serialize + Deserialize + Debug + Clone + Eq + PartialEq,
> {
    pub predicted_players: HashMap<LocalIndex, PredictedStepsForOnePlayer<StepT>>,
}

impl<StepT: Serialize + Deserialize + Debug + Clone + Eq + PartialEq>
    PredictedStepsForAllPlayers<StepT>
{
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.predicted_players.len() as u8)?;

        for (local_index, predicted_steps_for_one_player) in self.predicted_players.iter() {
            stream.write_u8(*local_index)?;
            predicted_steps_for_one_player.to_stream(stream)?;
        }

        Ok(())
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        let player_count = stream.read_u8()?;

        let mut players_vector = HashMap::with_capacity(player_count as usize);

        for _ in 0..player_count {
            let predicted_steps_for_one_player = PredictedStepsForOnePlayer::from_stream(stream)?;
            let index = stream.read_u8()?;
            players_vector.insert(index, predicted_steps_for_one_player);
        }

        Ok(Self {
            predicted_players: players_vector,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PredictedStepsForOnePlayer<StepT: Clone + Eq + PartialEq + Serialize + Deserialize> {
    pub first_tick_id: TickId,
    pub predicted_steps: Vec<StepT>,
}

impl<StepT: Clone + Eq + PartialEq + Serialize + Deserialize> PredictedStepsForOnePlayer<StepT> {
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        TickIdUtil::to_stream(self.first_tick_id, stream)?;
        stream.write_u8(self.predicted_steps.len() as u8)?;

        for predicted_step_for_player in self.predicted_steps.iter() {
            predicted_step_for_player.serialize(stream)?;
        }

        Ok(())
    }
    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        let first_tick_id = TickIdUtil::from_stream(stream)?;
        let step_count = stream.read_u8()?;

        let mut predicted_steps_for_players = Vec::<StepT>::with_capacity(step_count as usize);

        for _ in 0..step_count {
            let predicted_steps_for_player = StepT::deserialize(stream)?;
            predicted_steps_for_players.push(predicted_steps_for_player);
        }

        Ok(Self {
            first_tick_id,
            predicted_steps: predicted_steps_for_players,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct StepsRequest<StepT: Clone + Eq + PartialEq + Serialize + Deserialize + Debug> {
    pub ack: StepsAck,
    pub combined_predicted_steps: PredictedStepsForAllPlayers<StepT>,
}

impl<StepT: Clone + Eq + PartialEq + Serialize + Deserialize + Debug> StepsRequest<StepT> {
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        self.ack.to_stream(stream)?;
        self.combined_predicted_steps.to_stream(stream)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        Ok(Self {
            ack: StepsAck::from_stream(stream)?,
            combined_predicted_steps: PredictedStepsForAllPlayers::from_stream(stream)?,
        })
    }
}
