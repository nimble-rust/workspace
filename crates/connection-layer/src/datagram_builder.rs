use crate::{verify_hash, write_to_stream, ConnectionId, ConnectionSecretSeed};
use datagram::{DatagramDecoder, DatagramEncoder};
use flood_rs::prelude::{InOctetStream, OutOctetStream};
use flood_rs::{Deserialize, ReadOctetStream, Serialize, WriteOctetStream};
use freelist_rs::FreeList;
use secure_random::SecureRandom;
use std::collections::HashMap;
use std::io;
use std::io::ErrorKind;

pub struct ConnectionInfo {
    connection_id: ConnectionId,
    seed: ConnectionSecretSeed,
}

pub struct HostConnection {
    created_from_request: RequestId,
    connection_id: ConnectionId,
    seed: ConnectionSecretSeed,
    pub has_received_connect: bool,
}

pub struct ConnectionLayerHostCodec {
    pub connection_ids: FreeList,
    pub connections: HashMap<u8, HostConnection>,
    pub random: Box<dyn SecureRandom>,
}

impl ConnectionLayerHostCodec {
    pub fn new(random: Box<dyn SecureRandom>) -> Self {
        let mut s = Self {
            connections: HashMap::new(),
            connection_ids: FreeList::new(0xff),
            random,
        };
        s.connection_ids.allocate(); // Reserve zero

        s
    }
}

pub struct ConnectionLayerClientCodec {
    pub connection_info: Option<ConnectionInfo>,
    pub request_id: RequestId,
}

impl ConnectionLayerClientCodec {
    pub fn new(request_id: RequestId) -> Self {
        Self {
            connection_info: None,
            request_id,
        }
    }
}

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

struct Version {
    pub major: u8,
    pub minor: u8,
}

impl Serialize for Version {
    fn serialize(&self, stream: &mut impl WriteOctetStream) -> io::Result<()>
    where
        Self: Sized,
    {
        stream.write_u8(self.major)?;
        stream.write_u8(self.minor)
    }
}

impl Deserialize for Version {
    fn deserialize(stream: &mut impl ReadOctetStream) -> io::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            major: stream.read_u8()?,
            minor: stream.read_u8()?,
        })
    }
}

pub type RequestId = u64; // So it is very likely that this number will change for each connection attempt
struct ConnectRequest {
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

enum ClientToHostCommands {
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

struct ConnectResponse {
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

enum HostToClientCommands {
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

pub trait DatagramHostEncoder {
    fn encode(&mut self, connection_id: u8, buf: &[u8]) -> io::Result<Vec<u8>>;
}

impl DatagramHostEncoder for ConnectionLayerHostCodec {
    fn encode(&mut self, connection_id: u8, buf: &[u8]) -> io::Result<Vec<u8>> {
        let connection = self.connections.get_mut(&connection_id);
        if connection.is_none() {
            Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Unknown connection {}", connection_id),
            ))?;
        }
        let actual_connection = connection.unwrap();
        let mut stream = OutOctetStream::new();
        if actual_connection.has_received_connect {
            write_to_stream(
                &mut stream,
                actual_connection.connection_id,
                actual_connection.seed,
                buf,
            )?;
        } else {
            ConnectionId { value: 0 }.to_stream(&mut stream)?;
            let connect_response = ConnectResponse {
                request_id: actual_connection.created_from_request,
                connection_id: actual_connection.connection_id,
                seed: actual_connection.seed,
            };
            HostToClientCommands::Connect(connect_response).serialize(&mut stream)?
        }

        flood_rs::WriteOctetStream::write(&mut stream, buf)?;

        Ok(stream.octets().to_vec())
    }
}

pub trait DatagramHostDecoder {
    fn decode(&mut self, buf: &[u8]) -> io::Result<(u8, Vec<u8>)>;
}

impl DatagramHostDecoder for ConnectionLayerHostCodec {
    fn decode(&mut self, buf: &[u8]) -> io::Result<(u8, Vec<u8>)> {
        let mut in_stream = InOctetStream::new(buf);
        let connection_id = ConnectionId::from_stream(&mut in_stream)?;
        if connection_id.value != 0 {
            if let Some(connection) = self.connections.get_mut(&connection_id.value) {
                let murmur = in_stream.read_u32()?;
                verify_hash(murmur, connection.seed, &buf[5..])?;
                connection.has_received_connect = true;
                //                Ok(buf[5..].to_vec())
                Ok((
                    connection_id.value,
                    buf[in_stream.cursor.position() as usize..].to_vec(),
                ))
            } else {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "unknown connection_id",
                ))?
            }
        } else {
            // OOB
            let command = ClientToHostCommands::deserialize(&mut in_stream)?;
            match command {
                ClientToHostCommands::Connect(connect_request) => {
                    let assigned_connection_id = self.connection_ids.allocate().ok_or(
                        io::Error::new(io::ErrorKind::InvalidData, "free list problem"),
                    )?;
                    let new_connection = HostConnection {
                        created_from_request: connect_request.request_id,
                        connection_id: ConnectionId {
                            value: assigned_connection_id,
                        },
                        seed: ConnectionSecretSeed(self.random.get_random_u64() as u32),
                        has_received_connect: false,
                    };
                    self.connections
                        .insert(assigned_connection_id, new_connection);
                    Ok((
                        assigned_connection_id,
                        buf[in_stream.cursor.position() as usize..].to_vec(),
                    ))
                }
            }
        }
    }
}

impl DatagramEncoder for ConnectionLayerClientCodec {
    fn encode(&mut self, buf: &[u8]) -> io::Result<Vec<u8>> {
        let mut stream = OutOctetStream::new();
        match &self.connection_info {
            None => {
                ConnectionId { value: 0 }.to_stream(&mut stream)?;
                let connect_request = ConnectRequest {
                    request_id: self.request_id,
                    version: Version { major: 0, minor: 2 },
                };
                ClientToHostCommands::Connect(connect_request).serialize(&mut stream)?;
            }
            Some(connection_info) => write_to_stream(
                &mut stream,
                connection_info.connection_id,
                connection_info.seed,
                buf,
            )?,
        }
        flood_rs::WriteOctetStream::write(&mut stream, buf)?;

        Ok(stream.octets().to_vec())
    }
}

impl DatagramDecoder for ConnectionLayerClientCodec {
    fn decode(&mut self, buf: &[u8]) -> io::Result<Vec<u8>> {
        let mut in_stream = InOctetStream::new(buf);
        let connection_id = ConnectionId::from_stream(&mut in_stream)?;

        match &self.connection_info {
            None => {
                let command = HostToClientCommands::deserialize(&mut in_stream)?;
                match command {
                    HostToClientCommands::Connect(connect_response) => {
                        self.connection_info = Some(ConnectionInfo {
                            connection_id: connect_response.connection_id,
                            seed: connect_response.seed,
                        })
                    }
                }
                Ok(buf[in_stream.cursor.position() as usize..].to_vec())
            }
            Some(connection_info) => {
                if connection_id != connection_info.connection_id {
                    Err(io::Error::new(io::ErrorKind::InvalidData, "problem"))
                } else {
                    let murmur = in_stream.read_u32()?;
                    verify_hash(murmur, connection_info.seed, &buf[5..])?;
                    Ok(buf[5..].to_vec())
                }
            }
        }
    }
}
