/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use datagram::{DatagramCodec, DatagramCommunicator};
use flood_rs::{Deserialize, Serialize};
use hexify::format_hex;
use log::{error, info, warn};
use nimble_assent::prelude::*;
use nimble_client::client::ClientStream;
use nimble_protocol::client_to_host::AuthoritativeStep;
use nimble_protocol::Version;
use nimble_rectify::RectifyCallback;
use nimble_seer::prelude::*;
use secure_random::GetRandom;
use std::cell::RefCell;
use std::fmt::Debug;
use std::io;
use std::rc::Rc;
use udp_client::UdpClient;

pub struct ExampleClient<
    Game: SeerCallback<AuthoritativeStep<StepData>>
    + AssentCallback<AuthoritativeStep<StepData>>
    + RectifyCallback,
    StepData: Clone + Deserialize + Serialize + Debug + Eq + PartialEq,
> {
    pub client: ClientStream<Game, StepData>,
    pub communicator: Box<dyn DatagramCommunicator>,
    pub codec: Box<dyn DatagramCodec>,
}

//"127.0.0.1:23000"

impl<
    Game: SeerCallback<AuthoritativeStep<StepData>>
    + AssentCallback<AuthoritativeStep<StepData>>
    + RectifyCallback,
    StepData: Clone + Deserialize + Serialize + Debug + Eq + PartialEq,
> ExampleClient<Game, StepData>
{
    pub fn new(url: &str) -> Self {
        let random = GetRandom;
        let random_box = Rc::new(RefCell::new(random));
        let application_version = Version {
            major: 0,
            minor: 0,
            patch: 0,
        };
        let client = ClientStream::<Game, StepData>::new(random_box, &application_version);
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

    pub fn update(&mut self) -> io::Result<()> {
        let mut buf = [1u8; 1200];
        for _ in 0..20 {
            let datagrams_to_send = self.client.send()?;
            for datagram_to_send in datagrams_to_send {
                info!(
                    "send nimble datagram of size: {} payload: {}",
                    datagram_to_send.len(),
                    format_hex(datagram_to_send.as_slice())
                );
                let processed = self.codec.encode(datagram_to_send.as_slice())?;
                self.communicator.send(processed.as_slice())?;
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
