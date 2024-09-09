/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use nimble_host::logic::HostLogic;
use nimble_host::state::State;
use nimble_protocol::client_to_host::DownloadGameStateRequest;
use nimble_protocol::prelude::{ClientToHostCommands, HostToClientCommands};
use nimble_steps::GenericOctetStep;
use tick_id::TickId;

#[test_log::test]
fn test_host_logic() {
    const TICK_ID: TickId = TickId(42);
    let state = State::new(TICK_ID, &[0xff, 0x33]);
    let mut logic = HostLogic::<GenericOctetStep>::new(state);

    let connection_id = logic.create_connection().expect("it should work");
    assert_eq!(connection_id.0, 0);

    let download_request = DownloadGameStateRequest { request_id: 99 };
    let answers = logic
        .update(
            connection_id,
            ClientToHostCommands::DownloadGameState(download_request.clone()),
        )
        .expect("Should download game state");
    assert_eq!(answers.len(), 1);

    if let HostToClientCommands::DownloadGameState(download_game_state_response) = &answers[0] {
        assert_eq!(download_game_state_response.tick_id.0, TICK_ID.0);
        assert_eq!(
            download_game_state_response.client_request,
            download_request.request_id
        );
        assert_eq!(download_game_state_response.blob_stream_channel, 1);
    } else {
        panic!("wrong answer");
    }

    logic
        .destroy_connection(connection_id)
        .expect("Should destroy connection");
}
