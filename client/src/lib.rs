/*----------------------------------------------------------------------------------------------------------
 *  Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------------------*/
use std::io;
use std::io::{Error, ErrorKind};

use flood_rs::{InOctetStream, OutOctetStream, WriteOctetStream};
use log::info;

use nimble_protocol::client_to_host::{ClientToHostCommands, ConnectRequest};
use nimble_protocol::host_to_client::{ConnectionAccepted, HostToClientCommands};
use nimble_protocol::{ConnectionId, Nonce, Version};
use secure_random::SecureRandom;

#[derive(PartialEq, Debug)]
enum ClientPhase {
    Connecting(Nonce),
    Connected(ConnectionId),
}

pub struct Client {
    phase: ClientPhase,
}

impl Client {
    pub fn new(mut random: Box<dyn SecureRandom>) -> Client {
        let phase = ClientPhase::Connecting(Nonce(random.get_random_u64()));
        Client { phase }
    }

    fn send_to_command(&self) -> ClientToHostCommands {
        match self.phase {
            ClientPhase::Connecting(nonce) => {
                let connect_cmd = ConnectRequest {
                    nimble_version: Version {
                        major: 0,
                        minor: 0,
                        patch: 4,
                    },
                    use_debug_stream: false,
                    application_version: Version {
                        major: 1,
                        minor: 0,
                        patch: 0,
                    },
                    nonce,
                };

                ClientToHostCommands::ConnectType(connect_cmd)
            }
            ClientPhase::Connected(connection_id) => {
                info!("connected. send steps {:?}", connection_id);
                ClientToHostCommands::Steps
            }
        }
    }

    fn send(&self) -> io::Result<Vec<Vec<u8>>> {
        let mut out_stream = OutOctetStream::new();
        let client_command = self.send_to_command();
        match client_command {
            ClientToHostCommands::ConnectType(cmd) => {
                let zero_connection_id = ConnectionId { value: 0 }; // oob
                zero_connection_id.to_stream(&mut out_stream).unwrap(); // OOB
            }
            _ => {}
        }
        client_command.to_stream(&mut out_stream).unwrap();

        let datagrams = vec![out_stream.data];
        Ok(datagrams)
    }

    fn on_connect(&mut self, cmd: ConnectionAccepted) -> io::Result<()> {
        match self.phase {
            ClientPhase::Connecting(nonce) => {
                if cmd.response_to_nonce != nonce {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "Wrong nonce when connecting {} vs {}",
                            cmd.response_to_nonce, nonce
                        ),
                    ));
                }
                self.phase = ClientPhase::Connected(cmd.host_assigned_connection_id);
                info!("connected!");
                Ok(())
            }
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "can not receive on_connect in current client state {:?}",
                    self.phase
                ),
            )),
        }
    }

    fn receive(&mut self, datagram: Vec<u8>) -> io::Result<()> {
        let mut in_stream = InOctetStream::new(datagram);
        let connection_id = ConnectionId::from_stream(&mut in_stream);
        let command = HostToClientCommands::from_stream(&mut in_stream)?;
        match command {
            HostToClientCommands::ConnectType(connect_command) => {
                self.on_connect(connect_command)?;
                return Ok(());
            }
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use log::{error, info, warn};
    use test_log::test;

    use datagram::{DatagramCommunicator, DatagramProcessor};
    use nimble_protocol::hex_output;
    use secure_random::GetRandom;
    use udp_client::UdpClient;

    use crate::Client;

    #[test]
    fn send_to_host() {
        let random = GetRandom {};
        let random_box = Box::new(random);
        let mut client = Client::new(random_box);
        let mut udp_client = UdpClient::new("127.0.0.1:23000").unwrap();
        let communicator: &mut dyn DatagramCommunicator = &mut udp_client;
        let random2 = GetRandom {};
        let random2_box = Box::new(random2);
        let mut udp_connections_client = udp_connections::Client::new(random2_box);

        let processor: &mut dyn DatagramProcessor = &mut udp_connections_client;
        let mut buf = [1u8; 1200];
        for _ in 0..10 {
            let datagrams_to_send = client.send().unwrap();
            for datagram_to_send in datagrams_to_send {
                let processed = processor
                    .send_datagram(datagram_to_send.as_slice())
                    .unwrap();
                communicator.send_datagram(processed.as_slice()).unwrap();
            }
            if let Ok(size) = communicator.receive_datagram(&mut buf) {
                let received_buf = &buf[0..size];
                info!(
                    "received datagram of size: {} {}",
                    size,
                    hex_output(received_buf)
                );
                match processor.receive_datagram(received_buf) {
                    Ok(datagram_for_client) => {
                        if datagram_for_client.len() > 0 {
                            info!(
                                "received datagram to client: {}",
                                hex_output(&datagram_for_client)
                            );
                            if let Err(e) = client.receive(datagram_for_client) {
                                warn!("receive error {}", e);
                            }
                        }
                    }
                    Err(some_error) => error!("error {}", some_error),
                }
            }
            thread::sleep(Duration::from_millis(16));
        }
    }
}
