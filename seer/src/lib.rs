/*----------------------------------------------------------------------------------------------------------
 *  Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/client
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------------------*/
use std::marker::PhantomData;

use nimble_steps::{Deserialize, Steps};
use nimble_transmute::TransmuteCallback;

// Define the Assent struct
impl<C, CombinedStepT> Default for Seer<C, CombinedStepT>
    where
        C: TransmuteCallback<CombinedStepT>
{
    fn default() -> Self {
        Self::new()
    }
}

pub struct Seer<C, CombinedStepT>
    where
        C: TransmuteCallback<CombinedStepT>
{
    combined_steps: Steps<CombinedStepT>,
    authoritative_has_changed: bool,
    phantom: PhantomData<C>,
}

impl<C, CombinedStepT> Seer<C, CombinedStepT>
    where
        C: TransmuteCallback<CombinedStepT>
{
    pub fn new() -> Self {
        Seer {
            combined_steps: Steps::new(),
            phantom: PhantomData,
            authoritative_has_changed: false,
        }
    }

    pub fn update(&mut self, callback: &mut C) {
        callback.on_pre_ticks();

        for combined_step_info in self.combined_steps.iter() {
            callback.on_tick(&combined_step_info.step);
        }

        callback.on_post_ticks();
        self.authoritative_has_changed = false;
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
    use nimble_transmute::TransmuteCallback;

    use super::*;

    pub struct TestGame {
        pub position_x: i32,
    }

    pub enum TestGameStep {
        MoveLeft,
        MoveRight,
    }

    impl Deserialize for TestGameStep {
        fn deserialize(bytes: &[u8]) -> Self {
            match bytes[0] {
                0 => TestGameStep::MoveRight,
                _ => TestGameStep::MoveLeft,
            }
        }
    }

    impl TransmuteCallback<TestGameStep> for TestGame {
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



