/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use flood_rs::{InOctetStream, OutOctetStream, ReadOctetStream, WriteOctetStream};
use nimble_assent::AssentCallback;
use nimble_rectify::RectifyCallback;
use nimble_seer::SeerCallback;
use nimble_steps::{Deserialize, Serialize};
use std::io;

#[derive(Clone)]
pub enum SampleStep {
    MoveLeft(i16),
    MoveRight(i16),
    Jump,
}

impl Serialize for SampleStep {
    fn serialize(&self, stream: &mut impl WriteOctetStream) -> io::Result<()>
    where
        Self: Sized,
    {
        match self {
            SampleStep::MoveLeft(amount) => {
                stream.write_u8(0x01)?;
                stream.write_i16(*amount)
            }
            SampleStep::MoveRight(amount) => {
                stream.write_u8(0x02)?;
                stream.write_i16(*amount)
            }
            SampleStep::Jump => stream.write_u8(0x03),
        }
    }
}

impl Deserialize for SampleStep {
    fn deserialize(stream: &mut impl ReadOctetStream) -> io::Result<Self>
    where
        Self: Sized,
    {
        let octet = stream.read_u8()?;
        let value = match octet {
            0x01 => SampleStep::MoveLeft(stream.read_i16()?),
            0x02 => SampleStep::MoveRight(stream.read_i16()?),
            0x03 => SampleStep::Jump,
            _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid input"))?,
        };
        Ok(value)
    }
}

#[derive(Default, Clone)]
pub struct SampleState {
    pub x: i32,
    pub y: i32,
}

impl SampleState {
    pub fn update(&mut self, step: &SampleStep) {
        match step {
            SampleStep::MoveLeft(amount) => self.x -= *amount as i32,
            SampleStep::MoveRight(amount) => self.x += *amount as i32,
            SampleStep::Jump => self.y += 1,
        }
    }

    pub fn to_octets(&self) -> io::Result<Vec<u8>> {
        let mut out = OutOctetStream::new();
        out.write_i32(self.x)?;
        out.write_i32(self.y)?;
        Ok(out.data)
    }

    #[allow(unused)]
    pub fn from_octets(payload: &[u8]) -> io::Result<Self> {
        let mut in_stream = InOctetStream::new(payload.to_vec());
        Ok(Self {
            x: in_stream.read_i32()?,
            y: in_stream.read_i32()?,
        })
    }
}

#[derive(Default)]
pub struct SampleGame {
    pub predicted: SampleState,
    pub authoritative: SampleState,
}

impl SampleGame {
    pub fn authoritative_octets(&self) -> io::Result<Vec<u8>> {
        self.authoritative.to_octets()
    }
}

impl SeerCallback<SampleStep> for SampleGame {
    fn on_tick(&mut self, step: &SampleStep) {
        self.predicted.update(step);
    }
}

impl AssentCallback<SampleStep> for SampleGame {
    fn on_pre_ticks(&mut self) {
        self.predicted = self.authoritative.clone();
    }
    fn on_tick(&mut self, step: &SampleStep) {
        self.predicted.update(step);
    }
    fn on_post_ticks(&mut self) {
        self.authoritative = self.predicted.clone();
    }
}

impl RectifyCallback for SampleGame {
    fn on_copy_from_authoritative(&mut self) {
        self.predicted = self.authoritative.clone();
    }
}
