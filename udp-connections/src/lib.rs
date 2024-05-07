use flood_rs::{OutOctetStream, ReadOctetStream, WriteOctetStream};

use datagram::DatagramSender;
use secure_random::get_random_u64;

#[derive(Debug, PartialEq)]
pub struct Nonce(pub u64);

impl Nonce {
    pub fn new(value: u64) -> Self {
        Self {
            0: value,
        }
    }
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        stream.write_u64(self.0)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        let x = stream.read_u64()?;
        Ok(Self(x))
    }
}

#[derive(Debug, PartialEq)]
pub struct ConnectionId(pub u64);

impl ConnectionId {
    pub fn new(value: u64) -> Self {
        Self {
            0: value,
        }
    }
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        stream.write_u64(self.0)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        let x = stream.read_u64()?;
        Ok(Self(x))
    }
}

#[derive(Debug, PartialEq)]
pub struct ServerChallenge(pub u64);

impl ServerChallenge {
    pub fn new(value: u64) -> Self {
        Self {
            0: value,
        }
    }
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        stream.write_u64(self.0)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        let x = stream.read_u64()?;
        Ok(Self(x))
    }
}

#[derive(Debug, PartialEq)]
pub struct PacketHeader {
    pub connection_id: ConnectionId,
    pub size: u16,
}

impl PacketHeader {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        self.connection_id.to_stream(stream)?;
        stream.write_u16(self.size)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        Ok(Self {
            connection_id: ConnectionId::from_stream(stream)?,
            size: stream.read_u16()?,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct ConnectCommand {
    pub nonce: Nonce,
    pub server_challenge: ServerChallenge,
}

impl ConnectCommand {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        self.nonce.to_stream(stream)?;
        self.server_challenge.to_stream(stream)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        Ok(Self {
            nonce: Nonce::from_stream(stream)?,
            server_challenge: ServerChallenge::from_stream(stream)?,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct InChallengeCommand {
    pub nonce: Nonce,
}

impl InChallengeCommand {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        self.nonce.to_stream(stream)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
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
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        self.nonce.to_stream(stream)?;
        self.connection_id.to_stream(stream)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        Ok(Self {
            nonce: Nonce::from_stream(stream)?,
            connection_id: ConnectionId::from_stream(stream)?,
        })
    }
}

pub struct Client {
    sender: Box<dyn DatagramSender>,
}

impl Client {
    pub fn new(sender: Box<dyn DatagramSender>) -> Self {
        Self {
            sender
        }
    }
}

impl DatagramSender for Client {
    fn send_datagram(&self, data: &[u8]) -> std::io::Result<()> {
        let mut out_stream = OutOctetStream::new();
        let challenge = InChallengeCommand {
            nonce: Nonce(get_random_u64()),
        };
        challenge.to_stream(&mut out_stream).unwrap();
        out_stream.write(data)?;
        self.sender.send_datagram(out_stream.data.as_slice())
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
