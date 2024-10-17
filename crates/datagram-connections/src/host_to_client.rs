use crate::{ConnectResponse, HostToClientPacketHeader, InChallengeCommand};
use flood_rs::{ReadOctetStream, WriteOctetStream};
use std::io;

#[repr(u8)]
pub enum HostToClientCommand {
    Challenge = 0x11,
    Connect = 0x12,
    Packet = 0x13,
}

impl TryFrom<u8> for HostToClientCommand {
    type Error = io::Error;

    fn try_from(value: u8) -> io::Result<Self> {
        match value {
            0x11 => Ok(Self::Challenge),
            0x12 => Ok(Self::Connect),
            0x13 => Ok(Self::Packet),
            _ => Err(io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unknown HostToClient DatagramConnections Command {}", value),
            )),
        }
    }
}

#[derive(Debug)]
pub enum HostToClientCommands {
    ChallengeType(InChallengeCommand),
    ConnectType(ConnectResponse),
    PacketType(HostToClientPacketHeader),
}

impl HostToClientCommands {
    #[allow(unused)]
    pub fn to_octet(&self) -> HostToClientCommand {
        match self {
            Self::ChallengeType(_) => HostToClientCommand::Challenge,
            Self::ConnectType(_) => HostToClientCommand::Connect,
            Self::PacketType(_) => HostToClientCommand::Packet,
        }
    }

    #[allow(unused)]
    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.to_octet() as u8)?;
        match self {
            Self::ChallengeType(client_to_host_challenge) => {
                client_to_host_challenge.to_stream(stream)
            }
            Self::ConnectType(connect_command) => connect_command.to_stream(stream),
            Self::PacketType(client_to_host_packet) => client_to_host_packet.0.to_stream(stream),
        }
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        let command_value = stream.read_u8()?;
        let command = HostToClientCommand::try_from(command_value)?;
        let x = match command {
            HostToClientCommand::Challenge => {
                Self::ChallengeType(InChallengeCommand::from_stream(stream)?)
            }
            HostToClientCommand::Connect => {
                Self::ConnectType(ConnectResponse::from_stream(stream)?)
            }
            HostToClientCommand::Packet => {
                Self::PacketType(HostToClientPacketHeader::from_stream(stream)?)
            }
        };
        Ok(x)
    }
}
