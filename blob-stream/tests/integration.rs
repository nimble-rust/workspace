/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use blob_stream::out_logic::Logic;
use blob_stream::prelude::TransferId;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use std::time::{Duration, Instant};

fn generate_deterministic_blob_array(length: usize, seed: u64) -> Vec<u8> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..length).map(|_| rng.gen()).collect()
}

#[test]
fn test_blob_stream() {
    let seed = 12345678;
    let blob_to_transfer = generate_deterministic_blob_array(32, seed);
    const CHUNK_SIZE: usize = 4;

    let mut in_logic = blob_stream::in_logic::Logic::new(blob_to_transfer.len(), CHUNK_SIZE);
    let mut out_logic = Logic::new(
        TransferId(0),
        CHUNK_SIZE,
        Duration::from_millis(30),
        blob_to_transfer.clone(),
    );

    let now = Instant::now();

    for _ in 0..20 {
        const MAX_CHUNK_COUNT_EACH_SEND: usize = 10;

        let set_chunks = out_logic.send(now, MAX_CHUNK_COUNT_EACH_SEND);
        assert!(set_chunks.len() <= MAX_CHUNK_COUNT_EACH_SEND);

        for set_chunk in set_chunks {
            in_logic
                .update(&set_chunk.data)
                .expect("should always be valid in test");
        }
    }

    assert_eq!(
        in_logic.blob().expect("blob should be ready"),
        blob_to_transfer
    );
}
