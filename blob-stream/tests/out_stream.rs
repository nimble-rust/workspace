/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use blob_stream::out_stream::BlobStreamOut;
use std::time::{Duration, Instant};

#[test]
fn check_update_timer() {
    let mut stream = BlobStreamOut::new(
        4,
        Duration::from_millis(250),
        &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 17, 18, 19, 20,
            17, 18, 19, 20, 21,
        ],
    );

    let mut now = Instant::now();

    {
        let entries = stream.send(now, 2);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].index, 0);
        assert_eq!(entries[1].index, 1);
    }

    now += Duration::from_millis(100);
    {
        let entries = stream.send(now, 3);

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].index, 2);
        assert_eq!(entries[1].index, 0);
        assert_eq!(entries[2].index, 1);
    }

    now += Duration::from_millis(100);
    {
        let entries = stream.send(now, 3);

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].index, 2);
        assert_eq!(entries[1].index, 3);
        assert_eq!(entries[2].index, 4);
    }

    now += Duration::from_millis(150);
    {
        let entries = stream.send(now, 3);

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].index, 0);
        assert_eq!(entries[1].index, 1);
        assert_eq!(entries[2].index, 5);
    }

    stream.set_waiting_for_chunk_index(3);

    {
        let entries = stream.send(now, 3);

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].index, 4);
        assert_eq!(entries[1].index, 5);
        assert_eq!(entries[2].index, 6);
    }
}
