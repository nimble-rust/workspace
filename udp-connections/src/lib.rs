use std::io;
use std::io::{Error, ErrorKind};

use flood_rs::{InOctetStream, OutOctetStream, ReadOctetStream, WriteOctetStream};

use datagram::{DatagramCommunicator, DatagramReceiver, DatagramSender};
use secure_random::get_random_u64;

#[derive(Debug, PartialEq)]
pub struct Nonce(pub u64);

impl Nonce {
    pub fn new(value: u64) -> Self {
        Self { 0: value }
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
        Self { 0: value }
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
        Self { 0: value }
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

#[derive(Debug)]
pub struct ClientToHostPacketHeader(PacketHeader);
#[derive(Debug)]
pub struct HostToClientPacketHeader(PacketHeader);


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
pub struct ClientToHostChallengeCommand {
    pub nonce: Nonce,
}

impl ClientToHostChallengeCommand {
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



#[derive(Debug)]
pub enum ClientToHostCommands {
    ChallengeType(ClientToHostChallengeCommand),
    ConnectType(ConnectCommand),
    PacketType(ClientToHostPacketHeader),
}

#[repr(u8)]
enum HostToClientCommand {
    Challenge = 0x11,
    Connect = 0x12,
    Packet = 0x13,
}

#[derive(Debug)]
pub enum HostToClientCommands {
    ChallengeType(InChallengeCommand),
    ConnectType(ConnectResponse),
    PacketType(HostToClientPacketHeader),
}


impl HostToClientCommands {
    pub fn to_octet(&self) -> HostToClientCommand {
        match self {
            HostToClientCommands::ChallengeType(_) => HostToClientCommand::Challenge,
            HostToClientCommands::ConnectType(_) => HostToClientCommand::Connect,
            HostToClientCommands::PacketType(_) => HostToClientCommand::Packet,
        }
    }

    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.to_octet() as u8)?;
        match self {
            HostToClientCommands::ChallengeType(client_to_host_challenge) => client_to_host_challenge.to_stream(stream),
            HostToClientCommands::ConnectType(connect_command) => connect_command.to_stream(stream),
            HostToClientCommands::PacketType(client_to_host_packet) => client_to_host_packet.0.to_stream(stream),
        }
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        let command = stream.read_u8()?;
        let x = match command {
            CHALLENGE_COMMAND => HostToClientCommands::ChallengeType(InChallengeCommand::from_stream(stream)?),
            _ => {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("unknown command {}", command),
                ));
            }
        };
        Ok(x)
    }
}


#[derive(Debug, PartialEq)]
pub struct ChallengeResponse {
    pub nonce: Nonce,
}

impl ChallengeResponse {
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

/*
#define UdpConnectionsSerializeCmdChallenge (0x01)
#define UdpConnectionsSerializeCmdConnect (0x02)
#define UdpConnectionsSerializeCmdPacket (0x03)

#define UdpConnectionsSerializeCmdChallengeResponse (0x11)
#define UdpConnectionsSerializeCmdConnectResponse (0x12)
#define UdpConnectionsSerializeCmdPacketToClient (0x13)
 */


#[repr(u8)]
enum ClientToHostCommand {
    Challenge = 0x01,
    Connect = 0x02,
    Packet = 0x03,
}

// Implement TryFrom to convert u8 to Command
impl TryFrom<u8> for ClientToHostCommand {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(ClientToHostCommand::Challenge),
            0x02 => Ok(ClientToHostCommand::Connect),
            0x03 => Ok(ClientToHostCommand::Packet),
            _ => Err("Unknown command"),
        }
    }
}

fn convert_to_io_result(byte: u8) -> io::Result<ClientToHostCommand> {
    ClientToHostCommand::try_from(byte).map_err(|e| {
        Error::new(ErrorKind::InvalidData, e)
    })
}

impl ClientToHostCommands {
    pub fn to_octet(&self) -> ClientToHostCommand {
        match self {
            ClientToHostCommands::ChallengeType(_) => ClientToHostCommand::Challenge,
            ClientToHostCommands::ConnectType(_) => ClientToHostCommand::Connect,
            ClientToHostCommands::PacketType(_) => ClientToHostCommand::Packet,
        }
    }

    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.to_octet() as u8)?;
        match self {
            ClientToHostCommands::ChallengeType(client_to_host_challenge) => client_to_host_challenge.to_stream(stream),
            ClientToHostCommands::ConnectType(connect_command) => connect_command.to_stream(stream),
            ClientToHostCommands::PacketType(client_to_host_packet) => client_to_host_packet.0.to_stream(stream),
        }
    }



    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        let command_value = stream.read_u8()?;
        let command = convert_to_io_result(command_value)?;
        let x = match command {
            ClientToHostCommand::Challenge => ClientToHostCommands::ChallengeType(ClientToHostChallengeCommand::from_stream(stream)?),
            _ => {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("unknown command {}", command_value),
                ));
            }
        };
        Ok(x)
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
    communicator: Box<dyn DatagramCommunicator>,
}

impl Client {
    pub fn new(
        communicator: Box<dyn DatagramCommunicator>,
    ) -> Self {
        Self { communicator }
    }

    pub fn on_challenge(&mut self, cmd: InChallengeCommand)  -> io::Result<usize> {
        Ok(0)
    }

    pub fn on_connect(&mut self, cmd: ConnectResponse)  -> io::Result<usize> {
        Ok(0)
    }

    pub fn on_packet(&mut self, cmd: HostToClientPacketHeader)  -> io::Result<usize> {
        Ok(0)
    }
}


impl DatagramCommunicator for Client {
    fn send_datagram(&mut self, data: &[u8]) -> io::Result<()> {
        let mut out_stream = OutOctetStream::new();
        let challenge = ClientToHostChallengeCommand {
            nonce: Nonce(get_random_u64()),
        };
        let client_command = ClientToHostCommands::ChallengeType(challenge);

        client_command.to_stream(&mut out_stream).unwrap();
        out_stream.write(data)?;
        self.communicator.send_datagram(out_stream.data.as_slice())
    }

    fn receive_datagram(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let mut in_stream = InOctetStream::new(buffer.to_vec());
        let command = HostToClientCommands::from_stream(&mut in_stream)?;
        match command {
            HostToClientCommands::ChallengeType(challenge_command) => self.on_challenge(challenge_command),
            HostToClientCommands::ConnectType(connect_command) => self.on_connect(connect_command),
            HostToClientCommands::PacketType(packet_command) => self.on_packet(packet_command),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
