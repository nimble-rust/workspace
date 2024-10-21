/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
pub mod layer;
pub use app_version::{Version, VersionProvider};

use datagram::{DatagramCodec, DatagramCommunicator};
use flood_rs::{Deserialize, Serialize};
use hexify::format_hex;
use log::{error, info, warn};
use monotonic_time_rs::Millis;
pub use nimble_rust::*;

use secure_random::GetRandom;
use std::fmt::{Debug, Display};
use udp_client::UdpClient;

pub struct ClientWithCodec<
    StateT: GameCallbacks<StepT> + Debug,
    StepT: Clone + Deserialize + Serialize + Debug + Display + Eq,
> {
    pub client: Client<StateT, StepT>,
    pub communicator: Box<dyn DatagramCommunicator>,
    pub codec: Box<dyn DatagramCodec>,
}

impl<
        StateT: GameCallbacks<StepT> + Debug,
        StepT: Clone + Deserialize + Serialize + Debug + Display + Eq + PartialEq,
    > ClientWithCodec<StateT, StepT>
{
    pub fn new(url: &str) -> Self {
        let now = Millis::new(0);
        let client = Client::<StateT, StepT>::new(now);
        let udp_client = UdpClient::new(url).unwrap();
        let communicator: Box<dyn DatagramCommunicator> = Box::new(udp_client);
        let random2 = GetRandom;
        let random2_box = Box::new(random2);
        let datagram_connections_layer_client =
            datagram_connections::prelude::Client::new(random2_box);

        let datagram_connections_codec_box: Box<dyn DatagramCodec> =
            Box::new(datagram_connections_layer_client);
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
            codec: datagram_connections_codec_box,
        }
    }

    pub fn game(&self) -> Option<&StateT> {
        self.client.game()
    }

    pub fn update(&mut self, now: Millis) -> Result<(), ClientError> {
        let mut buf = [1u8; 1200];
        let datagrams_to_send = self.client.send(now)?;
        for datagram_to_send in datagrams_to_send {
            info!(
                "send nimble datagram of size: {} payload: {}",
                datagram_to_send.len(),
                format_hex(datagram_to_send.as_slice())
            );
            let processed = self
                .codec
                .encode(datagram_to_send.as_slice())
                .map_err(ClientError::IoError)?;
            self.communicator
                .send(processed.as_slice())
                .map_err(ClientError::IoError)?;
        }
        while let Ok(size) = self.communicator.receive(&mut buf) {
            if size == 0 {
                // No more data to process; exit the loop
                break;
            }
            let received_buf = &buf[0..size];
            info!(
                "received datagram of size: {} payload: {}",
                size,
                format_hex(received_buf)
            );
            match self.codec.decode(received_buf) {
                Ok(datagram_for_client) => {
                    if !datagram_for_client.is_empty() {
                        info!(
                            "received datagram to normal client: {}",
                            format_hex(&datagram_for_client)
                        );
                        if let Err(e) = self.client.receive(now, datagram_for_client.as_slice()) {
                            if e.error_level() == ErrorLevel::Info {
                                info!("received info {:?}", e);
                            } else {
                                warn!("receive error {:?}", e);
                            }
                        }
                    }
                }
                Err(some_error) => error!("error {}", some_error),
            }
        }
        Ok(())
    }
}
