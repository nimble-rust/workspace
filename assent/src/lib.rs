/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
pub mod prelude;

use std::marker::PhantomData;

use nimble_steps::Steps;
use tick_id::TickId;

pub trait AssentCallback<CombinedStepT> {
    fn on_pre_ticks(&mut self) {}

    fn on_tick(&mut self, step: &CombinedStepT);

    fn on_post_ticks(&mut self) {}
}

#[derive(Debug, PartialEq)]
pub enum UpdateState {
    ConsumedAllKnowledge,
    DidNotConsumeAllKnowledge,
}

// Define the Assent struct
pub struct Assent<C, CombinedStepT>
where
    C: AssentCallback<CombinedStepT>,
{
    phantom: PhantomData<C>,
    steps: Steps<CombinedStepT>,
}

impl<C, CombinedStepT> Default for Assent<C, CombinedStepT>
where
    C: AssentCallback<CombinedStepT>,
{
    fn default() -> Self {
        Assent::new()
    }
}

impl<C, CombinedStepT> Assent<C, CombinedStepT>
where
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

    pub fn end_tick_id(&self) -> Option<TickId> {
        self.steps.back_tick_id()
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
