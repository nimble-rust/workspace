/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use anyhow::{self, Context};
use easy_repl::{command, CommandStatus, Repl};
use example_client::ExampleClient;
use nimble_assent::AssentCallback;
use nimble_rectify::RectifyCallback;
use nimble_seer::SeerCallback;
use nimble_steps::Deserialize;

struct ExampleGame;

#[derive(Clone)]
struct ExampleStep;

impl Deserialize for ExampleStep {
    fn deserialize(bytes: &[u8]) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl SeerCallback<ExampleStep> for ExampleGame {
    fn on_tick(&mut self, step: &ExampleStep) {}
}

impl AssentCallback<ExampleStep> for ExampleGame {
    fn on_tick(&mut self, step: &ExampleStep) {}

    fn on_pre_ticks(&mut self) {}

    fn on_post_ticks(&mut self) {}
}

impl RectifyCallback for ExampleGame {
    fn on_copy_from_authoritative(&mut self) {}
}

fn main() -> anyhow::Result<()> {
    let client = ExampleClient::<ExampleGame, ExampleStep>::new("localhost:27000");

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
