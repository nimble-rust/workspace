use crate::host_to_client::HostToClientCommands;
use crate::{
    ClientPhase, ClientToHostChallengeCommand, ClientToHostCommands, ClientToHostPacket,
    ConnectCommand, ConnectResponse, DatagramConnectionsError, HostToClientPacketHeader,
    InChallengeCommand, Nonce, PacketHeader,
};
use datagram::{DatagramDecoder, DatagramEncoder};
use flood_rs::in_stream::InOctetStream;
use flood_rs::out_stream::OutOctetStream;
use flood_rs::{ReadOctetStream, WriteOctetStream};
use hexify::format_hex;
use log::{info, trace};
use secure_random::SecureRandom;
use std::io;

impl DatagramEncoder for Client {
    fn encode(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        let mut out_stream = OutOctetStream::new();

        let client_to_server_cmd = self
            .send(data)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        client_to_server_cmd.to_stream(&mut out_stream)?;
        out_stream.write(data)?;

        Ok(out_stream.octets())
    }
}

impl DatagramDecoder for Client {
    fn decode(&mut self, buffer: &[u8]) -> io::Result<Vec<u8>> {
        self.decode(buffer)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }
}

pub struct Client {
    phase: ClientPhase,
}

impl Client {
    pub fn new(mut random: Box<dyn SecureRandom>) -> Self {
        let phase = ClientPhase::Challenge(Nonce(random.random_u64()));
        Self { phase }
    }

    pub fn on_challenge(
        &mut self,
        cmd: InChallengeCommand,
    ) -> Result<(), DatagramConnectionsError> {
        match self.phase {
            ClientPhase::Challenge(nonce) => {
                if cmd.nonce != nonce {
                    return Err(DatagramConnectionsError::WrongNonceInChallenge);
                }
                self.phase = ClientPhase::Connecting(nonce, cmd.incoming_server_challenge);
                Ok(())
            }
            _ => Err(DatagramConnectionsError::ReceivedChallengeInWrongPhase),
        }
    }

    pub fn on_connect(&mut self, cmd: ConnectResponse) -> Result<(), DatagramConnectionsError> {
        match self.phase {
            ClientPhase::Connecting(nonce, _) => {
                if cmd.nonce != nonce {
                    return Err(DatagramConnectionsError::WrongNonceWhileConnecting);
                }
                info!(
                    "udp_connections: on_connect connected {}",
                    cmd.connection_id
                );
                self.phase = ClientPhase::Connected(cmd.connection_id);
                Ok(())
            }
            _ => Err(DatagramConnectionsError::ReceiveConnectInWrongPhase),
        }
    }

    pub fn on_packet(
        &mut self,
        cmd: HostToClientPacketHeader,
        in_stream: &mut InOctetStream,
    ) -> Result<Vec<u8>, DatagramConnectionsError> {
        match self.phase {
            ClientPhase::Connected(expected_connection_id) => {
                if cmd.0.connection_id != expected_connection_id {
                    return Err(DatagramConnectionsError::WrongConnectionId);
                }
                let mut target_buffer = vec![0u8; cmd.0.size as usize];
                in_stream
                    .read(&mut target_buffer)
                    .map_err(DatagramConnectionsError::IoError)?;
                trace!(
                    "receive packet of size: {} target:{}  {}",
                    cmd.0.size,
                    target_buffer.len(),
                    format_hex(target_buffer.as_slice())
                );
                Ok(target_buffer)
            }
            _ => Err(DatagramConnectionsError::ReceivedPacketInWrongPhase),
        }
    }

    pub fn send_challenge(
        &mut self,
    ) -> Result<ClientToHostChallengeCommand, DatagramConnectionsError> {
        match self.phase {
            ClientPhase::Challenge(nonce) => Ok(ClientToHostChallengeCommand { nonce }),
            _ => Err(DatagramConnectionsError::SendChallengeInWrongPhase),
        }
    }

    pub fn send_connect_request(&mut self) -> Result<ConnectCommand, DatagramConnectionsError> {
        match self.phase {
            ClientPhase::Connecting(nonce, server_challenge) => Ok(ConnectCommand {
                nonce,
                server_challenge,
            }),
            _ => Err(DatagramConnectionsError::SendConnectRequestInWrongPhase),
        }
    }

    pub fn send_packet(
        &mut self,
        data: &[u8],
    ) -> Result<ClientToHostPacket, DatagramConnectionsError> {
        match self.phase {
            ClientPhase::Connected(connection_id) => {
                trace!("send packet: {}", format_hex(data));
                Ok(ClientToHostPacket {
                    header: PacketHeader {
                        connection_id,
                        size: data.len() as u16,
                    },
                    payload: data.to_vec(),
                })
            }
            _ => Err(DatagramConnectionsError::SendPacketInWrongPhase),
        }
    }

    pub fn send(&mut self, data: &[u8]) -> Result<ClientToHostCommands, DatagramConnectionsError> {
        trace!("send: phase: {}", self.phase);
        match self.phase {
            ClientPhase::Challenge(_) => {
                let challenge = self.send_challenge()?;
                Ok(ClientToHostCommands::ChallengeType(challenge))
            }

            ClientPhase::Connecting(_, _) => {
                let connect_request = self.send_connect_request()?;
                Ok(ClientToHostCommands::ConnectType(connect_request))
            }

            ClientPhase::Connected(_) => {
                let packet = self.send_packet(data)?;
                trace!("sending datagram {:?}", packet);
                Ok(ClientToHostCommands::PacketType(packet))
            }
        }
    }

    pub fn decode(&mut self, buffer: &[u8]) -> Result<Vec<u8>, DatagramConnectionsError> {
        let mut in_stream = InOctetStream::new(buffer);
        let command = HostToClientCommands::from_stream(&mut in_stream)
            .map_err(DatagramConnectionsError::IoError)?;

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
