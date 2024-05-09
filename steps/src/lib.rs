/*----------------------------------------------------------------------------------------------------------
 *  Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------------------*/
use std::collections::VecDeque;

use tick_id::TickId;

pub mod pending_steps;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct JoinedData {
    pub participant_id: u8,
    pub tick_id: TickId,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Step<T> {
    Forced,
    WaitingForReconnect,
    Joined(JoinedData),
    Left,
    Custom(T),
}

pub trait Deserialize {
    fn deserialize(bytes: &[u8]) -> Self
    where
        Self: Sized;
}

#[derive(Clone)]
pub struct StepInfo<T> {
    pub step: T,
    pub tick_id: TickId,
}

pub struct Steps<T> {
    steps: VecDeque<StepInfo<T>>,
    expected_read_id: TickId,
    expected_write_id: TickId,
}

impl<T> Steps<T> {
    pub fn iter(&self) -> impl Iterator<Item = &StepInfo<T>> {
        self.steps
            .iter()
            .filter(move |step_info| step_info.tick_id == self.expected_read_id)
    }
}

pub struct FromIndexIterator<'a, T> {
    deque: &'a VecDeque<StepInfo<T>>,
    start_index: usize,
    current_index: usize,
}

impl<'a, T> FromIndexIterator<'a, T> {
    pub fn new(deque: &'a VecDeque<StepInfo<T>>, start_index: usize) -> Self {
        Self {
            deque,
            start_index,
            current_index: start_index,
        }
    }
}

impl<T: Clone> Iterator for FromIndexIterator<'_, T> {
    type Item = StepInfo<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.deque.get(self.current_index)?;
        self.current_index += 1;
        Some(item.clone())
    }
}

impl<T> Default for Steps<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub const TICK_ID_MAX: u32 = u32::MAX;

impl<T> Steps<T> {
    pub fn new() -> Self {
        Self {
            steps: VecDeque::new(),
            expected_read_id: TickId::new(0),
            expected_write_id: TickId::new(0),
        }
    }

    pub fn clear(&mut self) {
        self.steps.clear();
        self.expected_read_id = TickId(0);
        self.expected_write_id = TickId(0);
    }

    pub fn new_with_initial_tick(initial_tick_id: TickId) -> Self {
        Self {
            steps: VecDeque::new(),
            expected_read_id: initial_tick_id,
            expected_write_id: initial_tick_id,
        }
    }

    pub fn push(&mut self, step: T) {
        let info = StepInfo {
            step,
            tick_id: self.expected_write_id,
        };
        self.steps.push_back(info);
        self.expected_write_id += 1;
    }

    pub fn pop(&mut self) -> Option<StepInfo<T>> {
        let info = self.steps.pop_front();
        if let Some(ref step_info) = info {
            assert_eq!(step_info.tick_id, self.expected_read_id);
            self.expected_read_id += 1;
        }
        info
    }

    pub fn pop_up_to(&mut self, tick_id: TickId) {
        while let Some(info) = self.steps.front() {
            if info.tick_id >= tick_id {
                break;
            }

            self.steps.pop_front();
        }
    }

    pub fn pop_count(&mut self, count: usize) {
        if count >= self.steps.len() {
            self.steps.clear();
        } else {
            self.steps.drain(..count);
        }
    }

    pub fn front_tick_id(&self) -> Option<TickId> {
        self.steps.front().map(|step_info| step_info.tick_id)
    }

    pub fn back_tick_id(&self) -> Option<TickId> {
        self.steps.back().map(|step_info| step_info.tick_id)
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    pub fn iter_index(&self, start_index: usize) -> FromIndexIterator<T> {
        FromIndexIterator::new(&self.steps, start_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    enum GameInput {
        Jumping(bool),
        MoveHorizontal(i32),
    }

    #[test]
    fn add_step() {
        let mut steps = Steps::<GameInput>::new_with_initial_tick(TickId(23));
        steps.push(GameInput::MoveHorizontal(-2));
        assert_eq!(steps.len(), 1);
        assert_eq!(steps.front_tick_id().unwrap().value(), 23)
    }

    #[test]
    fn push_and_pop_step() {
        let mut steps = Steps::<GameInput>::new_with_initial_tick(TickId(23));
        steps.push(GameInput::Jumping(true));
        steps.push(GameInput::MoveHorizontal(42));
        assert_eq!(steps.len(), 2);
        assert_eq!(steps.front_tick_id().unwrap().value(), 23);
        assert_eq!(steps.pop().unwrap().step, GameInput::Jumping(true));
        assert_eq!(steps.front_tick_id().unwrap().value(), 24);
    }

    #[test]
    fn push_and_pop_count() {
        let mut steps = Steps::<GameInput>::new_with_initial_tick(TickId(23));
        steps.push(GameInput::Jumping(true));
        steps.push(GameInput::MoveHorizontal(42));
        assert_eq!(steps.len(), 2);
        steps.pop_count(8);
        assert_eq!(steps.len(), 0);
    }

    #[test]
    fn push_and_pop_up_to_lower() {
        let mut steps = Steps::<GameInput>::new_with_initial_tick(TickId(23));
        steps.push(GameInput::Jumping(true));
        steps.push(GameInput::MoveHorizontal(42));
        assert_eq!(steps.len(), 2);
        steps.pop_up_to(TickId(1));
        assert_eq!(steps.len(), 2);
    }

    #[test]
    fn push_and_pop_up_to_equal() {
        let mut steps = Steps::<GameInput>::new_with_initial_tick(TickId(23));
        steps.push(GameInput::Jumping(true));
        steps.push(GameInput::MoveHorizontal(42));
        assert_eq!(steps.len(), 2);
        steps.pop_up_to(TickId::new(24));
        assert_eq!(steps.len(), 1);
    }
}
