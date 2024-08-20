/*----------------------------------------------------------------------------------------------------------
 *  Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------------------*/
use std::io;
use std::io::{Error, ErrorKind};

use flood_rs::{InOctetStream, OutOctetStream, WriteOctetStream};
use log::info;

use connection_layer::{ConnectionId, ConnectionSecretSeed, prepare_out_stream, write_to_stream};
use datagram_pinger::{client_out_ping, ClientTime};
use nimble_protocol::{Nonce, Version};
use nimble_protocol::client_to_host::{ClientToHostCommands, PredictedStepsForPlayers, ConnectRequest, JoinGameRequest, JoinGameType, JoinPlayerRequest, JoinPlayerRequests, StepsAck, StepsRequest, PredictedStepsForPlayer};
use nimble_protocol::host_to_client::{ConnectionAccepted, GameStepResponse, HostToClientCommands, JoinGameAccepted};
use ordered_datagram::OrderedOut;
use secure_random::SecureRandom;

#[derive(PartialEq, Debug)]
enum ClientPhase {
    Connecting(Nonce),
    Connected(ConnectionId, ConnectionSecretSeed),
}

pub struct Client {
    phase: ClientPhase,
    joining_player: Option<JoinGameRequest>,
    random: Box<dyn SecureRandom>,
    ordered_datagram_out: OrderedOut,
    tick_id: u32,
    debug_tick_id_to_send: u32,
}


impl Client {
    pub fn new(mut random: Box<dyn SecureRandom>) -> Client {
        let phase = ClientPhase::Connecting(Nonce(random.get_random_u64()));
        Client {
            phase,
            random,
            joining_player: None,
            ordered_datagram_out: OrderedOut::default(),
            tick_id: 0,
            debug_tick_id_to_send: 0,
        }
    }

    pub fn set_joining_player(&mut self, join_game_request: JoinGameRequest) {
        self.joining_player = Some(join_game_request);
    }

    pub fn debug_set_tick_id(&mut self, tick_id: u32) {
        self.tick_id = tick_id;
        self.debug_tick_id_to_send = self.tick_id;
    }

    // At this client level, it should not mutate when sending a command
    // mutation should happen when receiving commands.
    fn send_to_command(&mut self) -> Vec<ClientToHostCommands> {
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

                self.debug_tick_id_to_send += 1;


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

    fn write_header(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        match self.phase {
            ClientPhase::Connected(assigned_connection_id, _) => {
                prepare_out_stream(stream)?; // Add hash stream
                self.ordered_datagram_out.to_stream(stream)?;
                info!("add connect header {}", self.ordered_datagram_out.sequence_to_send);

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

    fn write_to_start_of_header(&self, connection_id: ConnectionId, seed: ConnectionSecretSeed, out_stream: &mut OutOctetStream) -> io::Result<()> {
        let mut payload = out_stream.data.as_mut_slice();
        let mut hash_stream = OutOctetStream::new();
        let payload_to_calculate_on = &payload[6..];
        info!("payload: {:?}", payload_to_calculate_on);
        write_to_stream(&mut hash_stream, connection_id, seed, payload_to_calculate_on)?; // Write hash connection layer header
        payload[..hash_stream.data.len()].copy_from_slice(hash_stream.data.as_slice());
        Ok(())
    }


    fn send(&mut self) -> io::Result<Vec<Vec<u8>>> {
        let mut out_stream = OutOctetStream::new();
        self.write_header(&mut out_stream)?;

        let client_commands_to_send = self.send_to_command();
        for command_to_send in client_commands_to_send.iter() {
            info!("sending command {}", command_to_send);
            command_to_send.to_stream(&mut out_stream)?;
        }

        match self.phase {
            ClientPhase::Connected(connection_id, seed) => {
                info!("writing connected header");
                self.write_to_start_of_header(connection_id, seed, &mut out_stream)?
            }
            _ => {}
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
                self.phase = ClientPhase::Connected(cmd.host_assigned_connection_id, ConnectionSecretSeed(half_secret));

                ClientToHostCommands::JoinGameType(JoinGameRequest {
                    nonce: Nonce(self.random.get_random_u64()),
                    join_game_type: JoinGameType::NoSecret,
                    player_requests: JoinPlayerRequests {
                        players: vec![
                            JoinPlayerRequest {
                                local_index: 42,
                            }
                        ]
                    },
                });
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

    fn receive(&mut self, datagram: &[u8]) -> io::Result<()> {
        let mut in_stream = InOctetStream::new(datagram.to_vec());
        let _connection_id = ConnectionId::from_stream(&mut in_stream);
        let command = HostToClientCommands::from_stream(&mut in_stream)?;
        match command {
            HostToClientCommands::ConnectType(connect_command) => {
                self.on_connect(connect_command)?;
                Ok(())
            }
            HostToClientCommands::JoinGame(join_game_response) => {
                self.on_join_game(join_game_response)?;
                Ok(())
            }
            HostToClientCommands::GameStep(game_step_response) => {
                self.on_game_step(game_step_response)?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use log::{error, info, warn};
    use test_log::test;

    use datagram::{DatagramCommunicator, DatagramProcessor};
    use nimble_protocol::{hex_output, Nonce};
    use nimble_protocol::client_to_host::{JoinGameRequest, JoinGameType, JoinPlayerRequest, JoinPlayerRequests};
    use secure_random::GetRandom;
    use udp_client::UdpClient;

    use crate::Client;

    #[test]
    fn send_to_host() {
        let random = GetRandom {};
        let random_box = Box::new(random);
        let mut client = Client::new(random_box);
        let mut udp_client = UdpClient::new("127.0.0.1:23000").unwrap();
        let communicator: &mut dyn DatagramCommunicator = &mut udp_client;
        let random2 = GetRandom {};
        let random2_box = Box::new(random2);
        let mut udp_connections_client = udp_connections::Client::new(random2_box);

        let processor: &mut dyn DatagramProcessor = &mut udp_connections_client;
        let joining_player = JoinPlayerRequest {
            local_index: 32,
        };

        let join_game_request = JoinGameRequest {
            nonce: Nonce(0),
            join_game_type: JoinGameType::NoSecret,
            player_requests: JoinPlayerRequests { players: vec![joining_player] },
        };
        client.set_joining_player(join_game_request);
        client.debug_set_tick_id(0x8BADF00D);

        let mut buf = [1u8; 1200];
        for _ in 0..20 {
            let datagrams_to_send = client.send().unwrap();
            for datagram_to_send in datagrams_to_send {
                info!("send nimble datagram of size: {} payload: {}", datagram_to_send.len(), hex_output(datagram_to_send.as_slice()));
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
}
