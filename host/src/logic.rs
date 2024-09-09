/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use crate::state::State;
use log::{debug, info, trace};
use nimble_protocol::client_to_host::{
    ClientToHostCommands, DownloadGameStateRequest, StepsRequest,
};
use nimble_protocol::host_to_client::{
    AuthoritativeStepRanges, DownloadGameStateResponse, GameStepResponseHeader,
    HostToClientCommands, JoinGameAccepted, JoinGameParticipants, PartyAndSessionSecret,
};
use nimble_protocol::prelude::{GameStepResponse, JoinGameRequest};

use crate::combinator::Combinator;
use blob_stream::out_logic_front::OutLogicFront;
use blob_stream::prelude::TransferId;
use freelist::FreeList;
use nimble_participant::ParticipantId;
use nimble_protocol::SessionConnectionSecret;
use nimble_steps::GenericOctetStep;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

#[derive(Copy, Clone, Debug)]
pub struct ConnectionId(pub u8);

#[derive(Copy, Clone, Debug)]
pub struct Participant {
    pub id: ParticipantId,
    pub client_local_index: u8,
}

pub struct GameSession {
    pub state: State,
    pub participants: HashMap<ParticipantId, Rc<RefCell<Participant>>>,
    pub participant_ids: FreeList,
}

impl GameSession {
    pub fn new(state: State) -> Self {
        Self {
            state,
            participants: HashMap::new(),
            participant_ids: FreeList::new(0xff),
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub(crate) fn create_participant(
        &mut self,
        client_local_index: u8,
    ) -> Option<Rc<RefCell<Participant>>> {
        let participant_id_value = self.participant_ids.allocate();
        if let Some(id_value) = participant_id_value {
            let participant_id = ParticipantId(id_value);
            let participant = Rc::new(RefCell::new(Participant {
                client_local_index,
                id: participant_id,
            }));

            self.participants
                .insert(participant_id, participant.clone());
            Some(participant)
        } else {
            None
        }
    }
}

#[derive(Debug)]
#[allow(clippy::new_without_default)]
pub struct Connection {
    pub participant_lookup: HashMap<u8, Rc<RefCell<Participant>>>,
    pub out_blob_stream: Option<OutLogicFront>,
    last_transfer_id: u16,
    debug_counter: u16,
}

#[allow(clippy::new_without_default)]
impl Connection {
    pub fn new() -> Self {
        Self {
            participant_lookup: Default::default(),
            out_blob_stream: None,
            last_transfer_id: 0,
            debug_counter: 0,
        }
    }
}

#[derive(Debug)]
pub enum HostLogicError {
    UnknownConnectionId(ConnectionId),
    FreeListError {
        connection_id: ConnectionId,
        message: String,
    },
    UnknownPartyMemberIndex(u8),
    NoFreeParticipantIds,
}

pub struct HostLogic<StepT> {
    #[allow(unused)]
    combinator: Combinator<StepT>,
    connections: HashMap<u8, Connection>,
    session: GameSession,
    free_list: freelist::FreeList,
}

impl<StepT> HostLogic<StepT> {
    pub fn new(last_known_state: State) -> Self {
        Self {
            combinator: Combinator::<StepT>::new(last_known_state.tick_id),
            connections: HashMap::new(),
            session: GameSession::new(last_known_state),
            free_list: FreeList::new(0xff),
        }
    }

    pub fn create_connection(&mut self) -> Option<ConnectionId> {
        let new_connection_id = self.free_list.allocate();
        if let Some(id) = new_connection_id {
            self.connections.insert(id, Connection::new());
            Some(ConnectionId(id))
        } else {
            None
        }
    }

    pub fn destroy_connection(
        &mut self,
        connection_id: ConnectionId,
    ) -> Result<(), HostLogicError> {
        self.free_list
            .free(connection_id.0)
            .map_err(|err| HostLogicError::FreeListError {
                connection_id,
                message: err,
            })?;

        if self.connections.remove(&connection_id.0).is_some() {
            Ok(())
        } else {
            Err(HostLogicError::UnknownConnectionId(connection_id))
        }
    }

    fn on_join(
        &mut self,
        connection_id: ConnectionId,
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

        let participant = self
            .session
            .create_participant(request.player_requests.players[0].local_index)
            .ok_or(HostLogicError::NoFreeParticipantIds)?;
        let connection = self
            .connections
            .get_mut(&connection_id.0)
            .ok_or(HostLogicError::UnknownConnectionId(connection_id))?;
        connection
            .participant_lookup
            .insert(request.player_requests.players[0].local_index, participant);
        Ok(HostToClientCommands::JoinGame(join_accepted))
    }

    fn on_steps(
        &mut self,
        connection_id: ConnectionId,
        request: &StepsRequest,
    ) -> Result<HostToClientCommands, HostLogicError> {
        trace!("on_step {:?}", request);
        /*
               for participant in request.combined_predicted_steps.predicted_steps_for_players {
                   self.combinator.receive_step(participant.participant_party_index)

               }

        */

        let connection = self
            .connections
            .get_mut(&connection_id.0)
            .ok_or(HostLogicError::UnknownConnectionId(connection_id))?;

        for predicted_step_for_player in request
            .combined_predicted_steps
            .predicted_steps_for_players
            .iter()
        {
            if let Some(participant) = connection
                .participant_lookup
                .get(&predicted_step_for_player.participant_party_index)
            {
                for serialized_predicted_step_for_participant in
                    &predicted_step_for_player.serialized_predicted_steps
                {
                    let _ = GenericOctetStep {
                        payload: serialized_predicted_step_for_participant.to_vec(),
                    };
                    connection.debug_counter += participant.borrow().client_local_index as u16;
                    info!("connection: {connection:?}");
                }
            } else {
                return Err(HostLogicError::UnknownPartyMemberIndex(
                    predicted_step_for_player.participant_party_index,
                ));
            }
        }

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
        connection_id: ConnectionId,
        request: &DownloadGameStateRequest,
    ) -> Result<HostToClientCommands, HostLogicError> {
        debug!("client requested download {:?}", request);
        let state = self.session.state();
        let connection = self
            .connections
            .get_mut(&connection_id.0)
            .ok_or(HostLogicError::UnknownConnectionId(connection_id))?;

        const FIXED_CHUNK_SIZE: usize = 1024;
        const RESEND_DURATION: Duration = Duration::from_millis(32 * 3);

        connection.last_transfer_id += 1;
        let transfer_id = TransferId(connection.last_transfer_id);
        connection.out_blob_stream = Some(OutLogicFront::new(
            transfer_id,
            FIXED_CHUNK_SIZE,
            RESEND_DURATION,
            self.session.state().data.as_slice(),
        ));

        let response = DownloadGameStateResponse {
            client_request: request.request_id,
            tick_id: nimble_protocol::host_to_client::TickId(state.tick_id.0),
            blob_stream_channel: transfer_id.0,
        };
        Ok(HostToClientCommands::DownloadGameState(response))
    }

    pub fn update(
        &mut self,
        connection_id: ConnectionId,
        request: ClientToHostCommands,
    ) -> Result<Vec<HostToClientCommands>, HostLogicError> {
        match request {
            ClientToHostCommands::JoinGameType(join_game_request) => {
                Ok(vec![self.on_join(connection_id, &join_game_request)?])
            }
            ClientToHostCommands::Steps(add_steps_request) => {
                Ok(vec![self.on_steps(connection_id, &add_steps_request)?])
            }
            ClientToHostCommands::DownloadGameState(download_game_state_request) => {
                Ok(vec![self.on_download(
                    connection_id,
                    &download_game_state_request,
                )?])
            }
        }
    }
}
