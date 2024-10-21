/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use getrandom::getrandom;
use std::fmt::Debug;

pub trait SecureRandom: Debug {
    fn random_u64(&mut self) -> u64;
}

#[derive(Debug, Clone)]
pub struct GetRandom;

impl SecureRandom for GetRandom {
    fn random_u64(&mut self) -> u64 {
        let mut buf = [0u8; 8];
        getrandom(&mut buf).expect("failed to get random octets from `getrandom()`");
        u64::from_le_bytes(buf)
    }
}
