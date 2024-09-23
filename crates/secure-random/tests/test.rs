/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use log::info;
use secure_random::{GetRandom, SecureRandom};

#[test_log::test]
fn check_random() {
    let mut random = GetRandom;
    let result = random.get_random_u64();
    info!("result: {}", result)
}
