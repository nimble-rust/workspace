/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use anyhow::{self, Context};
use easy_repl::{command, CommandStatus, Repl};
use example_client::layer::ExampleClientWithLayer;
use log::{info, warn};
use nimble_rust::Step;
use nimble_sample_step::SampleStep;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut client = ExampleClientWithLayer::<Step<SampleStep>>::new("localhost:23000");

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
        .add(
            "update",
            command! {
                "Update the client",
                () => || {
                    info!("client update");
                    let result = client.update();
                    match result {
                        Ok(_) => { info!("worked!"); }
                        Err(err) => { warn!("not worked: {}", err)}
                    }
                    Ok(CommandStatus::Done)
                }
            },
        )
        .build()
        .context("Failed to create repl")?;

    repl.run()
}
