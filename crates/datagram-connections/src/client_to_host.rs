use crate::{ClientToHostChallengeCommand, ClientToHostCommands};
use flood_rs::{ReadOctetStream, WriteOctetStream};
use std::io;

#[repr(u8)]
pub enum ClientToHostCommand {
    Challenge = 0x01,
    Connect = 0x02,
    Packet = 0x03,
}

// Implement TryFrom to convert u8 to Command
impl TryFrom<u8> for ClientToHostCommand {
    type Error = io::Error;

    fn try_from(value: u8) -> std::io::Result<Self> {
        match value {
            0x01 => Ok(ClientToHostCommand::Challenge),
            0x02 => Ok(ClientToHostCommand::Connect),
            0x03 => Ok(ClientToHostCommand::Packet),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unknown command {}", value),
            )),
        }
    }
}

impl ClientToHostCommands {
    fn to_octet(&self) -> ClientToHostCommand {
        match self {
            ClientToHostCommands::ChallengeType(_) => ClientToHostCommand::Challenge,
            ClientToHostCommands::ConnectType(_) => ClientToHostCommand::Connect,
            ClientToHostCommands::PacketType(_) => ClientToHostCommand::Packet,
        }
    }

    pub fn to_stream(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.to_octet() as u8)?;
        match self {
            ClientToHostCommands::ChallengeType(client_to_host_challenge) => {
                client_to_host_challenge.to_stream(stream)
            }
            ClientToHostCommands::ConnectType(connect_command) => connect_command.to_stream(stream),
            ClientToHostCommands::PacketType(client_to_host_packet) => {
                client_to_host_packet.to_stream(stream)
            }
        }
    }

    pub fn from_stream(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        let command_value = stream.read_u8()?;
        let command = ClientToHostCommand::try_from(command_value)?;
        let x = match command {
            ClientToHostCommand::Challenge => ClientToHostCommands::ChallengeType(
                ClientToHostChallengeCommand::from_stream(stream)?,
            ),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unknown command {}", command_value),
                ));
            }
        };
        Ok(x)
    }
}
