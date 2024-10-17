/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
mod client;
mod client_to_host;
mod host_to_client;
pub mod prelude;

use flood_rs::prelude::*;
use log::info;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::{fmt, io};

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Nonce(pub u64);

impl Nonce {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> std::io::Result<()> {
        stream.write_u64(self.0)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> std::io::Result<Self> {
        let x = stream.read_u64()?;
        Ok(Self(x))
    }
}

impl fmt::Display for Nonce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Nonce({:X})", self.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ConnectionId(pub u64);

impl ConnectionId {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> std::io::Result<()> {
        stream.write_u64(self.0)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> std::io::Result<Self> {
        let x = stream.read_u64()?;
        Ok(Self(x))
    }
}

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ConnectionId({:X})", self.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ServerChallenge(pub u64);

impl ServerChallenge {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> std::io::Result<()> {
        stream.write_u64(self.0)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> std::io::Result<Self> {
        let x = stream.read_u64()?;
        Ok(Self(x))
    }
}

impl fmt::Display for ServerChallenge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ServerChallenge({:X})", self.0)
    }
}

#[derive(Debug)]
pub struct ClientToHostPacket {
    pub header: PacketHeader,
    pub payload: Vec<u8>,
}

impl ClientToHostPacket {
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> std::io::Result<()> {
        self.header.to_stream(stream)?;
        stream.write(self.payload.as_slice())?;
        Ok(())
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> std::io::Result<Self> {
        let header = PacketHeader::from_stream(stream)?;
        let mut target_buffer = Vec::with_capacity(header.size as usize);
        stream.read(&mut target_buffer)?;
        Ok(Self {
            header,
            payload: target_buffer,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct PacketHeader {
    pub connection_id: ConnectionId,
    pub size: u16,
}

impl PacketHeader {
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> std::io::Result<()> {
        self.connection_id.to_stream(stream)?;
        stream.write_u16(self.size)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> std::io::Result<Self> {
        Ok(Self {
            connection_id: ConnectionId::from_stream(stream)?,
            size: stream.read_u16()?,
        })
    }
}

#[derive(Debug)]
pub struct HostToClientPacketHeader(PacketHeader);

impl HostToClientPacketHeader {
    pub fn from_stream(stream: &mut impl ReadOctetStream) -> std::io::Result<Self> {
        info!("packet from host");
        Ok(Self(PacketHeader::from_stream(stream)?))
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ConnectCommand {
    pub nonce: Nonce,
    pub server_challenge: ServerChallenge,
}

impl ConnectCommand {
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> std::io::Result<()> {
        self.nonce.to_stream(stream)?;
        self.server_challenge.to_stream(stream)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> std::io::Result<Self> {
        Ok(Self {
            nonce: Nonce::from_stream(stream)?,
            server_challenge: ServerChallenge::from_stream(stream)?,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct InChallengeCommand {
    pub nonce: Nonce,
    pub incoming_server_challenge: ServerChallenge,
}

impl InChallengeCommand {
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> std::io::Result<()> {
        self.nonce.to_stream(stream)?;
        self.incoming_server_challenge.to_stream(stream)?;

        Ok(())
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> std::io::Result<Self> {
        Ok(Self {
            nonce: Nonce::from_stream(stream)?,
            incoming_server_challenge: ServerChallenge::from_stream(stream)?,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct ClientToHostChallengeCommand {
    pub nonce: Nonce,
}

impl ClientToHostChallengeCommand {
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> std::io::Result<()> {
        self.nonce.to_stream(stream)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> std::io::Result<Self> {
        Ok(Self {
            nonce: Nonce::from_stream(stream)?,
        })
    }
}

#[derive(Debug)]
pub enum ClientToHostCommands {
    ChallengeType(ClientToHostChallengeCommand),
    ConnectType(ConnectCommand),
    PacketType(ClientToHostPacket),
}

#[derive(Debug, PartialEq)]
pub struct ChallengeResponse {
    pub nonce: Nonce,
}

impl ChallengeResponse {
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> std::io::Result<()> {
        self.nonce.to_stream(stream)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> std::io::Result<Self> {
        Ok(Self {
            nonce: Nonce::from_stream(stream)?,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct ConnectResponse {
    pub nonce: Nonce,
    pub connection_id: ConnectionId,
}

impl ConnectResponse {
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> std::io::Result<()> {
        self.nonce.to_stream(stream)?;
        self.connection_id.to_stream(stream)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> std::io::Result<Self> {
        Ok(Self {
            nonce: Nonce::from_stream(stream)?,
            connection_id: ConnectionId::from_stream(stream)?,
        })
    }
}

#[derive(PartialEq, Debug)]
enum ClientPhase {
    Challenge(Nonce),
    Connecting(Nonce, ServerChallenge),
    Connected(ConnectionId),
}

impl fmt::Display for ClientPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Challenge(nonce) => {
                write!(f, "clientPhase: Challenge Phase with {}", nonce)
            }
            Self::Connecting(nonce, challenge) => write!(
                f,
                "clientPhase: Connecting Phase with {} and {}",
                nonce, challenge
            ),
            Self::Connected(connection_id) => {
                write!(f, "clientPhase: Connected with {}", *connection_id)
            }
        }
    }
}

#[derive(Debug)]
pub enum DatagramConnectionsError {
    IoError(io::Error),
    ReceiveConnectInWrongPhase,
    WrongNonceWhileConnecting,
    WrongNonceInChallenge,
    ReceivedChallengeInWrongPhase,
    WrongConnectionId,
    ReceivedPacketInWrongPhase,
    SendChallengeInWrongPhase,
    SendConnectRequestInWrongPhase,
    SendPacketInWrongPhase,
}

impl Display for DatagramConnectionsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for DatagramConnectionsError {}
