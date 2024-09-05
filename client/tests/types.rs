/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use std::io;
use flood_rs::{InOctetStream, ReadOctetStream};
use nimble_assent::AssentCallback;
use nimble_rectify::RectifyCallback;
use nimble_seer::SeerCallback;
use nimble_steps::Deserialize;

#[derive(Clone)]
pub struct ExampleStep(i32);

impl Deserialize for ExampleStep {
    fn deserialize(bytes: &[u8]) -> io::Result<Self>
    where
        Self: Sized,
    {
        let mut stream = InOctetStream::new(bytes.to_vec());
        Ok(Self(stream.read_i32()?))
    }
}

#[derive(Clone)]
pub struct SimulationState {
    pub x: i32,
}

impl SimulationState {
    pub fn update(&mut self, step: &ExampleStep) {
        self.x += step.0;
    }
}

pub struct ExampleGame {
    pub current: SimulationState,
    pub saved: SimulationState,
}

impl SeerCallback<ExampleStep> for ExampleGame {
    fn on_tick(&mut self, step: &ExampleStep) {
        self.current.update(step);
    }
}

impl AssentCallback<ExampleStep> for ExampleGame {
    fn on_pre_ticks(&mut self) {
        self.current = self.saved.clone();
    }
    fn on_tick(&mut self, step: &ExampleStep) {
        self.current.update(step);
    }
    fn on_post_ticks(&mut self) {
        self.saved = self.current.clone();
    }
}

impl RectifyCallback for ExampleGame {
    fn on_copy_from_authoritative(&mut self) {
        self.current = self.saved.clone();
    }
}