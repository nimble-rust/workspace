/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use flood_rs::prelude::*;
use log::info;
use nimble_assent::AssentCallback;
use nimble_protocol::client_to_host::AuthoritativeStep;
use nimble_rectify::RectifyCallback;
use nimble_seer::SeerCallback;
use nimble_steps::Step;
use std::io;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SampleStep {
    MoveLeft(i16),
    MoveRight(i16),
    Jump,
    Nothing,
}

impl Serialize for SampleStep {
    fn serialize(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        match self {
            SampleStep::Nothing => stream.write_u8(0x00),
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
    fn deserialize(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        let octet = stream.read_u8()?;
        let value = match octet {
            0x00 => SampleStep::Nothing,
            0x01 => SampleStep::MoveLeft(stream.read_i16()?),
            0x02 => SampleStep::MoveRight(stream.read_i16()?),
            0x03 => SampleStep::Jump,
            _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid input"))?,
        };
        Ok(value)
    }
}

#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct SampleState {
    pub x: i32,
    pub y: i32,
}

impl SampleState {
    pub fn update(&mut self, step: &AuthoritativeStep<Step<SampleStep>>) {
        for (participant_id, step) in &step.authoritative_participants {
            match &step {
                Step::Custom(custom) => match custom {
                    SampleStep::MoveLeft(amount) => self.x -= *amount as i32,
                    SampleStep::MoveRight(amount) => self.x += *amount as i32,
                    SampleStep::Jump => self.y += 1,
                    SampleStep::Nothing => {}
                },
                Step::Forced => self.y += 1,
                Step::WaitingForReconnect => info!("waiting for reconnect"),
                Step::Joined(data) => info!(
                    "participant {} joined at time {}",
                    participant_id, data.tick_id
                ),
                Step::Left => info!("participant {} left", participant_id),
            }
        }
    }

    pub fn to_octets(&self) -> io::Result<Vec<u8>> {
        let mut out = OutOctetStream::new();
        out.write_i32(self.x)?;
        out.write_i32(self.y)?;
        Ok(out.octets())
    }

    #[allow(unused)]
    pub fn from_octets(payload: &[u8]) -> io::Result<Self> {
        let mut in_stream = InOctetStream::new(payload);
        Ok(Self {
            x: in_stream.read_i32()?,
            y: in_stream.read_i32()?,
        })
    }
}

impl Serialize for SampleState {
    fn serialize(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        stream.write_i32(self.x)?;
        stream.write_i32(self.y)
    }
}

impl Deserialize for SampleState {
    fn deserialize(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        Ok(Self {
            x: stream.read_i32()?,
            y: stream.read_i32()?,
        })
    }
}

#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct SampleGame {
    pub predicted: SampleState,
    pub authoritative: SampleState,
}

impl SampleGame {
    pub fn authoritative_octets(&self) -> io::Result<Vec<u8>> {
        self.authoritative.to_octets()
    }
}

impl Serialize for SampleGame {
    fn serialize(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
        self.authoritative.serialize(stream)
    }
}

impl Deserialize for SampleGame {
    fn deserialize(stream: &mut impl ReadOctetStream) -> io::Result<Self> {
        Ok(Self {
            authoritative: SampleState::deserialize(stream)?,
            predicted: SampleState::default(),
        })
    }
}

impl SeerCallback<AuthoritativeStep<Step<SampleStep>>> for SampleGame {
    fn on_tick(&mut self, step: &AuthoritativeStep<Step<SampleStep>>) {
        self.predicted.update(step);
    }
}

impl AssentCallback<AuthoritativeStep<Step<SampleStep>>> for SampleGame {
    fn on_pre_ticks(&mut self) {
        self.predicted = self.authoritative.clone();
    }
    fn on_tick(&mut self, step: &AuthoritativeStep<Step<SampleStep>>) {
        self.authoritative.update(step);
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