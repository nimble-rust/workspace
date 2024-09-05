/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use nimble_protocol::client_to_host::DownloadGameStateRequest;
use nimble_protocol::host_to_client::{DownloadGameStateResponse, TickId};

#[derive(Debug, Clone)]
pub struct State {
    pub data: Vec<u8>,
    pub tick_id: TickId,
}
impl State {
    pub fn new(tick_id: TickId, state: &[u8]) -> Self {
        Self {
            data: state.to_vec(),
            tick_id,
        }
    }
}

pub struct HostStateConnection {
    pub client_request: Option<u8>,
    pub assigned_blob_stream_channel: Option<u16>,
    pub last_blob_stream_channel: u16,
    pub blob_stream_channel: Option<u8>,
}

pub struct HostState {
    pub state: State,
}

impl HostState {
    pub fn new(state: State) -> Self {
        Self { state }
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }

    pub fn request(
        &self,
        connection: &mut HostStateConnection,
        request: DownloadGameStateRequest,
    ) -> DownloadGameStateResponse {
        let was_same = connection
            .client_request
            .map_or(false, |found| found == request.request_id);
        if !was_same {
            connection.client_request = Some(request.request_id);
            connection.assigned_blob_stream_channel = Some(connection.last_blob_stream_channel);
            connection.last_blob_stream_channel += 1;
        }
        DownloadGameStateResponse {
            client_request: connection
                .client_request
                .expect("client_request should always be set at this point"),
            tick_id: self.state.tick_id,
            blob_stream_channel: connection
                .assigned_blob_stream_channel
                .expect("assigned_blob_stream_channel should always be set at this point"),
        }
    }
}
