/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use flood_rs::{InOctetStream, ReadOctetStream};
use std::time::Duration;
use std::{io, thread};
use udp_connections::DatagramProcessor;

use crate::types::{ExampleGame, ExampleStep};
use datagram::DatagramCommunicator;
use log::{error, info, warn};
use nimble_assent::AssentCallback;
use nimble_client::Client;
use nimble_protocol::client_to_host::{
    JoinGameRequest, JoinGameType, JoinPlayerRequest, JoinPlayerRequests,
};
use nimble_protocol::{hex_output, Nonce};
use nimble_rectify::RectifyCallback;
use nimble_seer::SeerCallback;
use nimble_steps::Deserialize;
use secure_random::GetRandom;
//use test_log::test;
use udp_client::UdpClient;

mod types;

//#[test]
#[allow(dead_code)]
fn send_to_host() {
    let random = GetRandom {};
    let random_box = Box::new(random);
    let mut client = Client::<ExampleGame, ExampleStep>::new(random_box);
    let mut udp_client = UdpClient::new("127.0.0.1:23000").unwrap();
    let communicator: &mut dyn DatagramCommunicator = &mut udp_client;
    let random2 = GetRandom {};
    let random2_box = Box::new(random2);
    let mut udp_connections_client = udp_connections::Client::new(random2_box);

    let processor: &mut dyn DatagramProcessor = &mut udp_connections_client;
    let joining_player = JoinPlayerRequest { local_index: 32 };

    let join_game_request = JoinGameRequest {
        nonce: Nonce(0),
        join_game_type: JoinGameType::NoSecret,
        player_requests: JoinPlayerRequests {
            players: vec![joining_player],
        },
    };
    client.set_joining_player(join_game_request);
    client.debug_set_tick_id(0x8BADF00D);

    let mut buf = [1u8; 1200];
    for _ in 0..20 {
        let datagrams_to_send = client.send().unwrap();
        for datagram_to_send in datagrams_to_send {
            info!(
                "send nimble datagram of size: {} payload: {}",
                datagram_to_send.len(),
                hex_output(datagram_to_send.as_slice())
            );
            let processed = processor
                .send_datagram(datagram_to_send.as_slice())
                .unwrap();
            communicator.send_datagram(processed.as_slice()).unwrap();
        }
        if let Ok(size) = communicator.receive_datagram(&mut buf) {
            let received_buf = &buf[0..size];
            info!(
                "received datagram of size: {} payload: {}",
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
                        if let Err(e) = client.receive(datagram_for_client.as_slice()) {
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
