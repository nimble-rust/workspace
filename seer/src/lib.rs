/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use log::info;
use std::marker::PhantomData;

use nimble_steps::Steps;
use tick_id::TickId;

pub trait SeerCallback<CombinedStepT> {
    fn on_pre_ticks(&mut self) {}

    fn on_tick(&mut self, step: &CombinedStepT);

    fn on_post_ticks(&mut self) {}
}

// Define the Assent struct
impl<Callback, CombinedStepT> Default for Seer<Callback, CombinedStepT>
where
    Callback: SeerCallback<CombinedStepT>,
{
    fn default() -> Self {
        Self::new()
    }
}

pub struct Seer<Callback, CombinedStepT>
where
    Callback: SeerCallback<CombinedStepT>,
{
    combined_steps: Steps<CombinedStepT>,
    authoritative_has_changed: bool,
    phantom: PhantomData<Callback>,
}

impl<Callback, CombinedStepT> Seer<Callback, CombinedStepT>
where
    Callback: SeerCallback<CombinedStepT>,
{
    pub fn new() -> Self {
        Seer {
            combined_steps: Steps::new(),
            phantom: PhantomData,
            authoritative_has_changed: false,
        }
    }

    pub fn update(&mut self, callback: &mut Callback) {
        callback.on_pre_ticks();

        info!("combined steps len:{}", self.combined_steps.len());
        for combined_step_info in self.combined_steps.iter() {
            callback.on_tick(&combined_step_info.step);
        }

        callback.on_post_ticks();
        self.authoritative_has_changed = false;
    }

    pub fn received_authoritative(&mut self, tick: TickId) {
        self.combined_steps.pop_up_to(tick + 1);
    }

    pub fn authoritative_has_changed(&mut self) {
        self.authoritative_has_changed = true;
    }

    pub fn push(&mut self, combined_step: CombinedStepT) {
        self.combined_steps.push(combined_step);
    }
}

#[cfg(test)]
mod tests {
    use nimble_steps::Deserialize;
    use std::io;

    use super::*;

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
        seer.combined_steps.push(TestGameStep::MoveRight);
        seer.update(&mut game);
        assert_eq!(game.position_x, -43);
    }
}
