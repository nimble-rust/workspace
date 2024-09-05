/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use flood_rs::{ReadOctetStream, WriteOctetStream};
use nimble_assent::AssentCallback;
use nimble_rectify::RectifyCallback;
use nimble_seer::SeerCallback;
use nimble_steps::{Deserialize, Serialize};
use std::io;

#[derive(Clone)]
pub struct ExampleStep(i32);

impl Serialize for ExampleStep {
    fn serialize(&self, stream: &mut impl WriteOctetStream) -> io::Result<()>
    where
        Self: Sized,
    {
        stream.write_i32(self.0)
    }
}

impl Deserialize for ExampleStep {
    fn deserialize(stream: &mut impl ReadOctetStream) -> io::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self(stream.read_i32()?))
    }
}

#[derive(Default, Clone)]
pub struct SimulationState {
    pub x: i32,
}

impl SimulationState {
    pub fn update(&mut self, step: &ExampleStep) {
        self.x += step.0;
    }
}

#[derive(Default)]
pub struct ExampleGame {
    pub predicted: SimulationState,
    pub authoritative: SimulationState,
}

impl SeerCallback<ExampleStep> for ExampleGame {
    fn on_tick(&mut self, step: &ExampleStep) {
        self.predicted.update(step);
    }
}

impl AssentCallback<ExampleStep> for ExampleGame {
    fn on_pre_ticks(&mut self) {
        self.predicted = self.authoritative.clone();
    }
    fn on_tick(&mut self, step: &ExampleStep) {
        self.predicted.update(step);
    }
    fn on_post_ticks(&mut self) {
        self.authoritative = self.predicted.clone();
    }
}

impl RectifyCallback for ExampleGame {
    fn on_copy_from_authoritative(&mut self) {
        self.predicted = self.authoritative.clone();
    }
}
