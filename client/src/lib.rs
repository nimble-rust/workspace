/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use std::io;
use std::io::{Error, ErrorKind};

use flood_rs::{InOctetStream, OutOctetStream, ReadOctetStream, WriteOctetStream};
use log::info;

use connection_layer::{
    prepare_out_stream, verify_hash, write_to_stream, ConnectionId, ConnectionLayerMode,
    ConnectionSecretSeed,
};
use datagram_pinger::{client_in_ping, client_out_ping, ClientTime};
use nimble_assent::AssentCallback;

use nimble_protocol::client_to_host::{
    ClientToHostCommands, ConnectRequest, JoinGameRequest, PredictedStepsForPlayer,
    PredictedStepsForPlayers, StepsAck, StepsRequest,
};
use nimble_protocol::host_to_client::{
    ConnectionAccepted, GameStepResponse, HostToClientCommands, JoinGameAccepted,
};
use nimble_protocol::{Nonce, Version};
use nimble_rectify::{Rectify, RectifyCallback};
use nimble_seer::SeerCallback;
use ordered_datagram::{OrderedIn, OrderedOut};
use secure_random::SecureRandom;

#[derive(PartialEq, Debug)]
enum ClientPhase {
    Connecting(Nonce),
    Connected(ConnectionId, ConnectionSecretSeed),
}

pub struct Client<
    Game: SeerCallback<StepT> + AssentCallback<StepT> + RectifyCallback,
    StepT: Clone + nimble_steps::Deserialize,
> {
    phase: ClientPhase,
    joining_player: Option<JoinGameRequest>,
    #[allow(unused)]
    random: Box<dyn SecureRandom>,
    ordered_datagram_out: OrderedOut,
    ordered_datagram_in: OrderedIn,
    tick_id: u32,
    debug_tick_id_to_send: u32,
    rectify: Rectify<Game, StepT>,
}

impl<
        Game: SeerCallback<StepT> + AssentCallback<StepT> + RectifyCallback,
        StepT: Clone + nimble_steps::Deserialize,
    > Client<Game, StepT>
{
    pub fn new(mut random: Box<dyn SecureRandom>) -> Client<Game, StepT> {
        let phase = ClientPhase::Connecting(Nonce(random.get_random_u64()));
        Self {
            phase,
            random,
            joining_player: None,
            ordered_datagram_out: OrderedOut::default(),
            ordered_datagram_in: OrderedIn::default(),
            tick_id: 0,
            debug_tick_id_to_send: 0,
            rectify: Rectify::new(),
        }
    }

    pub fn set_joining_player(&mut self, join_game_request: JoinGameRequest) {
        self.joining_player = Some(join_game_request);
    }

    pub fn debug_set_tick_id(&mut self, tick_id: u32) {
        self.tick_id = tick_id;
        self.debug_tick_id_to_send = self.tick_id;
    }

    fn send_to_command(&self) -> Vec<ClientToHostCommands> {
        let mut commands: Vec<ClientToHostCommands> = vec![];

        match self.phase {
            ClientPhase::Connecting(nonce) => {
                let connect_cmd = ConnectRequest {
                    nimble_version: Version {
                        major: 0,
                        minor: 0,
                        patch: 5,
                    },
                    use_debug_stream: false,
                    application_version: Version {
                        major: 1,
                        minor: 0,
                        patch: 0,
                    },
                    nonce,
                };

                commands.push(ClientToHostCommands::ConnectType(connect_cmd))
            }
            ClientPhase::Connected(connection_id, _) => {
                if let Some(joining_game) = &self.joining_player {
                    info!("connected. send join_game_request {:?}", connection_id);
                    commands.push(ClientToHostCommands::JoinGameType(joining_game.clone()));
                }

                let payload = vec![0xfau8, 64];

                let predicted_steps_for_one_player = PredictedStepsForPlayer {
                    participant_party_index: 0,
                    first_step_id: self.debug_tick_id_to_send,
                    serialized_predicted_steps: vec![payload],
                };

                //self.debug_tick_id_to_send += 1;

                let steps_request = StepsRequest {
                    ack: StepsAck {
                        latest_received_step_tick_id: self.tick_id,
                        lost_steps_mask_after_last_received: 0,
                    },
                    combined_predicted_steps: PredictedStepsForPlayers {
                        predicted_steps_for_players: vec![predicted_steps_for_one_player],
                    },
                };

                let steps_command = ClientToHostCommands::Steps(steps_request);

                commands.push(steps_command);
            }
        };

        commands
    }

    pub fn update(&mut self, game: &mut Game) {
        self.rectify.update(game)
    }

    pub fn add_predicted_step(&mut self, step: StepT) {
        self.rectify.push_predicted(step);
    }

    fn write_header(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        match self.phase {
            ClientPhase::Connected(_, _) => {
                prepare_out_stream(stream)?; // Add hash stream
                self.ordered_datagram_out.to_stream(stream)?;
                info!(
                    "add connect header {}",
                    self.ordered_datagram_out.sequence_to_send
                );

                let client_time = ClientTime::new(0);
                client_out_ping(client_time, stream)
            }
            _ => {
                info!("oob zero connection");
                let zero_connection_id = ConnectionId { value: 0 }; // oob
                zero_connection_id.to_stream(stream) // OOB
            }
        }
    }

    fn write_to_start_of_header(
        &self,
        connection_id: ConnectionId,
        seed: ConnectionSecretSeed,
        out_stream: &mut OutOctetStream,
    ) -> io::Result<()> {
        let payload = out_stream.data.as_mut_slice();
        let mut hash_stream = OutOctetStream::new();
        let payload_to_calculate_on = &payload[5..];
        info!("payload: {:?}", payload_to_calculate_on);
        write_to_stream(
            &mut hash_stream,
            connection_id,
            seed,
            payload_to_calculate_on,
        )?; // Write hash connection layer header
        payload[..hash_stream.data.len()].copy_from_slice(hash_stream.data.as_slice());
        Ok(())
    }

    pub fn send(&mut self) -> io::Result<Vec<Vec<u8>>> {
        let mut out_stream = OutOctetStream::new();
        self.write_header(&mut out_stream)?;

        let client_commands_to_send = self.send_to_command();
        for command_to_send in client_commands_to_send.iter() {
            info!("sending command {}", command_to_send);
            command_to_send.to_stream(&mut out_stream)?;
        }

        if let ClientPhase::Connected(connection_id, seed) = self.phase {
            info!("writing connected header");
            self.write_to_start_of_header(connection_id, seed, &mut out_stream)?
        }

        let datagrams = vec![out_stream.data];
        Ok(datagrams)
    }

    fn on_join_game(&mut self, cmd: JoinGameAccepted) -> io::Result<()> {
        info!("join game accepted: {:?}", cmd);
        Ok(())
    }

    fn on_game_step(&mut self, cmd: GameStepResponse) -> io::Result<()> {
        info!("game step response: {:?}", cmd);
        Ok(())
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
                let half_secret = cmd.host_assigned_connection_secret.value as u32;
                info!("half_secret: {:X}", half_secret);
                self.phase = ClientPhase::Connected(
                    cmd.host_assigned_connection_id,
                    ConnectionSecretSeed(half_secret),
                );

                /*
                ClientToHostCommands::JoinGameType(JoinGameRequest {
                    nonce: Nonce(self.random.get_random_u64()),
                    join_game_type: JoinGameType::NoSecret,
                    player_requests: JoinPlayerRequests {
                        players: vec![JoinPlayerRequest { local_index: 42 }],
                    },
                });

                 */
                info!("connected! cmd:{:?}", cmd);
                Ok(())
            }
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "can not receive on_connect in current client state '{:?}'",
                    self.phase
                ),
            )),
        }
    }

    pub fn receive(&mut self, datagram: &[u8]) -> io::Result<()> {
        let mut in_stream = InOctetStream::new(datagram.to_vec());
        let connection_mode = ConnectionLayerMode::from_stream(&mut in_stream)?;
        match connection_mode {
            ConnectionLayerMode::OOB => {
                let command = HostToClientCommands::from_stream(&mut in_stream)?;
                match command {
                    HostToClientCommands::ConnectType(connect_command) => {
                        self.on_connect(connect_command)?;
                        Ok(())
                    }
                    _ => Err(Error::new(ErrorKind::InvalidData, "unknown OOB command")),
                }
            }
            ConnectionLayerMode::Connection(connection_layer) => {
                match self.phase {
                    ClientPhase::Connecting(_) => Err(Error::new(
                        ErrorKind::InvalidData,
                        "received connection layer connection mode without a connection",
                    )),
                    ClientPhase::Connected(connection_id, connection_seed) => {
                        if connection_layer.connection_id != connection_id {
                            return Err(Error::new(
                                ErrorKind::InvalidData,
                                format!(
                                    "wrong connection id, expected {:?} but received {:?}",
                                    connection_id, connection_layer.connection_id
                                ),
                            ));
                        }

                        verify_hash(
                            connection_layer.murmur3_hash,
                            connection_seed,
                            &datagram[5..],
                        )?;

                        self.ordered_datagram_in.read_and_verify(&mut in_stream)?;
                        let _ = client_in_ping(&mut in_stream)?;

                        // TODO: Add latency calculations

                        for _ in 0..8 {
                            // only allowed to have at maximum eight commands in one datagram
                            if in_stream.has_reached_end() {
                                break;
                            }
                            let command = HostToClientCommands::from_stream(&mut in_stream)?;
                            match command {
                                HostToClientCommands::JoinGame(join_game_response) => {
                                    self.on_join_game(join_game_response)?;
                                }
                                HostToClientCommands::GameStep(game_step_response) => {
                                    self.on_game_step(game_step_response)?;
                                }
                                _ => return Err(Error::new(
                                    ErrorKind::InvalidData,
                                    "unknown command for host to layer connected client command",
                                )),
                            }
                        }

                        Ok(())
                    }
                }
            }
        }
    }
}
