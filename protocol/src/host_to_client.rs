/*----------------------------------------------------------------------------------------------------------
 *  Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------------------*/
use std::io;
use std::io::ErrorKind;

use flood_rs::{ReadOctetStream, WriteOctetStream};

use connection_layer::ConnectionId;

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
    Connect = 0x0D,
}

impl TryFrom<u8> for HostToClientCommand {
    type Error = io::Error;

    fn try_from(value: u8) -> std::io::Result<Self> {
        match value {
            0x0D => Ok(HostToClientCommand::Connect),
            0x09 => Ok(HostToClientCommand::JoinGame),
            0x08 => Ok(HostToClientCommand::GameStep),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Unknown host to client command {}", value),
            )),
        }
    }
}

#[derive(Debug)]
pub enum HostToClientCommands {
    ConnectType(ConnectionAccepted),
    JoinGame(JoinGameAccepted),
    GameStep(GameStepResponse),
}

impl HostToClientCommands {
    pub fn to_octet(&self) -> u8 {
        match self {
            HostToClientCommands::ConnectType(_) => HostToClientCommand::Connect as u8,
            HostToClientCommands::JoinGame(_) => HostToClientCommand::JoinGame as u8,
            HostToClientCommands::GameStep(_) => HostToClientCommand::GameStep as u8,
        }
    }

    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        stream.write_u8(self.to_octet())?;
        match self {
            HostToClientCommands::ConnectType(connect_command) => connect_command.to_stream(stream),
            HostToClientCommands::JoinGame(join_game_response) => {
                join_game_response.to_stream(stream)
            }
            HostToClientCommands::GameStep(game_step_response) => {
                game_step_response.to_stream(stream)
            }
        }
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        let command_value = stream.read_u8()?;
        let command = HostToClientCommand::try_from(command_value)?;
        let x = match command {
            HostToClientCommand::Connect => {
                HostToClientCommands::ConnectType(ConnectionAccepted::from_stream(stream)?)
            }
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

#[derive(Debug, PartialEq)]
pub struct PartyAndSessionSecret {
    pub session_secret: SessionConnectionSecret,
    pub party_id: u8,
}

impl PartyAndSessionSecret {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        self.session_secret.to_stream(stream)?;
        stream.write_u8(self.party_id)
    }
    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
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
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        stream.write_u8(self.local_index)?;
        self.participant_id.to_stream(stream)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        Ok(Self {
            local_index: stream.read_u8()?,
            participant_id: ParticipantId::from_stream(stream)?,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct JoinGameParticipants(pub Vec<JoinGameParticipant>);

impl JoinGameParticipants {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        stream.write_u8(self.0.len() as u8)?;
        for join_game_participant in &self.0 {
            join_game_participant.to_stream(stream)?
        }
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        let count = stream.read_u8()?;
        let mut vec = Vec::<JoinGameParticipant>::with_capacity(count as usize);
        for v in vec.iter_mut() {
            *v = JoinGameParticipant::from_stream(stream)?;
        }

        Ok(Self(vec))
    }
}

#[derive(Debug, PartialEq)]
pub struct JoinGameAccepted {
    pub party_and_session_secret: PartyAndSessionSecret,
    pub participants: JoinGameParticipants,
}

//const SECRET_MARKER: u8 = 0x65;

impl JoinGameAccepted {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        self.party_and_session_secret.to_stream(stream)?;
        self.participants.to_stream(stream)
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        Ok(Self {
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
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        stream.write_u8(self.connection_buffer_count)?;
        stream.write_i8(self.delta_buffer)?;
        stream.write_u32(self.last_step_received_from_client)
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
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
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        stream.write_u8(self.delta_steps_from_previous)?;
        stream.write_u8(self.authoritative_steps.len() as u8)?;
        for authoritative_step_payload in &self.authoritative_steps {
            stream.write_u8(authoritative_step_payload.len() as u8)?;
            stream.write(authoritative_step_payload.as_slice())?;
        }
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
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
    start_step_id: u32,
    ranges: Vec<AuthoritativeStepRange>,
}

impl AuthoritativeStepRanges {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        stream.write_u32(self.start_step_id)?;
        stream.write_u8(self.ranges.len() as u8)?;
        for range in &self.ranges {
            range.to_stream(stream)?;
        }
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
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

#[derive(Debug, PartialEq)]
pub struct GameStepResponse {
    pub response_header: GameStepResponseHeader,
    pub authoritative_ranges: AuthoritativeStepRanges,
}

impl GameStepResponse {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        self.response_header.to_stream(stream)?;
        self.authoritative_ranges.to_stream(stream)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        Ok(Self {
            response_header: GameStepResponseHeader::from_stream(stream)?,
            authoritative_ranges: AuthoritativeStepRanges::from_stream(stream)?,
        })
    }
}
