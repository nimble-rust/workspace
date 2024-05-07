/*----------------------------------------------------------------------------------------------------------
 *  Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/client
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------------------*/

use flood_rs::OutOctetStream;

use nimble_protocol::{ClientSendCommands, ConnectCommand, ConnectionId, Nonce, Version};
use secure_random::get_random_u64;

pub struct ClientDatagram {
    pub payload: Vec<u8>,
}

pub struct Client {}

impl Client {
    pub fn new() -> Client {
        Client {}
    }

    fn send(&self) -> Vec<ClientDatagram> {
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
            nonce: Nonce::new(get_random_u64()),
        };

        let client_command = ClientSendCommands::ConnectType(connect_cmd);

        let mut out_stream = OutOctetStream::new();
        let zero_connection_id = ConnectionId {
            value: 0
        };
        zero_connection_id.to_stream(&mut out_stream).unwrap();
        client_command.to_stream(&mut out_stream).unwrap();

        let datagrams = vec![
            ClientDatagram { payload: out_stream.data },
        ];
        datagrams
    }
}


#[cfg(test)]
mod tests {
    use datagram::DatagramSender;
    use udp_client::UdpClient;

    use crate::Client;

    #[test]
    fn it_works() {
        let client = Client::new();
        let udp_client = UdpClient::new("127.0.0.1:23000").unwrap();
        let udp_client_box = Box::new(udp_client);
        let udp_connections_client = udp_connections::Client::new(udp_client_box);
        let sender: &dyn DatagramSender = &udp_connections_client;
        for _ in 0..100 {
            let datagrams = client.send();
            for datagram in datagrams {
                sender.send_datagram(datagram.payload.as_slice()).unwrap();
            }
        }
    }
}
