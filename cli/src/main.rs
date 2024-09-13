/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use anyhow::{self, Context};
use easy_repl::{command, CommandStatus, Repl};
use example_client::ExampleClient;


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
