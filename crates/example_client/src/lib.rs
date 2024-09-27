/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
pub mod layer;

use datagram::{DatagramCodec, DatagramCommunicator};
use err_rs::{ErrorLevel, ErrorLevelProvider};
use flood_rs::{BufferDeserializer, Deserialize, Serialize};
use hexify::format_hex;
use log::{error, info, warn};
use nimble_rust::client::{ClientStream, ClientStreamError};
use nimble_rust::Version;
use secure_random::GetRandom;
use std::fmt::Debug;
use udp_client::UdpClient;

pub struct ExampleClient<
    StateT: Debug + BufferDeserializer,
    StepData: Clone + Deserialize + Serialize + Debug + Eq + PartialEq,
> {
    pub client: ClientStream<StateT, StepData>,
    pub communicator: Box<dyn DatagramCommunicator>,
    pub codec: Box<dyn DatagramCodec>,
}

//"127.0.0.1:23000"

impl<
        StateT: Debug + BufferDeserializer,
        StepT: Clone + Deserialize + Serialize + Debug + Eq + PartialEq,
    > ExampleClient<StateT, StepT>
{
    pub fn new(url: &str) -> Self {
        let application_version = Version {
            major: 1,
            minor: 0,
            patch: 0,
        };
        let client = ClientStream::<StateT, StepT>::new(&application_version);
        let udp_client = UdpClient::new(url).unwrap();
        let communicator: Box<dyn DatagramCommunicator> = Box::new(udp_client);
        let random2 = GetRandom;
        let random2_box = Box::new(random2);
        let udp_connections_client = udp_connections::Client::new(random2_box);

        let processor: Box<dyn DatagramCodec> = Box::new(udp_connections_client);
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
            codec: processor,
        }
    }

    pub fn state(&self) -> Option<&StateT> {
        self.client.state()
    }

    pub fn update(&mut self) -> Result<(), ClientStreamError> {
        let mut buf = [1u8; 1200];
        let datagrams_to_send = self.client.send()?;
        for datagram_to_send in datagrams_to_send {
            info!(
                "send nimble datagram of size: {} payload: {}",
                datagram_to_send.len(),
                format_hex(datagram_to_send.as_slice())
            );
            let processed = self
                .codec
                .encode(datagram_to_send.as_slice())
                .map_err(ClientStreamError::IoErr)?;
            self.communicator
                .send(processed.as_slice())
                .map_err(ClientStreamError::IoErr)?;
        }
        if let Ok(size) = self.communicator.receive(&mut buf) {
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
                            "received datagram to client: {}",
                            format_hex(&datagram_for_client)
                        );
                        if let Err(e) = self.client.receive(datagram_for_client.as_slice()) {
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
