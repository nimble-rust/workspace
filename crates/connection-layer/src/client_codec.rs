use crate::client_to_host::{ClientToHostCommands, ConnectRequest};
use crate::host_to_client::HostToClientCommands;
use crate::{verify_hash, write_to_stream, ConnectionId, ConnectionSecretSeed, RequestId, Version};
use datagram::{DatagramDecoder, DatagramEncoder};
use flood_rs::in_stream::InOctetStream;
use flood_rs::out_stream::OutOctetStream;
use flood_rs::{Deserialize, ReadOctetStream, Serialize};
use log::{debug, trace};
use std::io;

pub struct ConnectionInfo {
    pub connection_id: ConnectionId,
    pub seed: ConnectionSecretSeed,
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
                debug!("client sending connect request {connect_request:?}");
                ClientToHostCommands::Connect(connect_request).serialize(&mut stream)?;
                trace!("send request {}", hexify::format_hex(stream.octets_ref()));
            }
            Some(connection_info) => {
                trace!(
                    "client sending payload connection_id: {} size: {}",
                    connection_info.connection_id.value,
                    buf.len()
                );

                write_to_stream(
                    &mut stream,
                    connection_info.connection_id,
                    connection_info.seed,
                    buf,
                )?
            }
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
                        debug!("client received connect response {connect_response:?}");
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
                    debug!(
                        "client received payload size:{} connection:{}",
                        buf.len() - 5,
                        connection_id.value
                    );
                    Ok(buf[5..].to_vec())
                }
            }
        }
    }
}
