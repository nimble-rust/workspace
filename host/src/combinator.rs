/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use std::collections::HashMap;

use tick_id::TickId;

use nimble_participant::ParticipantId;
use nimble_participant_steps::ParticipantSteps;
use nimble_steps::{Step, Steps};

#[derive(Debug)]
pub enum CombinatorError {
    NotReadyToProduceStep {
        can_provide: usize,
        can_not_provide: usize,
    },
    OtherError,
    // Add more error variants as needed
}

#[derive(Default)]
pub struct Combinator<T> {
    pub in_buffers: HashMap<ParticipantId, Steps<T>>,
    pub tick_id_to_produce: TickId,
}

impl<T> Combinator<T> {
    pub fn new(tick_id_to_produce: TickId) -> Self {
        Combinator {
            in_buffers: HashMap::new(),
            tick_id_to_produce,
        }
    }

    pub fn create_buffer(&mut self, id: ParticipantId) {
        self.in_buffers.insert(id, Steps::new());
    }

    pub fn add(&mut self, id: ParticipantId, step: T) {
        if let Some(buffer) = self.in_buffers.get_mut(&id) {
            buffer.push(step);
        }
    }

    pub fn participants_that_can_provide(&self) -> (usize, usize) {
        let mut participant_count_that_can_not_give_step = 0;
        let mut participant_count_that_can_provide_step = 0;
        for (_, steps) in self.in_buffers.iter() {
            if let Some(first_tick) = steps.front_tick_id() {
                if first_tick != self.tick_id_to_produce {
                    participant_count_that_can_not_give_step += 1;
                    continue;
                } else {
                    participant_count_that_can_provide_step += 1;
                }
            } else {
                participant_count_that_can_not_give_step += 1;
            }
        }

        (
            participant_count_that_can_provide_step,
            participant_count_that_can_not_give_step,
        )
    }

    pub fn produce(&mut self) -> Result<ParticipantSteps<T>, CombinatorError> {
        let (can_provide, can_not_provide) = self.participants_that_can_provide();
        if can_provide == 0 {
            return Err(CombinatorError::NotReadyToProduceStep {
                can_provide,
                can_not_provide,
            });
        }

        let mut combined_step = ParticipantSteps::<T>::new();
        for (participant_id, steps) in self.in_buffers.iter_mut() {
            if let Some(first_tick) = steps.front_tick_id() {
                if first_tick == self.tick_id_to_produce {
                    combined_step.insert(
                        participant_id.clone(),
                        Step::Custom(steps.pop().unwrap().step),
                    )
                } else {
                    combined_step.insert(participant_id.clone(), Step::Forced);
                    steps.pop_up_to(self.tick_id_to_produce);
                }
            }
        }

        self.tick_id_to_produce += 1;

        Ok(combined_step)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    enum TestStep {
        InGame(i8),
        SelectTeam(u16),
    }

    #[test]
    fn test_combinator_add() {
        let mut combinator = Combinator::<TestStep>::new(TickId(0));
        combinator.create_buffer(ParticipantId(1));
        combinator.create_buffer(ParticipantId(2));

        combinator.add(ParticipantId(1), TestStep::InGame(-2));
        combinator.add(ParticipantId(2), TestStep::SelectTeam(42));

        assert_eq!(combinator.in_buffers.len(), 2);
        assert_eq!(
            combinator.in_buffers.get(&ParticipantId(1)).unwrap().len(),
            1
        );
        let steps_for_participant_1: &mut Steps<TestStep> =
            combinator.in_buffers.get_mut(&ParticipantId(1)).unwrap();
        let first_step_for_participant_1 = steps_for_participant_1.pop().unwrap();
        assert_eq!(first_step_for_participant_1.step, TestStep::InGame(-2));

        assert_eq!(
            combinator.in_buffers.get(&ParticipantId(2)).unwrap().len(),
            1
        );

        let combined_step = combinator.produce().unwrap();

        assert_eq!(combined_step.steps.len(), 1);
        let first_step = combined_step.steps.get(&ParticipantId(2)); // Participant 1 has been popped up previously
        assert_eq!(first_step.unwrap(), &Step::Custom(TestStep::SelectTeam(42)));
    }
}
