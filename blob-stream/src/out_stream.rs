/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use std::cmp::Ordering;
use std::time::{Duration, Instant};

/// Represents an individual chunk of the blob data being streamed out.
/// Each `BlobStreamOutEntry` holds metadata about a chunk, including:
/// - `timer`: The time when the chunk was last sent, or `None` if it has not been sent.
/// - `index`: The index of the chunk.
/// - `start` and `end`: Byte ranges representing the chunk's position within the full blob.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlobStreamOutEntry {
    pub timer: Option<Instant>,
    pub index: usize,
    pub start: usize,
    pub end: usize,
}

impl BlobStreamOutEntry {
    /// Creates a new `BlobStreamOutEntry`.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the chunk.
    /// * `start` - The start position of the chunk in the blob.
    /// * `end` - The end position of the chunk in the blob.
    ///
    /// # Returns
    ///
    /// A new `BlobStreamOutEntry` with a `None` timer.
    #[must_use]
    pub fn new(index: usize, start: usize, end: usize) -> Self {
        Self {
            timer: None,
            index,
            start,
            end,
        }
    }

    /// Updates the timer to the specified `Instant`, marking the time this entry was last sent.
    ///
    /// # Arguments
    ///
    /// * `time` - The `Instant` at which the entry is being sent.
    pub fn sent_at_time(&mut self, time: Instant) {
        self.timer = Some(time);
    }
}

impl Ord for BlobStreamOutEntry {
    /// Compares two `BlobStreamOutEntry` instances.
    ///
    /// The comparison is done first by the `timer` field. If the timers are equal or
    /// `None`, the `index` field is used as a secondary criterion.
    fn cmp(&self, other: &Self) -> Ordering {
        self.timer
            .cmp(&other.timer) // Compare by timer first
            .then(self.index.cmp(&other.index)) // If timer is the same, compare by index
    }
}

impl PartialOrd for BlobStreamOutEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Manages the streaming out of binary blob data, split into fixed-size chunks.
/// `BlobStreamOut` keeps track of which chunks have been sent, the time they were sent,
/// and controls resending based on elapsed time since the last send.
#[allow(unused)]
#[derive(Debug)]
pub struct BlobStreamOut {
    pub(crate) entries: Vec<BlobStreamOutEntry>,
    pub(crate) fixed_chunk_size: usize,
    start_index_to_send: usize,
    index_to_start_from_if_not_filled_up: usize,
    resend_duration: Duration,
    blob: Vec<u8>,
}

impl BlobStreamOut {
    /// Creates a new `BlobStreamOut` instance.
    ///
    /// # Arguments
    ///
    /// * `fixed_chunk_size` - The size of each chunk.
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
    pub fn new(fixed_chunk_size: usize, resend_duration: Duration, blob: &[u8]) -> Self {
        assert_ne!(fixed_chunk_size, 0, "fixed chunk size cannot be zero");

        let chunk_count = blob.len().div_ceil(fixed_chunk_size);

        // Initialize the entries vector by chunking the blob data
        let entries: Vec<BlobStreamOutEntry> = (0..chunk_count)
            .map(|i| {
                let start = i * fixed_chunk_size;
                let end = std::cmp::min(start + fixed_chunk_size, blob.len());
                BlobStreamOutEntry::new(i, start, end)
            })
            .collect();

        Self {
            entries,
            fixed_chunk_size,
            resend_duration,
            index_to_start_from_if_not_filled_up: 0,
            start_index_to_send: 0,
            blob: blob.to_vec(),
        }
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
    pub fn send(&mut self, now: Instant, max_count: usize) -> Vec<BlobStreamOutEntry> {
        // Filter by index range, timer expiration, and limit the number of results
        let mut filtered_out: Vec<BlobStreamOutEntry> = self
            .entries
            .iter()
            .skip(self.start_index_to_send)
            .take(max_count) // Limit to MAX_COUNT entries
            .filter(|entry| {
                // Check if enough time has passed since the timer was set
                entry
                    .timer
                    .map_or(true, |t| now.duration_since(t) >= self.resend_duration)
            })
            .cloned() // Clone to return owned entries
            .collect(); // Collect into a Vec

        if filtered_out.len() < max_count {
            let remaining = max_count - filtered_out.len();

            if self.index_to_start_from_if_not_filled_up + remaining >= self.entries.len() {
                self.index_to_start_from_if_not_filled_up = self.entries.len() - 1 - remaining;
            }

            // Get additional entries starting from `index_to_start_from_if_not_filled_up`
            let additional_entries: Vec<BlobStreamOutEntry> = self
                .entries
                .iter()
                .skip(self.index_to_start_from_if_not_filled_up) // Start from the alternate index
                .filter(|entry| {
                    // Ensure that we are not duplicating any already selected entries
                    !filtered_out.iter().any(|e| e.index == entry.index)
                })
                .cloned()
                .take(remaining) // Take only the number of remaining entries
                .collect();

            if !additional_entries.is_empty() {
                let last_additional_index = additional_entries[additional_entries.len() - 1].index;
                if last_additional_index + 1 >= self.entries.len() {
                    self.index_to_start_from_if_not_filled_up = 0;
                } else {
                    self.index_to_start_from_if_not_filled_up = last_additional_index + 1;
                }
            }
            // Append additional entries to fill up to `max_count`
            filtered_out.extend(additional_entries);
        }

        for entry in filtered_out.iter() {
            let ent = self
                .entries
                .get_mut(entry.index)
                .expect("should always be there");
            ent.sent_at_time(now);
        }

        filtered_out
    }
}
