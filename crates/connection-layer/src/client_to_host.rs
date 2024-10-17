use crate::{RequestId, Version};
use flood_rs::{Deserialize, ReadOctetStream, Serialize, WriteOctetStream};
use std::io;
use std::io::ErrorKind;

#[repr(u8)]
enum ClientToHostCommand {
    Connect = 0x05,
}

impl TryFrom<u8> for ClientToHostCommand {
    type Error = io::Error;

    fn try_from(value: u8) -> io::Result<Self> {
        match value {
            0x05 => Ok(ClientToHostCommand::Connect),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Unknown command {}", value),
            )),
        }
    }
}


#[derive(Debug)]
pub struct ConnectRequest {
    pub request_id: RequestId,
    pub version: Version, // Connection Layer version
}

impl Serialize for ConnectRequest {
    fn serialize(&self, stream: &mut impl WriteOctetStream) -> io::Result<()>
    where
        Self: Sized,
    {
        stream.write_u64(self.request_id)?;
        self.version.serialize(stream)
    }
}

impl Deserialize for ConnectRequest {
    fn deserialize(stream: &mut impl ReadOctetStream) -> io::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            request_id: stream.read_u64()?,
            version: Version::deserialize(stream)?,
        })
    }
}

pub enum ClientToHostCommands {
    Connect(ConnectRequest),
}

impl Serialize for ClientToHostCommands {
    fn serialize(&self, stream: &mut impl WriteOctetStream) -> io::Result<()>
    where
        Self: Sized,
    {
        match self {
            ClientToHostCommands::Connect(connect_request) => {
                stream.write_u8(ClientToHostCommand::Connect as u8)?;
                connect_request.serialize(stream)
            }
        }
    }
}

impl Deserialize for ClientToHostCommands {
    fn deserialize(stream: &mut impl ReadOctetStream) -> io::Result<Self>
    where
        Self: Sized,
    {
        let command_value = stream.read_u8()?;
        let command = ClientToHostCommand::try_from(command_value)?;
        let answer = match command {
            ClientToHostCommand::Connect => {
                let request = ConnectRequest::deserialize(stream)?;
                ClientToHostCommands::Connect(request)
            }
        };
        Ok(answer)
    }
}