/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use datagram::DatagramCommunicator;
use log::{error, info, warn};
use nimble_assent::AssentCallback;
use nimble_client::Client;
use nimble_protocol::hex_output;
use nimble_seer::SeerCallback;
use nimble_steps::Deserialize;
use secure_random::GetRandom;
use std::io;
use udp_client::UdpClient;
use udp_connections::DatagramProcessor;

pub struct ExampleClient<
    Game: SeerCallback<StepData> + AssentCallback<StepData> + nimble_rectify::RectifyCallback,
    StepData: Clone + Deserialize,
> {
    pub client: Client<Game, StepData>,
    pub communicator: Box<dyn DatagramCommunicator>,
    pub processor: Box<dyn DatagramProcessor>,
}

//"127.0.0.1:23000"

impl<
        Game: SeerCallback<StepData> + AssentCallback<StepData> + nimble_rectify::RectifyCallback,
        StepData: Clone + Deserialize,
    > ExampleClient<Game, StepData>
{
    pub fn new(url: &str) -> Self {
        let random = GetRandom {};
        let random_box = Box::new(random);
        let client = Client::<Game, StepData>::new(random_box);
        let udp_client = UdpClient::new(url).unwrap();
        let communicator: Box<dyn DatagramCommunicator> = Box::new(udp_client);
        let random2 = GetRandom {};
        let random2_box = Box::new(random2);
        let udp_connections_client = udp_connections::Client::new(random2_box);

        let processor: Box<dyn DatagramProcessor> = Box::new(udp_connections_client);
        //let joining_player = JoinPlayerRequest { local_index: 32 };
        /*
                let join_game_request = JoinGameRequest {
                    nonce: Nonce(0),
                    join_game_type: JoinGameType::NoSecret,
                    player_requests: JoinPlayerRequests {
                        players: vec![joining_player],
                    },
                };
        */
        // client.set_joining_player(join_game_request);
        // client.debug_set_tick_id(0x8BADF00D);
        // thread::sleep(Duration::from_millis(16));

        Self {
            client,
            communicator,
            processor,
        }
    }

    pub fn update(&mut self) -> io::Result<()> {
        let mut buf = [1u8; 1200];
        for _ in 0..20 {
            let datagrams_to_send = self.client.send()?;
            for datagram_to_send in datagrams_to_send {
                info!(
                    "send nimble datagram of size: {} payload: {}",
                    datagram_to_send.len(),
                    hex_output(datagram_to_send.as_slice())
                );
                let processed = self.processor.send_datagram(datagram_to_send.as_slice())?;
                self.communicator.send_datagram(processed.as_slice())?;
            }
            if let Ok(size) = self.communicator.receive_datagram(&mut buf) {
                let received_buf = &buf[0..size];
                info!(
                    "received datagram of size: {} payload: {}",
                    size,
                    hex_output(received_buf)
                );
                match self.processor.receive_datagram(received_buf) {
                    Ok(datagram_for_client) => {
                        if !datagram_for_client.is_empty() {
                            info!(
                                "received datagram to client: {}",
                                hex_output(&datagram_for_client)
                            );
                            if let Err(e) = self.client.receive(datagram_for_client.as_slice()) {
                                warn!("receive error {}", e);
                            }
                        }
                    }
                    Err(some_error) => error!("error {}", some_error),
                }
            }
        }
        Ok(())
    }
}
