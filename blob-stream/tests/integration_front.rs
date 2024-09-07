/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use crate::helper::generate_deterministic_blob_array;
use blob_stream::out_logic_front::OutLogicFront;
use blob_stream::prelude::TransferId;
use log::trace;
use rand::prelude::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::time::{Duration, Instant};
pub mod helper;

#[test_log::test]
fn test_blob_stream_front() {
    let seed = 12345678;
    let blob_to_transfer = generate_deterministic_blob_array(OCTET_COUNT, seed);
    let mut drop_rng = StdRng::seed_from_u64(seed);
    const CHUNK_SIZE: usize = 4;
    const CHUNK_COUNT: usize = 30;
    const OCTET_COUNT: usize = (CHUNK_SIZE * (CHUNK_COUNT - 1)) + 1;
    const ITERATION_COUNT: usize = 5;

    let mut in_logic = blob_stream::in_logic_front::FrontLogic::new();
    let mut out_logic = OutLogicFront::new(
        TransferId(42),
        CHUNK_SIZE,
        Duration::from_millis(31 * 3),
        blob_to_transfer.clone(),
    );

    let mut now = Instant::now();

    for _ in 0..ITERATION_COUNT {
        let send_commands = out_logic.send(now).expect("should work");
        for send_command in send_commands {
            // Intentionally drop commands (datagrams)
            if !drop_rng.gen_bool(0.2) {
                let commands_from_receiver = in_logic.update(&send_command).expect("should work");
                if !drop_rng.gen_bool(0.2) {
                    out_logic
                        .receive(commands_from_receiver)
                        .expect("should work");
                } else {
                    trace!("dropped from receiver to sender: {:?}", send_command);
                }
            } else {
                trace!("dropped from sender to receiver: {:?}", send_command);
            }
        }
        now += Duration::from_millis(32);
    }

    assert_eq!(
        in_logic.blob().expect("blob should be ready"),
        blob_to_transfer
    );

    assert!(out_logic.is_received_by_remote());
}
