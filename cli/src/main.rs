/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use anyhow::{self, Context};
use easy_repl::{command, CommandStatus, Repl};
use example_client::ExampleClient;
use flood_rs::{ReadOctetStream, WriteOctetStream};
use nimble_assent::AssentCallback;
use nimble_rectify::RectifyCallback;
use nimble_seer::SeerCallback;
use nimble_steps::{Deserialize, Serialize};

struct ExampleGame;

#[derive(Clone)]
struct ExampleStep;

impl Serialize for ExampleStep {
    fn serialize(&self, _: &mut impl WriteOctetStream) -> std::io::Result<()>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl Deserialize for ExampleStep {
    fn deserialize(_: &mut impl ReadOctetStream) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl SeerCallback<ExampleStep> for ExampleGame {
    fn on_tick(&mut self, _: &ExampleStep) {}
}

impl AssentCallback<ExampleStep> for ExampleGame {
    fn on_tick(&mut self, _: &ExampleStep) {}
}

impl RectifyCallback for ExampleGame {
    fn on_copy_from_authoritative(&mut self) {}
}

fn main() -> anyhow::Result<()> {
    let _ = ExampleClient::<ExampleGame, ExampleStep>::new("localhost:27000");

    let mut repl = Repl::builder()
        .add(
            "join",
            command! {
                "Join with a participant",
                (name: String, local_index: i32) => |name, local_index| {
                    println!("{} : {}", name, local_index);
                    Ok(CommandStatus::Done)
                }
            },
        )
        .build()
        .context("Failed to create repl")?;

    repl.run().context("Critical REPL error")
}
