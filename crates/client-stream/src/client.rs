use crate::datagram_parse::{DatagramType, NimbleDatagramParser};
use flood_rs::prelude::InOctetStream;
use flood_rs::ReadOctetStream;
use log::info;
use nimble_client_connecting::ConnectingClient;
use nimble_client_logic::logic::ClientLogic;
use nimble_connection_layer::{ConnectionId, ConnectionSecretSeed};
use nimble_protocol::prelude::HostToClientCommands;
use nimble_protocol::{Nonce, Version};
use secure_random::{GetRandom, SecureRandom};
use std::io;
use std::io::{Error, ErrorKind};

#[derive(Debug)]
enum ClientPhase<GameT: nimble_seer::SeerCallback<nimble_protocol::client_to_host::AuthoritativeStep<StepT>> + nimble_assent::AssentCallback<nimble_protocol::client_to_host::AuthoritativeStep<StepT>> + nimble_rectify::RectifyCallback, StepT: std::clone::Clone + flood_rs::Deserialize + flood_rs::Serialize + std::fmt::Debug> {
    Connecting(ConnectingClient),
    Connected(ClientLogic<GameT, StepT>),
}
pub struct ClientStream<GameT: nimble_seer::SeerCallback<nimble_protocol::client_to_host::AuthoritativeStep<StepT>> + nimble_assent::AssentCallback<nimble_protocol::client_to_host::AuthoritativeStep<StepT>> + nimble_rectify::RectifyCallback, StepT: std::clone::Clone + flood_rs::Deserialize + flood_rs::Serialize + std::fmt::Debug> {
    pub datagram_parser: NimbleDatagramParser,
    pub phase: ClientPhase<GameT, StepT>,
}

impl<GameT: nimble_seer::SeerCallback<nimble_protocol::client_to_host::AuthoritativeStep<StepT>> + nimble_assent::AssentCallback<nimble_protocol::client_to_host::AuthoritativeStep<StepT>> + nimble_rectify::RectifyCallback, StepT: std::clone::Clone + flood_rs::Deserialize + flood_rs::Serialize + std::fmt::Debug> ClientStream<GameT, StepT> {
    pub fn new(random: &mut dyn SecureRandom, application_version: &Version) -> Self {
        let nimble_protocol_version = Version {
            major: 0,
            minor: 0,
            patch: 5,
        };
        let nonce = Nonce(random.get_random_u64());
        Self {
            datagram_parser: NimbleDatagramParser::new(),
            phase: ClientPhase::Connecting(ConnectingClient::new(nonce, *application_version, nimble_protocol_version)),
        }
    }

    fn connecting_receive(&mut self, in_octet_stream: InOctetStream) -> io::Result<()> {
        Ok(())
    }

    pub fn connecting_receive_front(&mut self, payload: &[u8]) -> io::Result<()> {
        let (datagram_type, mut in_stream) = self.datagram_parser.parse(payload, None)?;
        match datagram_type {
            DatagramType::Oob => {
                self.connecting_receive(in_stream)
            }
            _ => Err(Error::new(ErrorKind::InvalidData, "can only receive Oob until connected"))
        }
    }

    fn connected_receive(&mut self, mut in_stream: &mut InOctetStream) -> io::Result<()> {
        let logic = match self.phase {
            ClientPhase::Connected(ref mut logic) => logic,
            _ => Err(Error::new(ErrorKind::InvalidData, "bad phase"))?
        };
        while !in_stream.has_reached_end() {
            let cmd = HostToClientCommands::from_stream(in_stream)?;
            logic.receive_cmd(&cmd).map_err(|err| Error::new(ErrorKind::InvalidData, err.to_string()))?;
        }
        Ok(())
    }

    pub fn connected_receive_front(&mut self, payload: &[u8]) -> io::Result<()> {

        let (datagram_type, mut in_stream) = self.datagram_parser.parse(payload, None)?;
        match datagram_type {
            DatagramType::Connection(connection_id, client_time) => {
                info!("client time {client_time:?}");
                self.connected_receive(&mut in_stream)
            }
            _ => Err(Error::new(ErrorKind::InvalidData, "can only receive connection datagrams when connected"))
        }
    }
    pub fn receive(&mut self, payload: &[u8]) -> io::Result<()> {
        match &mut self.phase {
            ClientPhase::Connecting(_) => {
                self.connecting_receive_front(payload)
            }
            ClientPhase::Connected(_) => {
                self.connected_receive_front(payload)
            }
        }
    }
}

