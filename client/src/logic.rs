/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use blob_stream::prelude::FrontLogic;
use flood_rs::{Deserialize, InOctetStream, OutOctetStream, Serialize};
use log::info;
use nimble_assent::prelude::*;
use nimble_protocol::client_to_host::{AuthoritativeCombinedStepForAllParticipants, DownloadGameStateRequest, PredictedStepsForAllPlayers};
use nimble_protocol::host_to_client::TickId;
use nimble_protocol::prelude::*;
use nimble_protocol::Nonce;
use nimble_rectify::prelude::*;
use nimble_seer::prelude::*;
use secure_random::SecureRandom;
use std::collections::HashMap;
use std::fmt::Debug;
use std::{fmt, io};

#[derive(Eq, Debug, PartialEq)]
pub enum ErrorLevel {
    Info,     // Informative, can be ignored
    Warning,  // Should be logged, but recoverable
    Critical, // Requires immediate attention, unrecoverable
}

#[derive(Debug)]
pub enum ClientError {
    Single(ClientErrorKind),
    Multiple(Vec<ClientErrorKind>),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Single(error) => error.fmt(f),
            Self::Multiple(errors) => {
                writeln!(f, "Multiple errors occurred:")?;

                for (index, error) in errors.iter().enumerate() {
                    writeln!(f, "{}: {}", index + 1, error)?;
                }

                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub enum ClientErrorKind {
    Unexpected,
    IoErr(io::Error),
    WrongConnectResponseNonce(Nonce),
    WrongDownloadRequestId,
    DownloadResponseWasUnexpected,
}

impl ClientErrorKind {
    pub fn error_level(&self) -> ErrorLevel {
        match self {
            Self::IoErr(_) => ErrorLevel::Critical,
            Self::WrongConnectResponseNonce(_) => ErrorLevel::Info,
            Self::WrongDownloadRequestId => ErrorLevel::Warning,
            Self::DownloadResponseWasUnexpected => ErrorLevel::Info,
            Self::Unexpected => ErrorLevel::Critical,
        }
    }
}

impl fmt::Display for ClientErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unexpected => {
                write!(f, "Unexpected")
            }
            Self::IoErr(io_err) => {
                write!(f, "io:err {:?}", io_err)
            }
            Self::WrongConnectResponseNonce(nonce) => {
                write!(f, "wrong nonce in reply to connect {:?}", nonce)
            }
            Self::WrongDownloadRequestId => {
                write!(f, "WrongDownloadRequestId")
            }
            Self::DownloadResponseWasUnexpected => {
                write!(f, "DownloadResponseWasUnexpected")
            }
        }
    }
}

impl std::error::Error for ClientErrorKind {} // it implements Debug and Display
impl std::error::Error for ClientError {} // it implements Debug and Display

#[derive(PartialEq, Debug)]
enum Phase {
    InGame,
}

pub struct ClientLogic<
    Game: SeerCallback<StepT> + AssentCallback<StepT> + RectifyCallback,
    StepT: Clone + Deserialize + Serialize + Debug + Eq + PartialEq,
> {
    phase: Phase,
    joining_player: Option<JoinGameRequest>,
    #[allow(unused)]
    random: Box<dyn SecureRandom>,
    tick_id: u32,
    debug_tick_id_to_send: u32,
    rectify: Rectify<Game, AuthoritativeCombinedStepForAllParticipants<StepT>>,
    blob_stream_client: FrontLogic,
    commands_to_send: Vec<ClientToHostCommands<StepT>>,
    downloading_game_state_tick_id: TickId,
    download_state_request_id: Option<u8>,
    #[allow(unused)]
    last_download_state_request_id: u8,
}

impl<
    Game: SeerCallback<StepT> + AssentCallback<StepT> + RectifyCallback,
    StepT: Clone + Deserialize + Serialize + Debug + Eq + PartialEq,
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
            blob_stream_client: FrontLogic::new(),
            commands_to_send: Vec::new(),
            download_state_request_id: None,
            last_download_state_request_id: 0,
            downloading_game_state_tick_id: TickId(0),
        }
    }

    pub fn set_joining_player(&mut self, join_game_request: JoinGameRequest) {
        self.joining_player = Some(join_game_request);
    }

    pub fn debug_set_tick_id(&mut self, tick_id: u32) {
        self.tick_id = tick_id;
        self.debug_tick_id_to_send = self.tick_id;
    }

    #[allow(unused)]
    fn request_game_state(&mut self) {
        self.last_download_state_request_id += 1;
        self.download_state_request_id = Some(self.last_download_state_request_id);
    }

    pub fn send(&mut self) -> Vec<ClientToHostCommands<StepT>> {
        let mut commands: Vec<ClientToHostCommands<StepT>> = self.commands_to_send.clone();
        self.commands_to_send.clear();

        match self.phase {
            Phase::InGame => {
                if let Some(joining_game) = &self.joining_player {
                    info!("connected. send join_game_request {:?}", joining_game);
                    commands.push(ClientToHostCommands::JoinGameType(joining_game.clone()));
                }


                let predicted_steps = self.rectify.seer().predicted_steps();

                let predicted_steps_for_one_player = PredictedStepsForOnePlayer {
                    local_index: 0,
                    first_tick_id: TickId(self.debug_tick_id_to_send),
                    predicted_steps: vec![],
                };

                //self.debug_tick_id_to_send += 1;

                let mut all = HashMap::new();

                all.insert(0, predicted_steps_for_one_player);

                let steps_request = StepsRequest {
                    ack: StepsAck {
                        latest_received_step_tick_id: self.tick_id,
                        lost_steps_mask_after_last_received: 0,
                    },
                    combined_predicted_steps: PredictedStepsForAllPlayers {
                        predicted_players: all,
                    },
                };

                let steps_command = ClientToHostCommands::Steps(steps_request);
                commands.push(steps_command);

                if let Some(id) = self.download_state_request_id {
                    let download_request = DownloadGameStateRequest { request_id: id };
                    let cmd = ClientToHostCommands::DownloadGameState(download_request);
                    commands.push(cmd);
                }
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

    fn on_join_game(&mut self, cmd: &JoinGameAccepted) -> Result<(), ClientErrorKind> {
        info!("join game accepted: {:?}", cmd);
        Ok(())
    }

    fn on_game_step(&mut self, cmd: &GameStepResponse<StepT>) -> Result<(), ClientErrorKind> {
        info!("game step response: {:?}", cmd);
        for authoritative_step_range in &cmd.authoritative_steps.ranges {
            for authoritative_step in &authoritative_step_range.authoritative_steps {
                self.rectify.push_authoritative(authoritative_step);
            }
        }
        Ok(())
    }

    fn receive_cmd(&mut self, command: &HostToClientCommands<StepT>) -> Result<(), ClientErrorKind> {
        match command {
            HostToClientCommands::JoinGame(ref join_game_response) => {
                self.on_join_game(join_game_response)?
            }
            HostToClientCommands::GameStep(game_step_response) => {
                self.on_game_step(game_step_response)?
            }
            HostToClientCommands::DownloadGameState(download_response) => {
                if self.download_state_request_id.is_none() {
                    return Err(ClientErrorKind::DownloadResponseWasUnexpected);
                } else if download_response.client_request
                    != self.download_state_request_id.unwrap()
                {
                    return Err(ClientErrorKind::WrongDownloadRequestId);
                }
                self.downloading_game_state_tick_id = download_response.tick_id;
            }
            HostToClientCommands::BlobStreamChannel(blob_stream_command) => {
                let answer = self
                    .blob_stream_client
                    .update(blob_stream_command)
                    .map_err(ClientErrorKind::IoErr)?;
                self.commands_to_send
                    .push(ClientToHostCommands::BlobStreamChannel(answer));
            }
        }
        Ok(())
    }

    pub fn receive(&mut self, commands: &[HostToClientCommands]) -> Result<(), ClientError> {
        let mut client_errors: Vec<ClientErrorKind> = Vec::new();

        for command in commands {
            if let Err(err) = self.receive_cmd(command) {
                if err.error_level() == ErrorLevel::Critical {
                    return Err(ClientError::Single(err));
                }
                client_errors.push(err);
            }
        }

        match client_errors.len() {
            0 => Ok(()),
            1 => Err(ClientError::Single(client_errors.pop().unwrap())),
            _ => Err(ClientError::Multiple(client_errors)),
        }
    }
}
