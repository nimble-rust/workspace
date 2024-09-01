/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use nimble_seer::prelude::*;
use nimble_steps::Deserialize;
use std::io;

pub struct TestGame {
    pub position_x: i32,
}

pub enum TestGameStep {
    MoveLeft,
    MoveRight,
}

impl Deserialize for TestGameStep {
    fn deserialize(bytes: &[u8]) -> io::Result<Self> {
        match bytes[0] {
            0 => Ok(TestGameStep::MoveRight),
            _ => Ok(TestGameStep::MoveLeft),
        }
    }
}

impl SeerCallback<TestGameStep> for TestGame {
    fn on_pre_ticks(&mut self) {}

    fn on_tick(&mut self, step: &TestGameStep) {
        match step {
            TestGameStep::MoveLeft => {
                self.position_x -= 1;
            }
            TestGameStep::MoveRight => {
                self.position_x += 1;
            }
        }
    }

    fn on_post_ticks(&mut self) {}
}

#[test]
fn test_seer() {
    let mut game = TestGame { position_x: -44 };
    let mut seer: Seer<TestGame, TestGameStep> = Seer::new();
    seer.push(TestGameStep::MoveRight);
    seer.update(&mut game);
    assert_eq!(game.position_x, -43);
}
