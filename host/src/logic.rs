/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use crate::combine::HostCombinator;
use log::{debug, trace};
use nimble_protocol::client_to_host::{
    ClientToHostCommands, DownloadGameStateRequest, StepsRequest,
};
use nimble_protocol::host_to_client::{
    AuthoritativeStepRanges, DownloadGameStateResponse, GameStepResponseHeader,
    HostToClientCommands, JoinGameAccepted, JoinGameParticipants, PartyAndSessionSecret, TickId,
};
use nimble_protocol::prelude::{GameStepResponse, JoinGameRequest};
use nimble_protocol::SessionConnectionSecret;

#[derive(Clone, Debug)]
pub enum HostLogicError {}

pub struct HostLogic<StepT> {
    #[allow(unused)]
    combinator: HostCombinator<StepT>,
}

impl<StepT> Default for HostLogic<StepT> {
    fn default() -> Self {
        Self::new()
    }
}

impl<StepT> HostLogic<StepT> {
    pub fn new() -> Self {
        Self {
            combinator: HostCombinator::new(),
        }
    }

    fn on_join(
        &mut self,
        request: &JoinGameRequest,
    ) -> Result<HostToClientCommands, HostLogicError> {
        debug!("on_join {:?}", request);

        let join_accepted = JoinGameAccepted {
            nonce: request.nonce,
            party_and_session_secret: PartyAndSessionSecret {
                session_secret: SessionConnectionSecret { value: 0 },
                party_id: 0,
            },
            participants: JoinGameParticipants(vec![]),
        };
        Ok(HostToClientCommands::JoinGame(join_accepted))
    }

    fn on_steps(&mut self, request: &StepsRequest) -> Result<HostToClientCommands, HostLogicError> {
        trace!("on_step {:?}", request);
        /*
               for participant in request.combined_predicted_steps.predicted_steps_for_players {
                   self.combinator.receive_step(participant.participant_party_index)

               }

        */
        let game_step_response = GameStepResponse {
            response_header: GameStepResponseHeader {
                connection_buffer_count: 0,
                delta_buffer: 0,
                last_step_received_from_client: 0,
            },
            authoritative_ranges: AuthoritativeStepRanges {
                start_step_id: 0,
                ranges: vec![],
            },
            payload: vec![],
        };
        Ok(HostToClientCommands::GameStep(game_step_response))
    }

    fn on_download(
        &mut self,
        request: &DownloadGameStateRequest,
    ) -> Result<HostToClientCommands, HostLogicError> {
        debug!("on_download {:?}", request);
        let response = DownloadGameStateResponse {
            client_request: request.request_id,
            tick_id: TickId(0),
            blob_stream_channel: 0,
        };
        Ok(HostToClientCommands::DownloadGameState(response))
    }

    pub fn update(
        &mut self,
        request: ClientToHostCommands,
    ) -> Result<Vec<HostToClientCommands>, HostLogicError> {
        match request {
            ClientToHostCommands::JoinGameType(join_game_request) => {
                Ok(vec![self.on_join(&join_game_request)?])
            }
            ClientToHostCommands::Steps(add_steps_request) => {
                Ok(vec![self.on_steps(&add_steps_request)?])
            }
            ClientToHostCommands::DownloadGameState(download_game_state_request) => {
                Ok(vec![self.on_download(&download_game_state_request)?])
            }
        }
    }
}
