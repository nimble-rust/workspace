use std::io;
use std::io::{ErrorKind, Result};

use flood_rs::{ReadOctetStream, WriteOctetStream};

#[derive(Debug, PartialEq)]
pub struct Nonce(pub u64);

impl Nonce {
    pub fn new(value: u64) -> Nonce {
        Self { 0: value }
    }
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> Result<()> {
        stream.write_u64(self.0)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> Result<Self> {
        let x = stream.read_u64()?;
        Ok(Self(x))
    }
}

#[derive(Debug, PartialEq)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl Version {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> Result<()> {
        stream.write_u16(self.major)?;
        stream.write_u16(self.minor)?;
        stream.write_u16(self.patch)?;

        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> Result<Self> {
        Ok(Self {
            major: stream.read_u16()?,
            minor: stream.read_u16()?,
            patch: stream.read_u16()?,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct ConnectCommand {
    pub nimble_version: Version,
    pub use_debug_stream: bool,
    pub application_version: Version,
    pub nonce: Nonce,
}

impl ConnectCommand {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> Result<()> {
        self.nimble_version.to_stream(stream)?;
        stream.write_u8(if self.use_debug_stream { 0x01 } else { 0x00 })?;
        self.application_version.to_stream(stream)?;
        self.nonce.to_stream(stream)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> Result<Self> {
        Ok(Self {
            nimble_version: Version::from_stream(stream)?,
            use_debug_stream: stream.read_u8()? != 0,
            application_version: Version::from_stream(stream)?,
            nonce: Nonce::from_stream(stream)?,
        })
    }
}

pub struct ConnectionId {
    pub value: u8,
}

impl ConnectionId {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> Result<()> {
        stream.write_u8(self.value)
    }
}

#[derive(Debug)]
pub enum ClientSendCommands {
    ConnectType(ConnectCommand),
}

impl ClientSendCommands {
    pub fn to_octet(&self) -> u8 {
        match self {
            ClientSendCommands::ConnectType(_) => 0x05,
        }
    }

    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> Result<()> {
        stream.write_u8(self.to_octet())?;
        match self {
            ClientSendCommands::ConnectType(connect_command) => connect_command.to_stream(stream),
        }
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> Result<Self> {
        let command = stream.read_u8()?;
        let x = match command {
            0x05 => ClientSendCommands::ConnectType(ConnectCommand::from_stream(stream)?),
            _ => {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("unknown command {}", command),
                ))
            }
        };
        Ok(x)
    }
}

#[cfg(test)]
mod tests {
    use flood_rs::{InOctetStream, OutOctetStream};

    use crate::{ConnectCommand, Nonce, Version};

    #[test]
    fn check_version() {
        let mut out_stream = OutOctetStream::new();
        let version = Version {
            major: 4,
            minor: 3,
            patch: 2,
        };
        version.to_stream(&mut out_stream).unwrap()
    }

    #[test]
    fn check_connect() {
        let mut out_stream = OutOctetStream::new();
        let version = Version {
            major: 4,
            minor: 3,
            patch: 2,
        };
        let nimble_version = Version {
            major: 99,
            minor: 66,
            patch: 33,
        };
        let connect = ConnectCommand {
            nimble_version,
            use_debug_stream: false,
            application_version: version,
            nonce: Nonce(0xff4411ff),
        };
        connect.to_stream(&mut out_stream).unwrap();

        let mut in_stream = InOctetStream::new(Vec::from(out_stream.data));

        let received_connect = ConnectCommand::from_stream(&mut in_stream).unwrap();

        assert_eq!(received_connect, connect);
    }
}
