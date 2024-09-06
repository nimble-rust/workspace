/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use blob_stream::out_stream::BlobStreamOut;
use std::time::{Duration, Instant};

#[test]
fn check_last_sent_time() {
    let mut stream = BlobStreamOut::new(4, Duration::from_millis(250));

    let mut now = Instant::now();

    {
        let entries = stream.send(now, 2);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], 0);
        assert_eq!(entries[1], 1);
    }

    now += Duration::from_millis(100);
    {
        let entries = stream.send(now, 3);

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0], 2);
        assert_eq!(entries[1], 0);
        assert_eq!(entries[2], 1);
    }

    now += Duration::from_millis(100);
    {
        let entries = stream.send(now, 3);

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0], 0);
        assert_eq!(entries[1], 1);
        assert_eq!(entries[2], 2);
    }

    now += Duration::from_millis(150);
    {
        let entries = stream.send(now, 3);

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0], 0);
        assert_eq!(entries[1], 1);
        assert_eq!(entries[2], 2);
    }

    stream.set_waiting_for_chunk_index(3);

    {
        let entries = stream.send(now, 3);

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0], 3);
        assert_eq!(entries[1], 1);
        assert_eq!(entries[2], 2);
    }
}
