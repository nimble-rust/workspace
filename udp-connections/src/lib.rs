use std::io::{Error, ErrorKind};
use std::{fmt, io};

use flood_rs::{InOctetStream, OutOctetStream, ReadOctetStream, WriteOctetStream};
use log::info;

use datagram::DatagramProcessor;
use secure_random::SecureRandom;

use crate::ClientPhase::{Challenge, Connected, Connecting};

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Nonce(pub u64);

impl Nonce {
    pub fn new(value: u64) -> Self {
        Self(value)
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
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        stream.write_u64(self.0)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
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

    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        stream.write_u64(self.0)?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        let x = stream.read_u64()?;
        Ok(Self(x))
    }
}

impl fmt::Display for ServerChallenge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ServerChallenge({:X})", self.0)
    }
}

fn extremely_unsecure_cypher(public_key: u64, secret_key: u64) -> u64 {
    public_key ^ secret_key
}

#[derive(Debug)]
pub struct ClientToHostPacket {
    pub header: PacketHeader,
    pub payload: Vec<u8>,
}

impl ClientToHostPacket {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        self.header.to_stream(stream)?;
        stream.write(self.payload.as_slice())?;
        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
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
    pub incoming_server_challenge: ServerChallenge,
}

impl InChallengeCommand {
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        self.nonce.to_stream(stream)?;
        self.incoming_server_challenge.to_stream(stream)?;

        Ok(())
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
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
    PacketType(ClientToHostPacket),
}

#[repr(u8)]
enum HostToClientCommand {
    Challenge = 0x11,
    Connect = 0x12,
    Packet = 0x13,
}

impl TryFrom<u8> for HostToClientCommand {
    type Error = io::Error;

    fn try_from(value: u8) -> std::io::Result<Self> {
        match value {
            0x11 => Ok(HostToClientCommand::Challenge),
            0x12 => Ok(HostToClientCommand::Connect),
            0x13 => Ok(HostToClientCommand::Packet),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Unknown HostToClient UdpConnections Command {}", value),
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
    fn to_octet(&self) -> HostToClientCommand {
        match self {
            HostToClientCommands::ChallengeType(_) => HostToClientCommand::Challenge,
            HostToClientCommands::ConnectType(_) => HostToClientCommand::Connect,
            HostToClientCommands::PacketType(_) => HostToClientCommand::Packet,
        }
    }

    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.to_octet() as u8)?;
        match self {
            HostToClientCommands::ChallengeType(client_to_host_challenge) => {
                client_to_host_challenge.to_stream(stream)
            }
            HostToClientCommands::ConnectType(connect_command) => connect_command.to_stream(stream),
            HostToClientCommands::PacketType(client_to_host_packet) => {
                client_to_host_packet.0.to_stream(stream)
            }
        }
    }

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        let command_value = stream.read_u8()?;
        let command = HostToClientCommand::try_from(command_value)?;
        let x = match command {
            HostToClientCommand::Challenge => {
                HostToClientCommands::ChallengeType(InChallengeCommand::from_stream(stream)?)
            }
            HostToClientCommand::Connect => {
                HostToClientCommands::ConnectType(ConnectResponse::from_stream(stream)?)
            }
            _ => {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "UdpConnections: unknown HostToClientCommands command {}",
                        command_value
                    ),
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
    type Error = io::Error;

    fn try_from(value: u8) -> std::io::Result<Self> {
        match value {
            0x01 => Ok(ClientToHostCommand::Challenge),
            0x02 => Ok(ClientToHostCommand::Connect),
            0x03 => Ok(ClientToHostCommand::Packet),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
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

    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
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

    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        let command_value = stream.read_u8()?;
        let command = ClientToHostCommand::try_from(command_value)?;
        let x = match command {
            ClientToHostCommand::Challenge => ClientToHostCommands::ChallengeType(
                ClientToHostChallengeCommand::from_stream(stream)?,
            ),
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

#[derive(PartialEq, Debug)]
enum ClientPhase {
    Challenge(Nonce),
    Connecting(Nonce, ServerChallenge),
    Connected(ConnectionId),
}

impl fmt::Display for ClientPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientPhase::Challenge(nonce) => {
                write!(f, "clientPhase: Challenge Phase with {}", nonce)
            }
            ClientPhase::Connecting(nonce, challenge) => write!(
                f,
                "clientPhase: Connecting Phase with {} and {}",
                nonce, challenge
            ),
            ClientPhase::Connected(connection_id) => {
                write!(f, "clientPhase: Connected with {}", connection_id)
            }
        }
    }
}

pub struct Client {
    random: Box<dyn SecureRandom>,
    phase: ClientPhase,
}

impl Client {
    pub fn new(mut random: Box<dyn SecureRandom>) -> Self {
        let phase = Challenge(Nonce(random.get_random_u64()));
        Self { random, phase }
    }

    pub fn on_challenge(&mut self, cmd: InChallengeCommand) -> io::Result<()> {
        match self.phase {
            Challenge(nonce) => {
                if cmd.nonce != nonce {
                    return Err(Error::new(ErrorKind::InvalidData, "Wrong nonce"));
                }
                self.phase = Connecting(nonce, cmd.incoming_server_challenge);
                Ok(())
            }
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "on_challenge: Message not applicable in current client state {}",
                    self.phase
                ),
            )),
        }
    }

    pub fn on_connect(&mut self, cmd: ConnectResponse) -> io::Result<()> {
        match self.phase {
            Connecting(nonce, _) => {
                if cmd.nonce != nonce {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "Wrong nonce when connecting",
                    ));
                }
                println!("connected {}", cmd.connection_id);
                info!("on_connect connected {}", cmd.connection_id);
                self.phase = Connected(cmd.connection_id);
                Ok(())
            }
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                "can not receive on_connect in current client state",
            )),
        }
    }

    pub fn on_packet(
        &mut self,
        cmd: HostToClientPacketHeader,
        in_stream: &mut InOctetStream,
    ) -> io::Result<Vec<u8>> {
        match self.phase {
            Connected(expected_connection_id) => {
                if cmd.0.connection_id != expected_connection_id {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "Wrong connection_id for received packet",
                    ));
                }
                let mut target_buffer = Vec::with_capacity(cmd.0.size as usize);
                in_stream.read(&mut target_buffer)?;

                Ok(target_buffer)
            }
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                "can not receive on_connect in current client state",
            )),
        }
    }

    pub fn send_challenge(&mut self) -> io::Result<ClientToHostChallengeCommand> {
        match self.phase {
            Challenge(nonce) => Ok(ClientToHostChallengeCommand { nonce }),
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                "can not send_challenge in current client state",
            )),
        }
    }

    pub fn send_connect_request(&mut self) -> io::Result<ConnectCommand> {
        match self.phase {
            Connecting(nonce, server_challenge) => Ok(ConnectCommand {
                nonce,
                server_challenge,
            }),
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                "can not send_connect_request in current client state",
            )),
        }
    }

    pub fn send_packet(&mut self, data: &[u8]) -> io::Result<ClientToHostPacket> {
        match self.phase {
            Connected(connection_id) => Ok(ClientToHostPacket {
                header: PacketHeader {
                    connection_id,
                    size: data.len() as u16,
                },
                payload: data.to_vec(),
            }),
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                "can not send_connect_request in current client state",
            )),
        }
    }

    pub fn send(&mut self, data: &[u8]) -> io::Result<ClientToHostCommands> {
        match self.phase {
            Challenge(_) => {
                let challenge = self.send_challenge()?;
                Ok(ClientToHostCommands::ChallengeType(challenge))
            }

            Connecting(_, _) => {
                let connect_request = self.send_connect_request()?;
                Ok(ClientToHostCommands::ConnectType(connect_request))
            }

            Connected(_) => {
                let challenge = self.send_packet(data)?;
                Ok(ClientToHostCommands::PacketType(challenge))
            }
        }
    }
}

impl DatagramProcessor for Client {
    fn send_datagram(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        let mut out_stream = OutOctetStream::new();

        let client_to_server_cmd = self.send(data)?;

        client_to_server_cmd.to_stream(&mut out_stream).unwrap();
        out_stream.write(data)?;

        Ok(out_stream.data)
    }

    fn receive_datagram(&mut self, buffer: &[u8]) -> io::Result<Vec<u8>> {
        let mut in_stream = InOctetStream::new(buffer.to_vec());
        let command = HostToClientCommands::from_stream(&mut in_stream)?;
        match command {
            HostToClientCommands::ChallengeType(challenge_command) => {
                self.on_challenge(challenge_command)?;
                Ok(vec![])
            }
            HostToClientCommands::ConnectType(connect_command) => {
                self.on_connect(connect_command)?;
                Ok(vec![])
            }
            HostToClientCommands::PacketType(packet_command) => {
                self.on_packet(packet_command, &mut in_stream)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use datagram::DatagramProcessor;
    use secure_random::SecureRandom;

    use crate::Client;

    pub struct FakeRandom {
        pub counter: u64,
    }

    impl SecureRandom for FakeRandom {
        fn get_random_u64(&mut self) -> u64 {
            self.counter += 1;
            self.counter
        }
    }

    #[test]
    fn simple_connection() {
        let random = FakeRandom { counter: 2 };

        let mut client = Client::new(Box::new(random));

        let example = vec![0x18, 0x24, 0x32];

        let datagram_to_send = client
            .send_datagram(example.as_slice())
            .expect("TODO: panic message");

        let expected = vec![
            1, // Challenge command 0x01
            0, 0, 0, 0, 0, 0, 0, 3, // Nonce in network order.
            0x18, 0x24, 0x32,
        ];
        assert_eq!(datagram_to_send, expected, "upd-connections-was wrong")
    }
}
