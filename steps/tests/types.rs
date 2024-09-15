/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
#[derive(Debug, PartialEq, Eq)] // Debug is needed for asserts in tests
pub enum GameInput {
    Jumping(bool),
    MoveHorizontal(i32),
}
