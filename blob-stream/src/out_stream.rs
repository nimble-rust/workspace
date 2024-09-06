/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use std::time::{Duration, Instant};

/// Represents an individual chunk of the blob data being streamed out.
/// Each `BlobStreamOutEntry` holds metadata about a chunk, including:
/// - `timer`: The time when the chunk was last sent, or `None` if it has not been sent.
/// - `index`: The index of the chunk.
/// - `start` and `end`: Byte ranges representing the chunk's position within the full blob.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlobStreamOutEntry {
    pub last_sent_at: Option<Instant>,
    pub index: usize,
}

impl BlobStreamOutEntry {
    /// Creates a new `BlobStreamOutEntry`.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the chunk.
    ///
    /// # Returns
    ///
    /// A new `BlobStreamOutEntry` with a `None` timer.
    #[must_use]
    pub fn new(index: usize) -> Self {
        Self {
            last_sent_at: None,
            index,
        }
    }

    /// Updates the timer to the specified `Instant`, marking the time this entry was last sent.
    ///
    /// # Arguments
    ///
    /// * `time` - The `Instant` at which the entry is being sent.
    pub fn sent_at_time(&mut self, time: Instant) {
        self.last_sent_at = Some(time);
    }
}

/// Manages the streaming out of binary blob data, split into fixed-size chunks.
/// `BlobStreamOut` keeps track of which chunks have been sent, the time they were sent,
/// and controls resending based on elapsed time since the last send.
#[allow(unused)]
#[derive(Debug)]
pub struct BlobStreamOut {
    pub(crate) entries: Vec<BlobStreamOutEntry>,
    start_index_to_send: usize,
    index_to_start_from_if_not_filled_up: usize,
    resend_duration: Duration,
}

impl BlobStreamOut {
    /// Creates a new `BlobStreamOut` instance.
    ///
    /// # Arguments
    ///
    /// * `chunk_count` - The total number of chunks.
    /// * `resend_duration` - The minimum time that must elapse before resending a chunk.
    /// * `blob` - The complete binary data to be streamed out.
    ///
    /// # Returns
    ///
    /// A new `BlobStreamOut` initialized with the provided chunk size, resend duration, and blob data.
    ///
    /// # Panics
    ///
    /// This function will panic if `fixed_chunk_size` is zero.
    #[must_use]
    pub fn new(chunk_count: usize, resend_duration: Duration) -> Self {
        assert_ne!(chunk_count, 0, "chunk_count cannot be zero");

        // Initialize the entries vector by chunking the blob data
        let entries: Vec<BlobStreamOutEntry> = (0..chunk_count)
            .map(|i| BlobStreamOutEntry::new(i))
            .collect();

        Self {
            entries,
            resend_duration,
            index_to_start_from_if_not_filled_up: 0,
            start_index_to_send: 0,
        }
    }

    pub fn chunk_count(&self) -> usize {
        self.entries.len()
    }

    /// Sets the starting index from which to send the next chunk.
    ///
    /// # Arguments
    ///
    /// * `index` - The starting index of the next chunk to be sent.
    pub fn set_waiting_for_chunk_index(&mut self, index: usize) {
        self.start_index_to_send = index;
    }

    /// Sends up to `max_count` chunks, starting from the configured `start_index_to_send`.
    /// Resends chunks if enough time has passed since their last send, or fills in additional
    /// chunks if the number of filtered chunks is less than `max_count`.
    ///
    /// # Arguments
    ///
    /// * `now` - The current time used for calculating elapsed time.
    /// * `max_count` - The maximum number of chunks to send in this batch.
    ///
    /// # Returns
    ///
    /// A vector containing up to `max_count` `BlobStreamOutEntry` items, representing the chunks to be sent.
    pub fn send(&mut self, now: Instant, max_count: usize) -> Vec<usize> {
        // Filter by index range, timer expiration, and limit the number of results
        let mut filtered_out_indices: Vec<usize> = self
            .entries
            .iter()
            .skip(self.start_index_to_send)
            .take(max_count) // Limit to MAX_COUNT entries
            .filter(|entry| {
                // Check if enough time has passed since the timer was set
                entry
                    .last_sent_at
                    .map_or(true, |t| now.duration_since(t) >= self.resend_duration)
            })
            .map(|entry| entry.index)
            .collect(); // Collect into a Vec

        if filtered_out_indices.len() < max_count {
            let mut remaining = max_count - filtered_out_indices.len();
            if remaining >= self.entries.len() {
                remaining = self.entries.len() - 1
            }

            if self.index_to_start_from_if_not_filled_up + remaining >= self.entries.len() {
                self.index_to_start_from_if_not_filled_up = self.entries.len() - 1 - remaining;
            }

            // Get additional entries starting from `index_to_start_from_if_not_filled_up`
            let additional_indicies: Vec<usize> = self
                .entries
                .iter()
                .skip(self.index_to_start_from_if_not_filled_up) // Start from the alternate index
                .filter(|entry| {
                    // Ensure that we are not duplicating any already selected entries
                    !filtered_out_indices.iter().any(|e| *e == entry.index)
                })
                .map(|entry| entry.index)
                .take(remaining) // Take only the number of remaining entries
                .collect();

            if !additional_indicies.is_empty() {
                let last_additional_index = additional_indicies[additional_indicies.len() - 1];
                if last_additional_index + 1 >= self.entries.len() {
                    self.index_to_start_from_if_not_filled_up = 0;
                } else {
                    self.index_to_start_from_if_not_filled_up = last_additional_index + 1;
                }
            }
            // Append additional entries to fill up to `max_count`
            filtered_out_indices.extend(additional_indicies);
        }

        for entry_index in filtered_out_indices.iter() {
            let ent = self
                .entries
                .get_mut(*entry_index)
                .expect("should always be there");
            ent.sent_at_time(now);
        }

        filtered_out_indices
    }
}
