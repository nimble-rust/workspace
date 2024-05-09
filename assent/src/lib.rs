/*----------------------------------------------------------------------------------------------------------
 *  Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------------------*/
use std::marker::PhantomData;

use nimble_steps::Steps;

pub trait AssentCallback<CombinedStepT: Clone> {
    fn on_pre_ticks(&mut self) {}

    fn on_tick(&mut self, step: &CombinedStepT);
}

#[derive(Debug, PartialEq)]
pub enum UpdateState {
    ConsumedAllKnowledge,
    DidNotConsumeAllKnowledge,
}

// Define the Assent struct
pub struct Assent<C, CombinedStepT>
where
    CombinedStepT: Clone,
    C: AssentCallback<CombinedStepT>,
{
    phantom: PhantomData<C>,
    steps: Steps<CombinedStepT>,
}

impl<C, CombinedStepT> Default for Assent<C, CombinedStepT>
where
    CombinedStepT: Clone,
    C: AssentCallback<CombinedStepT>,
{
    fn default() -> Self {
        Assent::new()
    }
}

impl<C, CombinedStepT> Assent<C, CombinedStepT>
where
    CombinedStepT: Clone,
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
        callback.on_pre_ticks();
        for combined_step_info in self.steps.iter() {
            callback.on_tick(&combined_step_info.step);
        }

        self.steps.clear();

        UpdateState::ConsumedAllKnowledge
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct TestGame {
        pub position_x: i32,
    }

    #[derive(Clone, Copy)]
    pub enum TestGameStep {
        MoveLeft,
        MoveRight,
    }

    impl AssentCallback<TestGameStep> for TestGame {
        fn on_tick(&mut self, step: &TestGameStep) {
            match step {
                TestGameStep::MoveLeft => self.position_x -= 1,
                TestGameStep::MoveRight => self.position_x += 1,
            }
        }
    }

    #[test]
    fn test_assent() {
        let mut game = TestGame { position_x: -44 };
        let mut assent: Assent<TestGame, TestGameStep> = Assent::new();
        let step = TestGameStep::MoveLeft;
        assent.push(step);
        assent.update(&mut game);
        assert_eq!(game.position_x, -45);
    }

    #[test]
    fn test_assent_right() {
        let mut game = TestGame { position_x: -44 };
        let mut assent: Assent<TestGame, TestGameStep> = Assent::new();
        let step = TestGameStep::MoveRight;
        assent.push(step);
        assent.push(step);
        assent.update(&mut game);
        assert_eq!(game.position_x, -43);
    }
}
