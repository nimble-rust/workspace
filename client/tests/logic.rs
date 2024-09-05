/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
mod types;

use nimble_client::logic::ClientLogic;
use nimble_protocol::prelude::ClientToHostCommands;
use secure_random::GetRandom;
use types::ExampleGame;
use crate::types::ExampleStep;

#[test]
fn basic_logic() {
    let random = GetRandom {};
    let random_box = Box::new(random);
    let logic = ClientLogic::<ExampleGame, ExampleStep>::new(random_box);

    {
        let commands = logic.send();
        assert_eq!(commands.len(), 1);
        assert!(matches!(commands[0], ClientToHostCommands::Steps(_)))
    }
}