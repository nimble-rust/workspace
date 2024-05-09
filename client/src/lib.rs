/*----------------------------------------------------------------------------------------------------------
 *  Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/client
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------------------*/

use std::io;
use std::io::{Error, ErrorKind};

use flood_rs::{InOctetStream, OutOctetStream};
use log::info;

use nimble_protocol::{
    ClientToHostCommands, ConnectCommand, ConnectionId, HostToClientCommands,
    HostToClientConnectCommand, Nonce, Version,
};
use secure_random::SecureRandom;

use crate::ClientPhase::Challenge;

#[derive(PartialEq)]
enum ClientPhase {
    Challenge(Nonce),
    Connecting(Nonce),
    Connected(ConnectionId),
}

pub struct Client {
    phase: ClientPhase,
    random: Box<dyn SecureRandom>,
}

impl Client {
    pub fn new(mut random: Box<dyn SecureRandom>) -> Client {
        let phase = ClientPhase::Challenge(Nonce(random.get_random_u64()));
        Client { random, phase }
    }

    fn send(&self) -> Vec<Vec<u8>> {
        let connect_cmd = ConnectCommand {
            nimble_version: Version {
                major: 1,
                minor: 2,
                patch: 3,
            },
            use_debug_stream: false,
            application_version: Version {
                major: 1,
                minor: 2,
                patch: 3,
            },
            nonce: Nonce::new(0),
        };

        let client_command = ClientToHostCommands::ConnectType(connect_cmd);

        let mut out_stream = OutOctetStream::new();
        let zero_connection_id = ConnectionId { value: 0 };
        zero_connection_id.to_stream(&mut out_stream).unwrap();
        client_command.to_stream(&mut out_stream).unwrap();

        let datagrams = vec![out_stream.data];
        datagrams
    }

    fn on_connect(&mut self, cmd: HostToClientConnectCommand) -> io::Result<()> {
        match self.phase {
            ClientPhase::Connecting(nonce) => {
                if cmd.nonce != nonce {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "Wrong nonce when connecting",
                    ));
                }
                self.phase = ClientPhase::Connected(cmd.connection_id);
                info!("connected!");
                Ok(())
            }
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                "can not receive on_connect in current client state",
            )),
        }
    }

    fn receive(&mut self, datagram: Vec<u8>) -> io::Result<()> {
        let mut in_stream = InOctetStream::new(datagram);
        let command = HostToClientCommands::from_stream(&mut in_stream)?;
        match command {
            HostToClientCommands::ConnectType(connect_command) => {
                self.on_connect(connect_command)?;
                return Ok(());
            }
            _ => todo!(),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use log::{info, warn};
    use test_log::test;

    use datagram::{DatagramCommunicator, DatagramProcessor};
    use secure_random::GetRandom;
    use udp_client::UdpClient;

    use crate::Client;

    #[test]
    fn send_to_host() {
        let random = GetRandom {};
        let random_box = Box::new(random);
        let mut client = Client::new(random_box);
        let mut udp_client = UdpClient::new("127.0.0.1:23000").unwrap();
        let mut communicator: &mut dyn DatagramCommunicator = &mut udp_client;
        let random2 = GetRandom {};
        let random2_box = Box::new(random2);
        let mut udp_connections_client = udp_connections::Client::new(random2_box);

        let mut processor: &mut dyn DatagramProcessor = &mut udp_connections_client;
        let mut buf = [1u8; 1200];
        for _ in 0..10 {
            let datagrams_to_send = client.send();
            for datagram_to_send in datagrams_to_send {
                let processed = processor
                    .send_datagram(datagram_to_send.as_slice())
                    .unwrap();
                communicator.send_datagram(processed.as_slice()).unwrap();
            }
            if let Ok(size) = communicator.receive_datagram(&mut buf) {
                let received_buf = &buf[0..size];
                info!("received datagram of size: {}", size);
                println!("received datagram of size2: {} {:?}", size, received_buf);
                match processor.receive_datagram(received_buf) {
                    Ok(datagram_for_client) => {
                        if datagram_for_client.len() > 0 {
                            println!("received datagram to client: {:?}", datagram_for_client);
                            //client.receive(datagram_fo_client)?,
                        }
                    }
                    Err(some_error) => println!("error {}", some_error),
                }
                //client.receive(buf.as_slice()[0..size].to_vec()).unwrap();
            }
            thread::sleep(Duration::from_millis(16));
        }
    }
}
