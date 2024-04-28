/*----------------------------------------------------------------------------------------------------------
 *  Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/assent
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------------------*/
use std::marker::PhantomData;

use nimble_participant::ParticipantId;
use nimble_participant_steps::ParticipantSteps;
use nimble_steps::{Deserialize, Step, Steps};

pub trait AssentCallback<CombinedStepT: Deserialize + Clone> {
    fn on_pre_ticks(&mut self) {}

    fn on_tick(&mut self, step: CombinedStepT);
}

#[derive(Debug, PartialEq)]
pub enum UpdateState {
    ConsumedAllKnowledge,
    DidNotConsumeAllKnowledge,
}


// Define the Assent struct
pub struct Assent<C, CombinedStepT>
    where
        CombinedStepT: Deserialize + Clone,
        C: AssentCallback<CombinedStepT>,
{
    phantom: PhantomData<C>,
    steps: Steps<CombinedStepT>,
}

impl<C, CombinedStepT> Default for Assent<C, CombinedStepT>
    where
        CombinedStepT: Deserialize + Clone,
        C: AssentCallback<CombinedStepT>,
{
    fn default() -> Self {
        Assent::new()
    }
}

impl<C, CombinedStepT> Assent<C, CombinedStepT>
    where
        CombinedStepT: Deserialize + Clone,
        C: AssentCallback<CombinedStepT>,
{
    pub fn new() -> Self {
        Assent {
            phantom: PhantomData {},
            steps: Steps::new(),
        }
    }

    pub fn push(&mut self, steps: CombinedStepT) {
        self.steps.push(steps);
    }

    pub fn update(&mut self, callback: &mut C) -> UpdateState {
        let bytes = [0u8; 1];
        let step = CombinedStepT::deserialize(&bytes);
        callback.on_tick(step);

        UpdateState::ConsumedAllKnowledge
    }
}

#[cfg(test)]
mod tests {
    use nimble_steps::Step;

    use super::*;

    pub struct TestGame {
        pub position_x: i32,
    }

    #[derive(Clone)]
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

    impl AssentCallback<ParticipantSteps<TestGameStep>> for TestGame {
        fn on_tick(&mut self, steps: ParticipantSteps<TestGameStep>) {
            for step in steps.steps.iter() {
                match step.1 {
                    Step::Custom(TestGameStep::MoveLeft) => self.position_x -= 1,
                    Step::Custom(TestGameStep::MoveRight) => self.position_x += 1,
                    Step::Forced => todo!(),
                    Step::WaitingForReconnect => todo!(),
                    Step::Joined(_) => {}
                    Step::Left => {}
                }
            }
        }
    }

    #[test]
    fn test_assent() {
        let mut game = TestGame { position_x: -44 };
        let mut assent: Assent<TestGame, ParticipantSteps<TestGameStep>> = Assent::new();
        let mut steps = ParticipantSteps::<TestGameStep>::new();
        let step = TestGameStep::MoveLeft;
        let custom_step = Step::Custom(step);
        steps.insert(ParticipantId(42), custom_step);
        assent.push(steps);
        assent.update(&mut game);
        assert_eq!(game.position_x, -43);
    }
}
