/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use crate::err::{ClientError, ClientErrorKind, ErrorLevel};
use blob_stream::prelude::{FrontLogic, SenderToReceiverFrontCommands};
use flood_rs::{Deserialize, Serialize};
use log::info;
use nimble_assent::prelude::*;
use nimble_participant::ParticipantId;
use nimble_protocol::client_to_host::{
    AuthoritativeStep, DownloadGameStateRequest, PredictedStep, PredictedStepsForAllPlayers,
};
use nimble_protocol::host_to_client::DownloadGameStateResponse;
use nimble_protocol::prelude::*;
use nimble_rectify::prelude::*;
use nimble_seer::prelude::*;
use nimble_steps::Steps;
use secure_random::SecureRandom;
use std::collections::HashMap;
use std::fmt::Debug;
use tick_id::TickId;


#[derive(Debug)]
pub struct ClientLogic<
    Game: SeerCallback<AuthoritativeStep<StepT>>
    + AssentCallback<AuthoritativeStep<StepT>>
    + RectifyCallback,
    StepT: Clone + Deserialize + Serialize + Debug,
> {
    joining_player: Option<JoinGameRequest>,
    #[allow(unused)]
    random: Box<dyn SecureRandom>,
    tick_id: u32,
    debug_tick_id_to_send: u32,
    rectify: Rectify<Game, AuthoritativeStep<StepT>>,
    blob_stream_client: FrontLogic,
    commands_to_send: Vec<ClientToHostCommands<StepT>>,
    downloading_game_state_tick_id: TickId,
    download_state_request_id: Option<u8>,
    outgoing_predicted_steps: HashMap<u8, Steps<StepT>>,
    #[allow(unused)]
    last_download_state_request_id: u8,
}

impl<
    Game: SeerCallback<AuthoritativeStep<StepT>>
    + AssentCallback<AuthoritativeStep<StepT>>
    + RectifyCallback,
    StepT: Clone + Deserialize + Serialize + Debug,
> ClientLogic<Game, StepT>
{
    pub fn new(random: Box<dyn SecureRandom>) -> ClientLogic<Game, StepT> {
        Self {
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
            outgoing_predicted_steps: HashMap::new(),
        }
    }

    pub fn debug_rectify(&self) -> &Rectify<Game, AuthoritativeStep<StepT>> {
        &self.rectify
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

        if let Some(joining_game) = &self.joining_player {
            info!("connected. send join_game_request {:?}", joining_game);
            commands.push(ClientToHostCommands::JoinGameType(joining_game.clone()));
        }

        let mut predicted_steps_for_all: HashMap<u8, PredictedStepsForOnePlayer<StepT>> =
            HashMap::new();
        for (index, step_queue) in self.outgoing_predicted_steps.iter() {
            if step_queue.is_empty() {
                continue;
            }
            let x: PredictedStepsForOnePlayer<StepT> = PredictedStepsForOnePlayer {
                first_tick_id: step_queue.front_tick_id().unwrap(),
                predicted_steps: step_queue
                    .iter()
                    .map(|step_info| step_info.step.clone())
                    .collect(),
            };
            predicted_steps_for_all.insert(*index, x);
        }

        let steps_request = StepsRequest {
            ack: StepsAck {
                latest_received_step_tick_id: self.tick_id,
                lost_steps_mask_after_last_received: 0,
            },
            combined_predicted_steps: PredictedStepsForAllPlayers {
                predicted_players: predicted_steps_for_all,
            },
        };

        let steps_command = ClientToHostCommands::Steps(steps_request);
        commands.push(steps_command);

        if let Some(id) = self.download_state_request_id {
            let download_request = DownloadGameStateRequest { request_id: id };
            let cmd = ClientToHostCommands::DownloadGameState(download_request);
            commands.push(cmd);
        }

        commands
    }

    pub fn update(&mut self, game: &mut Game) {
        self.rectify.update(game)
    }

    pub fn add_predicted_step(&mut self, step: PredictedStep<StepT>) {
        let predicted_authenticated_combined_step: HashMap<_, _> = step
            .predicted_players
            .iter()
            .map(|(local_index, predict_step)| (ParticipantId(*local_index), predict_step.clone()))
            .collect();
        self.rectify.push_predicted(AuthoritativeStep {
            authoritative_participants: predicted_authenticated_combined_step,
        });

        for (index, step) in &step.predicted_players {
            if !self.outgoing_predicted_steps.contains_key(index) {
                self.outgoing_predicted_steps.insert(*index, Steps::new());
            }
            self.outgoing_predicted_steps
                .get_mut(index)
                .unwrap()
                .push(step.clone());
        }
    }

    fn on_join_game(&mut self, cmd: &JoinGameAccepted) -> Result<(), ClientErrorKind> {
        info!("join game accepted: {:?}", cmd);
        Ok(())
    }

    fn on_game_step(&mut self, cmd: &GameStepResponse<StepT>) -> Result<(), ClientErrorKind> {
        info!("game step response: {:?}", cmd);
        let mut current_authoritative_tick_id = cmd.authoritative_steps.start_tick_id;

        for authoritative_step_range in &cmd.authoritative_steps.ranges {
            current_authoritative_tick_id +=
                authoritative_step_range.delta_steps_from_previous as u32;
            let skip_count = self
                .rectify
                .waiting_for_authoritative_tick_id()
                .map_or(0, |tick_id| tick_id - current_authoritative_tick_id);

            if skip_count >= authoritative_step_range.step_count as i64 {
                continue;
            }

            if skip_count + authoritative_step_range.step_count as i64 <= 0 {
                continue;
            }

            let actual_skip_count = if skip_count < 0 {
                0
            } else {
                skip_count as usize
            };

            let remaining_step_count =
                authoritative_step_range.step_count as usize - actual_skip_count;

            let mut vector_for_range: Vec<HashMap<ParticipantId, StepT>> =
                Vec::with_capacity(remaining_step_count);
            for _ in 0..remaining_step_count {
                vector_for_range.push(HashMap::new());
            }

            for (participant_id, authoritative_steps_vector_for_each_participant) in
                &authoritative_step_range
                    .authoritative_steps
                    .authoritative_participants
            {
                for (index, authoritative_step_for_one_participant) in
                    authoritative_steps_vector_for_each_participant
                        .iter()
                        .skip(actual_skip_count)
                        .enumerate()
                {
                    let x = vector_for_range.get_mut(index).unwrap();
                    x.insert(
                        *participant_id,
                        authoritative_step_for_one_participant.clone(),
                    );
                }
            }
            for (index, combined_authoritative) in vector_for_range.iter().enumerate() {
                self.rectify
                    .push_authoritative_with_check(
                        current_authoritative_tick_id + actual_skip_count as u32 + index as u32,
                        AuthoritativeStep {
                            authoritative_participants: combined_authoritative.clone(),
                        },
                    )
                    .map_err(ClientErrorKind::Unexpected)?;
            }
        }
        Ok(())
    }

    fn on_download_state_response(
        &mut self,
        download_response: &DownloadGameStateResponse,
    ) -> Result<(), ClientErrorKind> {
        if self.download_state_request_id.is_none() {
            return Err(ClientErrorKind::DownloadResponseWasUnexpected);
        } else if download_response.client_request != self.download_state_request_id.unwrap() {
            return Err(ClientErrorKind::WrongDownloadRequestId);
        }
        self.downloading_game_state_tick_id = download_response.tick_id;
        Ok(())
    }

    fn on_blob_stream(
        &mut self,
        blob_stream_command: &SenderToReceiverFrontCommands,
    ) -> Result<(), ClientErrorKind> {
        let answer = self
            .blob_stream_client
            .update(blob_stream_command)
            .map_err(ClientErrorKind::IoErr)?;
        self.commands_to_send
            .push(ClientToHostCommands::BlobStreamChannel(answer));
        Ok(())
    }

    pub fn receive_cmd(
        &mut self,
        command: &HostToClientCommands<StepT>,
    ) -> Result<(), ClientErrorKind> {
        match command {
            HostToClientCommands::JoinGame(ref join_game_response) => {
                self.on_join_game(join_game_response)?
            }
            HostToClientCommands::GameStep(game_step_response) => {
                self.on_game_step(game_step_response)?
            }
            HostToClientCommands::DownloadGameState(download_response) => {
                self.on_download_state_response(download_response)?
            }
            HostToClientCommands::BlobStreamChannel(blob_stream_command) => {
                self.on_blob_stream(blob_stream_command)?
            }
        }
        Ok(())
    }

    pub fn receive(&mut self, commands: &[HostToClientCommands<StepT>]) -> Result<(), ClientError> {
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
