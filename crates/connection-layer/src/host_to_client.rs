use crate::{ConnectionId, ConnectionSecretSeed, RequestId};
use flood_rs::{Deserialize, ReadOctetStream, Serialize, WriteOctetStream};
use std::io;
use std::io::ErrorKind;

#[repr(u8)]
enum HostToClientCommand {
    Connect = 0x06,
}

impl TryFrom<u8> for HostToClientCommand {
    type Error = io::Error;

    fn try_from(value: u8) -> io::Result<Self> {
        match value {
            0x06 => Ok(HostToClientCommand::Connect),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Unknown command {}", value),
            )),
        }
    }
}

#[derive(Debug)]
pub struct ConnectResponse {
    pub request_id: RequestId,
    pub connection_id: ConnectionId,
    pub seed: ConnectionSecretSeed,
}

impl Serialize for ConnectResponse {
    fn serialize(&self, stream: &mut impl WriteOctetStream) -> io::Result<()>
    where
        Self: Sized,
    {
        stream.write_u64(self.request_id)?;
        stream.write_u8(self.connection_id.value)?;
        stream.write_u32(self.seed.0)
    }
}

impl Deserialize for ConnectResponse {
    fn deserialize(stream: &mut impl ReadOctetStream) -> io::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            request_id: stream.read_u64()?,
            connection_id: ConnectionId {
                value: stream.read_u8()?,
            },
            seed: ConnectionSecretSeed(stream.read_u32()?),
        })
    }
}

pub enum HostToClientCommands {
    Connect(ConnectResponse),
}

impl Serialize for HostToClientCommands {
    fn serialize(&self, stream: &mut impl WriteOctetStream) -> io::Result<()>
    where
        Self: Sized,
    {
        stream.write_u8(HostToClientCommand::Connect as u8)?;
        match self {
            HostToClientCommands::Connect(connect_response) => connect_response.serialize(stream),
        }
    }
}

impl Deserialize for HostToClientCommands {
    fn deserialize(stream: &mut impl ReadOctetStream) -> io::Result<Self>
    where
        Self: Sized,
    {
        let command_value = stream.read_u8()?;
        let command = HostToClientCommand::try_from(command_value)?;
        let answer = match command {
            HostToClientCommand::Connect => {
                let response = ConnectResponse::deserialize(stream)?;
                HostToClientCommands::Connect(response)
            }
        };
        Ok(answer)
    }
}
