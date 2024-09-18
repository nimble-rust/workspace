use crate::client::ClientPhase::Connected;
use crate::datagram_build::{NimbleDatagramBuilder, NimbleOobDatagramBuilder};
use crate::datagram_parse::{DatagramType, NimbleDatagramParser};
use datagram::DatagramBuilder;
use datagram_builder::serialize::serialize_datagrams;
use flood_rs::prelude::{InOctetStream, OutOctetStream};
use flood_rs::ReadOctetStream;
use log::info;
use nimble_client_connecting::ConnectingClient;
use nimble_client_logic::logic::ClientLogic;
use nimble_connection_layer::ConnectionSecretSeed;
use nimble_protocol::prelude::{HostToClientCommands, HostToClientOobCommands};
use nimble_protocol::{ClientRequestId, Version};
use secure_random::SecureRandom;
use std::cell::RefCell;
use std::io;
use std::io::{Error, ErrorKind};
use std::rc::Rc;

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum ClientPhase<
    GameT: nimble_seer::SeerCallback<nimble_protocol::client_to_host::AuthoritativeStep<StepT>>
        + nimble_assent::AssentCallback<nimble_protocol::client_to_host::AuthoritativeStep<StepT>>
        + nimble_rectify::RectifyCallback,
    StepT: std::clone::Clone + flood_rs::Deserialize + flood_rs::Serialize + std::fmt::Debug,
> {
    Connecting(ConnectingClient),
    Connected(ClientLogic<GameT, StepT>),
}
pub struct ClientStream<
    GameT: nimble_seer::SeerCallback<nimble_protocol::client_to_host::AuthoritativeStep<StepT>>
        + nimble_assent::AssentCallback<nimble_protocol::client_to_host::AuthoritativeStep<StepT>>
        + nimble_rectify::RectifyCallback,
    StepT: std::clone::Clone + flood_rs::Deserialize + flood_rs::Serialize + std::fmt::Debug,
> {
    datagram_parser: NimbleDatagramParser,
    datagram_builder: NimbleDatagramBuilder,
    oob_datagram_builder: NimbleOobDatagramBuilder,
    phase: ClientPhase<GameT, StepT>,
    random: Rc<RefCell<dyn SecureRandom>>,
}

impl<
        GameT: nimble_seer::SeerCallback<nimble_protocol::client_to_host::AuthoritativeStep<StepT>>
            + nimble_assent::AssentCallback<nimble_protocol::client_to_host::AuthoritativeStep<StepT>>
            + nimble_rectify::RectifyCallback,
        StepT: std::clone::Clone + flood_rs::Deserialize + flood_rs::Serialize + std::fmt::Debug,
    > ClientStream<GameT, StepT>
{
    pub fn new(random: Rc<RefCell<dyn SecureRandom>>, application_version: &Version) -> Self {
        let nimble_protocol_version = Version {
            major: 0,
            minor: 0,
            patch: 5,
        };
        let client_request_id = ClientRequestId(random.borrow_mut().get_random_u64());
        const DATAGRAM_MAX_SIZE: usize = 1024;
        Self {
            random,
            datagram_parser: NimbleDatagramParser::new(),
            datagram_builder: NimbleDatagramBuilder::new(DATAGRAM_MAX_SIZE),
            oob_datagram_builder: NimbleOobDatagramBuilder::new(DATAGRAM_MAX_SIZE),
            phase: ClientPhase::Connecting(ConnectingClient::new(
                client_request_id,
                *application_version,
                nimble_protocol_version,
            )),
        }
    }

    fn connecting_receive(&mut self, mut in_octet_stream: InOctetStream) -> io::Result<()> {
        let connecting_client = match self.phase {
            ClientPhase::Connecting(ref mut connecting_client) => connecting_client,
            _ => Err(Error::new(ErrorKind::InvalidData, "bad phase"))?,
        };

        let command = HostToClientOobCommands::from_stream(&mut in_octet_stream)?;
        connecting_client
            .receive(&command)
            .map_err(|err| Error::new(ErrorKind::InvalidData, err.to_string()))?;
        if let Some(connected_info) = connecting_client.connected_info() {
            info!("connected!");
            let seed = ConnectionSecretSeed(connected_info.session_connection_secret.value as u32);
            self.datagram_builder.set_secrets(
                nimble_connection_layer::ConnectionId {
                    value: connected_info.connection_id.0,
                },
                seed,
            );

            self.phase = Connected(ClientLogic::new(self.random.clone()));
        }
        Ok(())
    }

    pub fn connecting_receive_front(&mut self, payload: &[u8]) -> io::Result<()> {
        let (datagram_type, in_stream) = self.datagram_parser.parse(payload, None)?;
        match datagram_type {
            DatagramType::Oob => self.connecting_receive(in_stream),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "can only receive Oob until connected",
            )),
        }
    }

    fn connected_receive(&mut self, in_stream: &mut InOctetStream) -> io::Result<()> {
        let logic = match self.phase {
            ClientPhase::Connected(ref mut logic) => logic,
            _ => Err(Error::new(ErrorKind::InvalidData, "bad phase"))?,
        };
        while !in_stream.has_reached_end() {
            let cmd = HostToClientCommands::from_stream(in_stream)?;
            logic
                .receive_cmd(&cmd)
                .map_err(|err| Error::new(ErrorKind::InvalidData, err.to_string()))?;
        }
        Ok(())
    }

    pub fn connected_receive_front(&mut self, payload: &[u8]) -> io::Result<()> {
        let (datagram_type, mut in_stream) = self.datagram_parser.parse(payload, None)?;
        match datagram_type {
            DatagramType::Connection(connection_id, client_time) => {
                // TODO: use connection_id from DatagramType::connection_id
                info!("connection: connection_id {connection_id:?} client time {client_time:?}");
                self.connected_receive(&mut in_stream)
            }
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "can only receive connection datagrams when connected",
            )),
        }
    }
    pub fn receive(&mut self, payload: &[u8]) -> io::Result<()> {
        match &mut self.phase {
            ClientPhase::Connecting(_) => self.connecting_receive_front(payload),
            ClientPhase::Connected(_) => self.connected_receive_front(payload),
        }
    }

    fn connecting_send_front(&mut self) -> io::Result<Vec<u8>> {
        let connecting_client = match &mut self.phase {
            ClientPhase::Connecting(ref mut connecting_client) => connecting_client,
            _ => Err(io::Error::new(ErrorKind::InvalidData, "illegal state"))?,
        };
        let request = connecting_client.send();
        let mut out_stream = OutOctetStream::new();
        request.to_stream(&mut out_stream)?;

        self.oob_datagram_builder.clear()?;
        self.oob_datagram_builder
            .push(out_stream.octets().as_slice())
            .map_err(|err| Error::new(ErrorKind::InvalidData, err.to_string()))?;
        Ok(self.oob_datagram_builder.finalize()?.to_vec())
    }

    fn connected_send_front(&mut self) -> io::Result<Vec<Vec<u8>>> {
        let client_logic = match &mut self.phase {
            ClientPhase::Connected(ref mut client_logic) => client_logic,
            _ => Err(io::Error::new(ErrorKind::InvalidData, "illegal state"))?,
        };
        let commands = client_logic.send();
        serialize_datagrams(commands, &mut self.datagram_builder)
    }

    pub fn send(&mut self) -> io::Result<Vec<Vec<u8>>> {
        match &mut self.phase {
            ClientPhase::Connecting(_) => Ok(vec![self.connecting_send_front()?]),
            ClientPhase::Connected(_) => self.connected_send_front(),
        }
    }

    pub fn debug_phase(&self) -> &ClientPhase<GameT, StepT> {
        &self.phase
    }
}
