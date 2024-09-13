/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use crate::client_to_host::AuthoritativeCombinedStepForAllParticipants;
use crate::{Nonce, SessionConnectionSecret};
use blob_stream::prelude::SenderToReceiverFrontCommands;
use flood_rs::{Deserialize, ReadOctetStream, Serialize, WriteOctetStream};
use io::ErrorKind;
use nimble_participant::ParticipantId;
use std::fmt::Debug;
use std::io;
use tick_id::TickId;
// #define NimbleSerializeCmdGameStepResponse (0x08)
// #define NimbleSerializeCmdJoinGameResponse (0x09)
// #define NimbleSerializeCmdGameStatePart (0x0a)
// #define NimbleSerializeCmdGameStateResponse (0x0b)
// #define NimbleSerializeCmdJoinGameOutOfParticipantSlotsResponse (0x0c)
// #define NimbleSerializeCmdConnectResponse (0x0d)

#[repr(u8)]
pub enum HostToClientCommand {
    GameStep = 0x08,
    JoinGame = 0x09,
    DownloadGameState = 0x0B,
    BlobStreamChannel = 0x0c,
}

impl TryFrom<u8> for HostToClientCommand {
    type Error = io::Error;

    fn try_from(value: u8) -> io::Result<Self> {
        match value {
            0x09 => Ok(HostToClientCommand::JoinGame),
            0x08 => Ok(HostToClientCommand::GameStep),
            0x0B => Ok(HostToClientCommand::DownloadGameState),
            0x0C => Ok(HostToClientCommand::BlobStreamChannel),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Unknown host to client command {}", value),
            )),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct TickIdUtil;

impl TickIdUtil {
    pub fn to_stream(tick_id: TickId, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        stream.write_u32(tick_id.0)
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<TickId> {
        Ok(TickId(stream.read_u32()?))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DownloadGameStateResponse {
    pub client_request: u8,
    pub tick_id: TickId,
    pub blob_stream_channel: u16,
}

impl DownloadGameStateResponse {
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.client_request)?;
        TickIdUtil::to_stream(self.tick_id, stream)?;
        stream.write_u16(self.blob_stream_channel)
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        Ok(Self {
            client_request: stream.read_u8()?,
            tick_id: TickIdUtil::from_stream(stream)?,
            blob_stream_channel: stream.read_u16()?,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct GameStatePart {
    pub blob_stream_command: SenderToReceiverFrontCommands,
}

#[derive(Debug)]
pub enum HostToClientCommands<StepT: Deserialize + Serialize + Debug + Clone + Eq> {
    JoinGame(JoinGameAccepted),
    GameStep(GameStepResponse<StepT>),
    DownloadGameState(DownloadGameStateResponse),
    BlobStreamChannel(SenderToReceiverFrontCommands),
}

impl<StepT: Deserialize + Serialize + Debug + Clone + Eq> HostToClientCommands<StepT> {
    pub fn to_octet(&self) -> u8 {
        match self {
            HostToClientCommands::JoinGame(_) => HostToClientCommand::JoinGame as u8,
            HostToClientCommands::GameStep(_) => HostToClientCommand::GameStep as u8,
            HostToClientCommands::DownloadGameState(_) => {
                HostToClientCommand::DownloadGameState as u8
            }
            HostToClientCommands::BlobStreamChannel(_) => {
                HostToClientCommand::BlobStreamChannel as u8
            }
        }
    }

    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.to_octet())?;
        match self {
            HostToClientCommands::JoinGame(join_game_response) => {
                join_game_response.to_stream(stream)
            }
            HostToClientCommands::GameStep(game_step_response) => {
                game_step_response.to_stream(stream)
            }
            HostToClientCommands::DownloadGameState(download_game_state_response) => {
                download_game_state_response.to_stream(stream)
            }
            HostToClientCommands::BlobStreamChannel(blob_stream_command) => {
                blob_stream_command.to_stream(stream)
            }
        }
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        let command_value = stream.read_u8()?;
        let command = HostToClientCommand::try_from(command_value)?;
        let x = match command {
            HostToClientCommand::JoinGame => {
                HostToClientCommands::JoinGame(JoinGameAccepted::from_stream(stream)?)
            }
            HostToClientCommand::GameStep => {
                HostToClientCommands::GameStep(GameStepResponse::from_stream(stream)?)
            } /*
            => {
            return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("unknown command {}", command_value),
            ));
            }

            */
            HostToClientCommand::DownloadGameState => HostToClientCommands::DownloadGameState(
                DownloadGameStateResponse::from_stream(stream)?,
            ),
            HostToClientCommand::BlobStreamChannel => HostToClientCommands::BlobStreamChannel(
                SenderToReceiverFrontCommands::from_stream(stream)?,
            ),
        };
        Ok(x)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PartyAndSessionSecret {
    pub session_secret: SessionConnectionSecret,
    pub party_id: u8,
}

impl PartyAndSessionSecret {
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        self.session_secret.to_stream(stream)?;
        stream.write_u8(self.party_id)
    }
    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        Ok(Self {
            session_secret: SessionConnectionSecret::from_stream(stream)?,
            party_id: stream.read_u8()?,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct JoinGameParticipant {
    pub local_index: u8,
    pub participant_id: ParticipantId,
}

impl JoinGameParticipant {
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.local_index)?;
        self.participant_id.to_stream(stream)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        Ok(Self {
            local_index: stream.read_u8()?,
            participant_id: ParticipantId::from_stream(stream)?,
        })
    }
}

#[derive(Debug)]
pub struct JoinGameParticipants(pub Vec<JoinGameParticipant>);

impl JoinGameParticipants {
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.0.len() as u8)?;
        for join_game_participant in &self.0 {
            join_game_participant.to_stream(stream)?
        }
        Ok(())
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        let count = stream.read_u8()?;
        let mut vec = Vec::<JoinGameParticipant>::with_capacity(count as usize);
        for _ in 0..count {
            vec.push(JoinGameParticipant::from_stream(stream)?);
        }

        Ok(Self(vec))
    }
}

#[derive(Debug)]
pub struct JoinGameAccepted {
    pub nonce: Nonce,
    pub party_and_session_secret: PartyAndSessionSecret,
    pub participants: JoinGameParticipants,
}

impl JoinGameAccepted {
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        self.nonce.to_stream(stream)?;
        self.party_and_session_secret.to_stream(stream)?;
        self.participants.to_stream(stream)
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        Ok(Self {
            nonce: Nonce::from_stream(stream)?,
            party_and_session_secret: PartyAndSessionSecret::from_stream(stream)?,
            participants: JoinGameParticipants::from_stream(stream)?,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct GameStepResponseHeader {
    pub connection_buffer_count: u8,
    pub delta_buffer: i8,
    pub last_step_received_from_client: u32,
}

impl GameStepResponseHeader {
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.connection_buffer_count)?;
        stream.write_i8(self.delta_buffer)?;
        stream.write_u32(self.last_step_received_from_client)
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        Ok(Self {
            connection_buffer_count: stream.read_u8()?,
            delta_buffer: stream.read_i8()?,
            last_step_received_from_client: stream.read_u32()?,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct AuthoritativeStepRange<StepT: Deserialize + Serialize + Debug + Eq + Clone> {
    pub delta_steps_from_previous: u8,
    pub authoritative_steps: Vec<AuthoritativeCombinedStepForAllParticipants<StepT>>,
}

impl<StepT: Deserialize + Serialize + Debug + Eq + Clone> AuthoritativeStepRange<StepT> {
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.delta_steps_from_previous)?;
        stream.write_u8(self.authoritative_steps.len() as u8)?;

        for authoritative_step_payload in &self.authoritative_steps {
            authoritative_step_payload.serialize(stream)?;
        }
        Ok(())
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        let delta_steps = stream.read_u8()?;
        let count = stream.read_u8()?;

        let mut authoritative_steps_vec = Vec::<AuthoritativeCombinedStepForAllParticipants<StepT>>::with_capacity(count as usize);
        for _ in 0..count {
            let authoritative_combined_step = AuthoritativeCombinedStepForAllParticipants::deserialize(stream)?;
            authoritative_steps_vec.push(authoritative_combined_step);
        }
        Ok(Self {
            delta_steps_from_previous: delta_steps,
            authoritative_steps: authoritative_steps_vec,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct AuthoritativeStepRanges<StepT: Deserialize + Serialize + Debug + Eq + Clone> {
    pub start_tick_id: TickId,
    pub ranges: Vec<AuthoritativeStepRange<StepT>>,
}

impl<StepT: Deserialize + Serialize + Debug + Eq + Clone> AuthoritativeStepRanges<StepT> {
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        TickIdUtil::to_stream(self.start_tick_id, stream)?;
        stream.write_u8(self.ranges.len() as u8)?;
        for range in &self.ranges {
            range.to_stream(stream)?;
        }
        Ok(())
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        let start_step_id = TickIdUtil::from_stream(stream)?;
        let range_count = stream.read_u8()?;

        let mut authoritative_step_ranges =
            Vec::<AuthoritativeStepRange<StepT>>::with_capacity(range_count as usize);

        for _ in 0..range_count {
            authoritative_step_ranges.push(AuthoritativeStepRange::from_stream(stream)?);
        }

        Ok(Self {
            start_tick_id: start_step_id,
            ranges: authoritative_step_ranges,
        })
    }
}

#[derive(Debug)]
pub struct GameStepResponse<StepT: Serialize + Deserialize + Debug + Clone + Eq> {
    pub response_header: GameStepResponseHeader,
    pub authoritative_steps: AuthoritativeStepRanges<StepT>,
}

fn read_octets(stream: &mut impl ReadOctetStream) -> io::Result<Vec<u8>> {
    let len = stream.read_u16()?;
    let mut data: Vec<u8> = vec![0u8; len as usize];
    stream.read(data.as_mut_slice())?;
    Ok(data)
}

fn write_octets(stream: &mut impl WriteOctetStream, payload: &[u8]) -> io::Result<()> {
    stream.write_u16(payload.len() as u16)?;
    stream.write(payload)
}

impl<StepT: Deserialize + Serialize + Debug + Clone + Eq> GameStepResponse<StepT> {
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        self.response_header.to_stream(stream)?;
        self.authoritative_steps.to_stream(stream)
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        Ok(Self {
            response_header: GameStepResponseHeader::from_stream(stream)?,
            authoritative_steps: AuthoritativeStepRanges::from_stream(stream)?,
        })
    }
}
