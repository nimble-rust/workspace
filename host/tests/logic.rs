/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use blob_stream::in_logic_front::FrontLogic;
use blob_stream::prelude::{ReceiverToSenderFrontCommands, SenderToReceiverFrontCommands};
use log::debug;
use nimble_host::logic::HostLogic;
use nimble_host::state::State;
use nimble_protocol::client_to_host::DownloadGameStateRequest;
use nimble_protocol::prelude::{ClientToHostCommands, HostToClientCommands};
use nimble_steps::GenericOctetStep;
use std::time::Instant;
use test_log::test;
use tick_id::TickId;

#[test]
fn game_state_download() {
    const TICK_ID: TickId = TickId(42);
    const EXPECTED_PAYLOAD: &[u8] = &[0xff, 0x33];
    let state = State::new(TICK_ID, EXPECTED_PAYLOAD);
    let mut logic = HostLogic::<GenericOctetStep>::new(state);

    let connection_id = logic.create_connection().expect("it should work");
    assert_eq!(connection_id.0, 0);
    let now = Instant::now();

    // Send a Download Game State request to the host.
    // This is usually done by the client, but we do it manually here.
    let download_request = DownloadGameStateRequest { request_id: 99 };
    let answers = logic
        .update(
            connection_id,
            now,
            &ClientToHostCommands::DownloadGameState(download_request.clone()),
        )
        .expect("Should download game state");

    debug!("{:?}", answers);

    assert_eq!(answers.len(), 2); // Download Game State Response and a Start Transfer

    debug!(
        "first answer (should be DownloadGameState response): {:?}",
        answers[0]
    );

    // Validate the DownloadGameState response
    let download_game_state_response = match &answers[0] {
        HostToClientCommands::DownloadGameState(response) => response,
        _ => panic!("Unexpected answer: expected DownloadGameState"),
    };
    assert_eq!(download_game_state_response.tick_id.0, TICK_ID.0);
    assert_eq!(
        download_game_state_response.client_request,
        download_request.request_id
    );
    assert_eq!(download_game_state_response.blob_stream_channel, 1);

    // Validate the StartTransfer response
    debug!(
        "second answer (should be StartTransfer response): {:?}",
        answers[1]
    );

    let start_transfer_data = match &answers[1] {
        HostToClientCommands::BlobStreamChannel(response) => match response {
            SenderToReceiverFrontCommands::StartTransfer(start_transfer_data) => {
                start_transfer_data
            }
            _ => panic!("Unexpected answer: expected SenderToReceiverFrontCommands"),
        },
        _ => panic!("Unexpected answer: expected BlobStreamChannel with Start Transfer"),
    };

    assert_eq!(start_transfer_data.transfer_id, 1);

    let mut in_stream = FrontLogic::new();

    // The client receives the Start Transfer from the host
    // and returns a ReceiverToSenderFrontCommands::AckStart.
    let probably_start_acks = in_stream
        .update(&SenderToReceiverFrontCommands::StartTransfer(
            start_transfer_data.clone(),
        ))
        .expect("Should start transfer");

    // The host receives the AckStart
    // and returns a number of BlobStreamChannel(SetChunk).
    let probably_set_chunks = logic
        .update(
            connection_id,
            now,
            &ClientToHostCommands::BlobStreamChannel(probably_start_acks),
        )
        .expect("Should download game state");

    // Extract SetChunk from BlobStreamChannel.
    let first_set_converted_chunks = probably_set_chunks
        .iter()
        .map(|x| match x {
            HostToClientCommands::BlobStreamChannel(sender_to_receiver) => match sender_to_receiver
            {
                SenderToReceiverFrontCommands::SetChunk(start_transfer_data) => start_transfer_data,
                _ => panic!(
                    "Unexpected sender to receiver {:?}",
                    &probably_set_chunks[0]
                ),
            },
            _ => panic!("Unexpected answer: expected BlobStreamChannel"),
        })
        .collect::<Vec<_>>();

    // Process SetChunks
    let last_ack = {
        let mut ack: Option<ReceiverToSenderFrontCommands> = None;

        for x in first_set_converted_chunks {
            debug!("should be SetChunkFrontData: {:?}", x);
            let resp = in_stream
                .update(&SenderToReceiverFrontCommands::SetChunk(x.clone()))
                .expect("should handle start transfer");
            ack = Some(resp);
        }
        ack
    };
    assert!(last_ack.is_some());

    // Ensure the in_stream ("client") has fully received the blob.
    // Verify that the host is aware the client has received the entire blob.
    assert_eq!(
        in_stream.blob().expect("blob should be ready here"),
        EXPECTED_PAYLOAD
    );

    logic
        .update(
            connection_id,
            now,
            &ClientToHostCommands::BlobStreamChannel(last_ack.unwrap()),
        )
        .expect("Should download game state");

    assert!(logic
        .get(connection_id)
        .as_ref()
        .expect("connection should exist")
        .is_state_received_by_remote());

    logic
        .destroy_connection(connection_id)
        .expect("Should destroy connection");
}
