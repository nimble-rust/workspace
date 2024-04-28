/*----------------------------------------------------------------------------------------------------------
 *  Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/host
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------------------*/
use std::collections::HashMap;

use nimble_participant::ParticipantId;
use nimble_steps::Steps;

pub struct Combinator<T> {
    pub in_buffers: HashMap<ParticipantId, Steps<T>>,
}

impl<T> Default for Combinator<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Combinator<T> {
    pub fn new() -> Self {
        Combinator {
            in_buffers: HashMap::new(),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    enum TestStep {
        InGame(i8),
        SelectTeam(u16),
    }

    #[test]
    fn test_combinator_add() {
        let mut combinator = Combinator::<TestStep>::new();
        combinator.create_buffer(ParticipantId(1));
        combinator.create_buffer(ParticipantId(2));

        combinator.add(ParticipantId(1), TestStep::InGame(-2));
        combinator.add(ParticipantId(2), TestStep::SelectTeam(42));

        assert_eq!(combinator.in_buffers.len(), 2);
        assert_eq!(combinator.in_buffers.get(&ParticipantId(1)).unwrap().len(), 1);
        let steps_for_participant_1: &mut Steps<TestStep> = combinator.in_buffers.get_mut(&ParticipantId(1)).unwrap();
        let first_step_for_participant_1 = steps_for_participant_1.pop().unwrap();
        assert_eq!(first_step_for_participant_1.step, TestStep::InGame(-2));

        assert_eq!(combinator.in_buffers.get(&ParticipantId(2)).unwrap().len(), 1);
    }
}
