/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use log::debug;
use nimble_host::logic::HostLogic;
use nimble_protocol::client_to_host::DownloadGameStateRequest;
use nimble_protocol::prelude::ClientToHostCommands;
use nimble_steps::GenericOctetStep;

#[test_log::test]
fn test_logic() {
    let mut logic = HostLogic::<GenericOctetStep>::new();

    let download_request = DownloadGameStateRequest { request_id: 42 };
    let answers = logic
        .update(ClientToHostCommands::DownloadGameState(download_request))
        .expect("Should download game state");
    for answer in answers {
        debug!("answer: {:?}", answer);
    }
}
