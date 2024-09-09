/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use blob_stream::prelude::SenderToReceiverFrontCommands;
use flood_rs::{ReadOctetStream, WriteOctetStream};
use io::ErrorKind;
use std::io;

use crate::{Nonce, ParticipantId, SessionConnectionSecret};
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
pub struct TickId(pub u32);

impl TickId {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u32(self.0)
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        Ok(Self(stream.read_u32()?))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DownloadGameStateResponse {
    pub client_request: u8,
    pub tick_id: TickId,
    pub blob_stream_channel: u16,
}

impl DownloadGameStateResponse {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.client_request)?;
        self.tick_id.to_stream(stream)?;
        stream.write_u16(self.blob_stream_channel)
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        Ok(Self {
            client_request: stream.read_u8()?,
            tick_id: TickId::from_stream(stream)?,
            blob_stream_channel: stream.read_u16()?,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct GameStatePart {
    pub blob_stream_command: SenderToReceiverFrontCommands,
}

#[derive(Debug)]
pub enum HostToClientCommands {
    JoinGame(JoinGameAccepted),
    GameStep(GameStepResponse),
    DownloadGameState(DownloadGameStateResponse),
    BlobStreamChannel(SenderToReceiverFrontCommands),
}

impl HostToClientCommands {
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

    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
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

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
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
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        self.session_secret.to_stream(stream)?;
        stream.write_u8(self.party_id)
    }
    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
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
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.local_index)?;
        self.participant_id.to_stream(stream)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        Ok(Self {
            local_index: stream.read_u8()?,
            participant_id: ParticipantId::from_stream(stream)?,
        })
    }
}

#[derive(Debug)]
pub struct JoinGameParticipants(pub Vec<JoinGameParticipant>);

impl JoinGameParticipants {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.0.len() as u8)?;
        for join_game_participant in &self.0 {
            join_game_participant.to_stream(stream)?
        }
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
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
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        self.nonce.to_stream(stream)?;
        self.party_and_session_secret.to_stream(stream)?;
        self.participants.to_stream(stream)
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
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
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.connection_buffer_count)?;
        stream.write_i8(self.delta_buffer)?;
        stream.write_u32(self.last_step_received_from_client)
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        Ok(Self {
            connection_buffer_count: stream.read_u8()?,
            delta_buffer: stream.read_i8()?,
            last_step_received_from_client: stream.read_u32()?,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct AuthoritativeStepRange {
    pub delta_steps_from_previous: u8,
    pub authoritative_steps: Vec<Vec<u8>>,
}

impl AuthoritativeStepRange {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.delta_steps_from_previous)?;
        stream.write_u8(self.authoritative_steps.len() as u8)?;
        for authoritative_step_payload in &self.authoritative_steps {
            stream.write_u8(authoritative_step_payload.len() as u8)?;
            stream.write(authoritative_step_payload.as_slice())?;
        }
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        let delta_steps = stream.read_u8()?;
        let count = stream.read_u8()?;

        let mut authoritative_steps_vec = Vec::<Vec<u8>>::with_capacity(count as usize);
        for _ in 0..count {
            let octet_count = stream.read_u8()?;
            let mut step_payload = Vec::<u8>::with_capacity(octet_count as usize);
            stream.read(step_payload.as_mut_slice())?;
            authoritative_steps_vec.push(step_payload);
        }
        Ok(Self {
            delta_steps_from_previous: delta_steps,
            authoritative_steps: authoritative_steps_vec,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct AuthoritativeStepRanges {
    pub start_step_id: u32,
    pub ranges: Vec<AuthoritativeStepRange>,
}

impl AuthoritativeStepRanges {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u32(self.start_step_id)?;
        stream.write_u8(self.ranges.len() as u8)?;
        for range in &self.ranges {
            range.to_stream(stream)?;
        }
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        let start_step_id = stream.read_u32()?;
        let range_count = stream.read_u8()?;

        let mut authoritative_step_ranges =
            Vec::<AuthoritativeStepRange>::with_capacity(range_count as usize);

        for _ in 0..range_count {
            authoritative_step_ranges.push(AuthoritativeStepRange::from_stream(stream)?);
        }

        Ok(Self {
            start_step_id,
            ranges: authoritative_step_ranges,
        })
    }
}

#[derive(Debug)]
pub struct GameStepResponse {
    pub response_header: GameStepResponseHeader,
    pub authoritative_ranges: AuthoritativeStepRanges,
    pub payload: Vec<u8>,
}

fn read_octets(stream: &mut dyn ReadOctetStream) -> io::Result<Vec<u8>> {
    let len = stream.read_u16()?;
    let mut data: Vec<u8> = vec![0u8; len as usize];
    stream.read(data.as_mut_slice())?;
    Ok(data)
}

fn write_octets(stream: &mut dyn WriteOctetStream, payload: &[u8]) -> io::Result<()> {
    stream.write_u16(payload.len() as u16)?;
    stream.write(payload)
}

impl GameStepResponse {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        self.response_header.to_stream(stream)?;
        self.authoritative_ranges.to_stream(stream)?;
        write_octets(stream, self.payload.as_slice())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        Ok(Self {
            response_header: GameStepResponseHeader::from_stream(stream)?,
            authoritative_ranges: AuthoritativeStepRanges::from_stream(stream)?,
            payload: read_octets(stream)?,
        })
    }
}
