/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use flood_rs::prelude::*;
use nimble_participant::ParticipantId;
use nimble_protocol::client_to_host::AuthoritativeStepRangeForAllParticipants;
use nimble_protocol::client_to_host_oob::ConnectRequest;
use nimble_protocol::host_to_client::{AuthoritativeStepRange, AuthoritativeStepRanges};
use nimble_protocol::{ClientRequestId, Version};
use nimble_sample_step::SampleStep;
use std::collections::HashMap;
use std::io;
use tick_id::TickId;

#[test_log::test]
fn check_version() {
    let mut out_stream = OutOctetStream::new();
    let version = Version {
        major: 4,
        minor: 3,
        patch: 2,
    };
    version.to_stream(&mut out_stream).unwrap()
}

#[test_log::test]
fn check_connect() {
    let mut out_stream = OutOctetStream::new();
    let version = Version {
        major: 4,
        minor: 3,
        patch: 2,
    };
    let nimble_version = Version {
        major: 99,
        minor: 66,
        patch: 33,
    };
    let connect = ConnectRequest {
        nimble_version,
        use_debug_stream: false,
        application_version: version,
        client_request_id: ClientRequestId(0xff4411ff),
    };
    connect.to_stream(&mut out_stream).unwrap();

    let mut in_stream = InOctetStream::new(out_stream.octets_ref());

    let received_connect = ConnectRequest::from_stream(&mut in_stream).unwrap();

    assert_eq!(received_connect, connect);
}

#[test_log::test]
fn check_authoritative() -> io::Result<()> {
    // Prepare all steps
    let mut range_for_all_participants = HashMap::<ParticipantId, Vec<SampleStep>>::new();

    const PARTICIPANT_COUNT: usize = 2;
    let first_steps = vec![
        SampleStep::Jump,
        SampleStep::MoveLeft(-10),
        SampleStep::MoveRight(32000),
    ];
    let first_participant_id = ParticipantId(255);
    range_for_all_participants.insert(first_participant_id, first_steps.clone());

    let second_steps = vec![SampleStep::MoveLeft(40), SampleStep::Jump, SampleStep::Jump];
    let second_participant_id = ParticipantId(1);
    range_for_all_participants.insert(second_participant_id, second_steps.clone());

    let range_to_send = AuthoritativeStepRange::<SampleStep> {
        delta_steps_from_previous: 0,
        step_count: first_steps.len() as u8,
        authoritative_steps: AuthoritativeStepRangeForAllParticipants {
            authoritative_participants: range_for_all_participants,
        },
    };

    const EXPECTED_TICK_ID: TickId = TickId(909);
    let ranges_to_send = AuthoritativeStepRanges {
        start_tick_id: EXPECTED_TICK_ID,
        ranges: vec![range_to_send],
    };

    // Write the ranges to stream
    let mut out_stream = OutOctetStream::new();

    ranges_to_send.to_stream(&mut out_stream)?;

    // Read back the stream
    let mut in_stream = OctetRefReader::new(out_stream.octets_ref());
    let received_ranges = AuthoritativeStepRanges::<SampleStep>::from_stream(&mut in_stream)?;

    // Verify the deserialized data
    assert_eq!(received_ranges.ranges.len(), ranges_to_send.ranges.len());
    assert_eq!(received_ranges.start_tick_id, EXPECTED_TICK_ID);

    let first_and_only_range = &received_ranges.ranges[0];
    assert_eq!(first_and_only_range.delta_steps_from_previous, 0);
    assert_eq!(first_and_only_range.step_count, first_steps.len() as u8);

    let hash_map = &first_and_only_range
        .authoritative_steps
        .authoritative_participants;

    assert_eq!(hash_map.len(), PARTICIPANT_COUNT);

    let first_participant_steps_in_range = &hash_map[&first_participant_id];
    assert_eq!(first_participant_steps_in_range.len(), first_steps.len());
    assert_eq!(*first_participant_steps_in_range, first_steps);

    let second_participant_steps_in_range = &hash_map[&second_participant_id];
    assert_eq!(second_participant_steps_in_range.len(), second_steps.len());
    assert_eq!(*second_participant_steps_in_range, second_steps);

    Ok(())
}
