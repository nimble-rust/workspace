/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use getrandom::getrandom;
use std::fmt::Debug;

pub trait SecureRandom: Debug {
    fn get_random_u64(&mut self) -> u64;
}

#[derive(Debug, Clone)]
pub struct GetRandom;

impl SecureRandom for GetRandom {
    fn get_random_u64(&mut self) -> u64 {
        let mut buf = [0u8; 8]; // Create a buffer for 8 bytes
        getrandom(&mut buf).expect("Failed to get random bytes"); // Fill buffer with random bytes

        // Convert bytes to u64
        u64::from_le_bytes(buf)
    }
}
