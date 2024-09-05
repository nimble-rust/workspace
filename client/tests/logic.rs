/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
mod types;

use crate::types::ExampleStep;
use nimble_client::logic::ClientLogic;
use nimble_protocol::client_to_host::{PredictedStepsForPlayers, StepsAck, StepsRequest};
use nimble_protocol::prelude::ClientToHostCommands;
use secure_random::GetRandom;
use types::ExampleGame;

#[test]
fn basic_logic() {
    let random = GetRandom {};
    let random_box = Box::new(random);
    let mut game = ExampleGame::default();
    let mut logic = ClientLogic::<ExampleGame, ExampleStep>::new(random_box);

    {
        let commands = logic.send();
        assert_eq!(commands.len(), 1);
        if let ClientToHostCommands::Steps(StepsRequest {
            ack:
                StepsAck {
                    latest_received_step_tick_id: 0,
                    lost_steps_mask_after_last_received: 0b0,
                },
            combined_predicted_steps:
                PredictedStepsForPlayers {
                    predicted_steps_for_players,
                },
        }) = &commands[0]
        {
            assert_eq!(predicted_steps_for_players.len(), 1);
        } else {
            panic!("Command did not match expected structure or pattern");
        }

        logic.update(&mut game);

        assert_eq!(game.predicted.x, 0);
    }
}
