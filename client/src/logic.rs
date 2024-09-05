/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use log::info;
use nimble_assent::prelude::*;
use nimble_protocol::prelude::*;
use nimble_protocol::Nonce;
use nimble_rectify::prelude::*;
use nimble_seer::prelude::*;
use secure_random::SecureRandom;
use std::{fmt, io};

#[derive(Debug)]
pub enum ClientError {
    Unexpected,
    IoErr(io::Error),
    WrongConnectResponseNonce(Nonce),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unexpected => {
                write!(f, "Unexpected")
            }
            ClientError::IoErr(io_err) => {
                write!(f, "io:err {:?}", io_err)
            }
            ClientError::WrongConnectResponseNonce(nonce) => {
                write!(f, "wrong nonce in reply to connect {:?}", nonce)
            }
        }
    }
}

impl std::error::Error for ClientError {} // it implements Debug and Display


#[derive(PartialEq, Debug)]
enum Phase {
    InGame,
}

pub struct ClientLogic<
    Game: SeerCallback<StepT> + AssentCallback<StepT> + RectifyCallback,
    StepT: Clone + nimble_steps::Deserialize,
> {
    phase: Phase,
    joining_player: Option<JoinGameRequest>,
    #[allow(unused)]
    random: Box<dyn SecureRandom>,
    tick_id: u32,
    debug_tick_id_to_send: u32,
    rectify: Rectify<Game, StepT>,
}

impl<
    Game: SeerCallback<StepT> + AssentCallback<StepT> + RectifyCallback,
    StepT: Clone + nimble_steps::Deserialize,
> ClientLogic<Game, StepT>
{
    pub fn new(random: Box<dyn SecureRandom>) -> ClientLogic<Game, StepT> {
        let phase = Phase::InGame;
        Self {
            phase,
            random,
            joining_player: None,
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

    pub fn send(&self) -> Vec<ClientToHostCommands> {
        let mut commands: Vec<ClientToHostCommands> = vec![];

        match self.phase {
            Phase::InGame => {
                if let Some(joining_game) = &self.joining_player {
                    info!("connected. send join_game_request {:?}", joining_game);
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

    fn on_join_game(&mut self, cmd: &JoinGameAccepted) -> Result<(), ClientError> {
        info!("join game accepted: {:?}", cmd);
        Ok(())
    }

    fn on_game_step(
        &mut self,
        cmd: &GameStepResponse,
    ) -> Result<(), ClientError> {
        info!("game step response: {:?}", cmd);
        for authoritative_step_range in &cmd.authoritative_ranges.ranges {
            for authoritative_step in &authoritative_step_range.authoritative_steps {
                let auth_step = StepT::deserialize(authoritative_step.as_slice()).map_err(|err| ClientError::IoErr(err))?;
                self.rectify.push_authoritative(auth_step);
            }
        }
        Ok(())
    }

    pub fn receive(&mut self, commands: &[HostToClientCommands]) -> Result<(), ClientError> {
        for command in commands {
            match command {
                HostToClientCommands::JoinGame(ref join_game_response) => {
                    self.on_join_game(join_game_response)?
                }
                HostToClientCommands::GameStep(game_step_response) => {
                    self.on_game_step(game_step_response)?
                }
                HostToClientCommands::DownloadGameState(_) => {}
                HostToClientCommands::BlobStreamChannel(_) => {}
            }
        }
        Ok(())
    }
}
