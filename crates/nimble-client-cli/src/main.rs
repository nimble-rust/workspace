/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use anyhow::{self, Context};
use easy_repl::{command, CommandStatus, Repl};
use example_client::ExampleClient;
use flood_rs::BufferDeserializer;
use log::{debug, info, warn};
use nimble_rust::Step;
use nimble_sample_step::SampleStep;

#[derive(Debug)]
struct GenericState {
    #[allow(unused)]
    pub buf: Vec<u8>,
}

impl BufferDeserializer for GenericState {
    fn deserialize(buf: &[u8]) -> std::io::Result<(Self, usize)>
    where
        Self: Sized,
    {
        Ok((GenericState { buf: buf.into() }, buf.len()))
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut client = ExampleClient::<GenericState, Step<SampleStep>>::new("localhost:23000");

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

                    let state = client.state();
                    debug!("{:?}", state);
                    Ok(CommandStatus::Done)
                }
            },
        )
        .build()
        .context("Failed to create repl")?;

    repl.run()
}
