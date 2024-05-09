/*----------------------------------------------------------------------------------------------------------
 *  Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/client
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------------------*/

use std::io;
use flood_rs::{InOctetStream, OutOctetStream};

use nimble_protocol::{ClientToHostCommands, ConnectCommand, ConnectionId, Nonce, Version};


pub struct Client {}

impl Client {
    pub fn new() -> Client {
        Client {}
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

        let datagrams = vec![
            out_stream.data,
        ];
        datagrams
    }

    fn receive(&self, datagram: Vec<u8>) -> io::Result<()> {
        let in_stream = InOctetStream::new(datagram);


        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use datagram::{DatagramCommunicator, DatagramSender};
    use udp_client::UdpClient;

    use crate::Client;

    #[test]
    fn send_to_host() {
        let client = Client::new();
        let udp_client = UdpClient::new("127.0.0.1:23000").unwrap();
        let udp_client_box = Box::new(udp_client);
        let mut udp_connections_client = udp_connections::Client::new(udp_client_box);
        let mut communicator: &mut dyn DatagramCommunicator = &mut udp_connections_client;
        for _ in 0..100 {
            let datagrams = client.send();
            for datagram in datagrams {
                communicator
                    .send_datagram(datagram.as_slice())
                    .unwrap();
            }
        }
    }
}
