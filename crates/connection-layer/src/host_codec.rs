use crate::client_to_host::ClientToHostCommands;
use crate::host_to_client::{ConnectResponse, HostToClientCommands};
use crate::{verify_hash, write_to_stream, ConnectionId, ConnectionSecretSeed, RequestId};
use flood_rs::in_stream::InOctetStream;
use flood_rs::out_stream::OutOctetStream;
use flood_rs::{Deserialize, ReadOctetStream, Serialize};
use freelist_rs::FreeList;
use log::{debug, trace};
use secure_random::SecureRandom;
use std::collections::HashMap;
use std::io;
use std::io::ErrorKind;

pub trait DatagramHostEncoder {
    fn encode(&mut self, connection_id: u8, buf: &[u8]) -> io::Result<Vec<u8>>;
}

pub struct HostConnection {
    pub created_from_request: RequestId,
    pub connection_id: ConnectionId,
    pub seed: ConnectionSecretSeed,
    pub has_received_connect: bool,
}

pub struct ConnectionLayerHostCodec {
    pub connection_ids: FreeList<u8>,
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
            trace!(
                "host sending on connection {} size: {}",
                actual_connection.connection_id.value,
                buf.len()
            );
            write_to_stream(
                &mut stream,
                actual_connection.connection_id,
                actual_connection.seed,
                buf,
            )?;
        } else {
            debug!(
                "host sending connect response connection_id: {} for request: {}",
                actual_connection.connection_id.value, actual_connection.created_from_request
            );
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
                trace!(
                    "host received payload of size: {} from connection {}",
                    buf.len() - 5,
                    connection.connection_id.value
                );

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
                    debug!("host received connect request {connect_request:?}");
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