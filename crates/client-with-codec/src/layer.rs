/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use datagram::{DatagramCodec, DatagramCommunicator};
use flood_rs::{Deserialize, Serialize};
use hexify::format_hex;
use log::{error, info, warn};
use monotonic_time_rs::Millis;
use nimble_rust::{Client, ClientError, GameCallbacks};
use secure_random::GetRandom;
use std::fmt::{Debug, Display};
use udp_client::UdpClient;

pub struct ExampleClientWithLayer<
    GameT: GameCallbacks<StepT> + Debug,
    StepT: Clone + Deserialize + Serialize + Debug + Display,
> {
    pub client: Client<GameT, StepT>,
    pub communicator: Box<dyn DatagramCommunicator>,
    pub codec: Box<dyn DatagramCodec>,
    pub connection_layer_codec: Box<dyn DatagramCodec>,
}

impl<
        GameT: GameCallbacks<StepT> + Debug,
        StepT: Clone + Deserialize + Serialize + Debug + Display + Eq,
    > ExampleClientWithLayer<GameT, StepT>
{
    pub fn new(url: &str) -> Self {
        let now = Millis::new(0);
        let client = Client::<GameT, StepT>::new(now);
        let udp_client = UdpClient::new(url).unwrap();
        let communicator: Box<dyn DatagramCommunicator> = Box::new(udp_client);
        let random2 = GetRandom;
        let random2_box = Box::new(random2);
        let datagram_connections_layer = datagram_connections::prelude::Client::new(random2_box);

        let connection_layer = connection_layer::prelude::ConnectionLayerClientCodec::new(0);
        let connection_layer_codec: Box<dyn DatagramCodec> = Box::new(connection_layer);

        let datagram_connections_codec_box: Box<dyn DatagramCodec> =
            Box::new(datagram_connections_layer);
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
            connection_layer_codec,
        }
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
            let processed_with_layer = self
                .connection_layer_codec
                .encode(&datagram_to_send)
                .map_err(ClientError::IoError)?;
            let processed_with_udp_connections = self
                .codec
                .encode(processed_with_layer.as_slice())
                .map_err(ClientError::IoError)?;
            self.communicator
                .send(processed_with_udp_connections.as_slice())
                .map_err(ClientError::IoError)?;
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
                        let decoded_layer = &*self
                            .connection_layer_codec
                            .decode(&datagram_for_client)
                            .map_err(ClientError::IoError)?;
                        if let Err(e) = self.client.receive(now, decoded_layer) {
                            warn!("receive error {:?}", e);
                        }
                    }
                }
                Err(some_error) => error!("error {}", some_error),
            }
        }
        Ok(())
    }
}
