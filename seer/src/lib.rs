/*----------------------------------------------------------------------------------------------------------
 *  Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/client
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------------------*/
use crate::transmute::TransmuteCallback;
use nimble_steps::{Deserialize, ParticipantStep, ParticipantSteps, Step, Steps};
use std::marker::PhantomData;

// Define the Assent struct
impl<C, StepT> Default for Seer<C, StepT>
    where
        C: TransmuteCallback<StepT>,
        StepT: Deserialize,
{
    fn default() -> Self {
        Self::new()
    }
}

pub struct Seer<C, StepT>
    where
        C: TransmuteCallback<StepT>,
        StepT: Deserialize,
{
    steps: Steps<StepT>,
    authoritative_has_changed: bool,
    phantom: PhantomData<C>,
}

impl<C, StepT> Seer<C, StepT>
    where
        C: TransmuteCallback<StepT>,
        StepT: Deserialize,
{
    pub fn new() -> Self {
        Seer {
            steps: Steps::new(),
            phantom: PhantomData,
            authoritative_has_changed: false,
        }
    }

    pub fn update(&mut self, callback: &mut C) {
        let mut steps = ParticipantSteps::<StepT>::new();
        let step = ParticipantStep::<StepT>::new(
            0,
            Step::Custom(<StepT>::deserialize(&[0])), // Use the deserialize method from the Deserialize trait
        );

        steps.steps.push(step);

        callback.on_tick(&steps);

        callback.on_post_ticks();
        self.authoritative_has_changed = false;
    }

    pub fn authoritative_has_changed(&mut self) {
        self.authoritative_has_changed = true;
    }

    pub fn push(&mut self, steps: ParticipantSteps<StepT>) {
        self.steps.push(steps);
    }
}

#[cfg(test)]
mod tests {
    use nimble_participant_steps::ParticipantSteps;
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
        fn on_tick(&mut self, step: &ParticipantSteps<TestGameStep>) {
            let first = step.steps.first().unwrap();
            match &first.step {
                Step::Custom(game_step) => match game_step {
                    TestGameStep::MoveLeft => {
                        self.position_x -= 1;
                    }
                    TestGameStep::MoveRight => {
                        self.position_x += 1;
                    }
                },
                _ => {}
            }
        }

        fn on_pre_ticks(&mut self) {}

        fn on_post_ticks(&mut self) {}
    }

    #[test]
    fn test_seer() {
        let mut game = TestGame { position_x: -44 };
        let mut seer: Seer<TestGame, TestGameStep> = Seer::new();
        seer.update(&mut game);
        assert_eq!(game.position_x, -43);
    }
}
