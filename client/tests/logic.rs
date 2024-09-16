/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use nimble_client::logic::ClientLogic;
use nimble_protocol::client_to_host::{
    PredictedStep, PredictedStepsForAllPlayers, StepsAck, StepsRequest,
};
use nimble_protocol::prelude::ClientToHostCommands;
use nimble_sample_step::{SampleGame, SampleStep};
use nimble_steps::Step;
use secure_random::GetRandom;
use test_log::test;

#[test]
fn basic_logic() {
    let random = GetRandom;
    let random_box = Box::new(random);
    let mut game = SampleGame::default();
    let mut client_logic = ClientLogic::<SampleGame, Step<SampleStep>>::new(random_box);

    {
        let commands = client_logic.send();
        assert_eq!(commands.len(), 1);
        if let ClientToHostCommands::Steps(StepsRequest {
            ack:
                StepsAck {
                    latest_received_step_tick_id: 0,
                    lost_steps_mask_after_last_received: 0b0,
                },
            combined_predicted_steps: PredictedStepsForAllPlayers { predicted_players },
        }) = &commands[0]
        {
            assert_eq!(predicted_players.len(), 0);
        } else {
            panic!("Command did not match expected structure or pattern");
        }

        client_logic.update(&mut game);

        assert_eq!(game.predicted.x, 0);
    }
}

#[test]
fn send_steps() {
    let random = GetRandom;
    let random_box = Box::new(random);
    let mut game = SampleGame::default();
    let mut client_logic = ClientLogic::<SampleGame, Step<SampleStep>>::new(random_box);

    client_logic.add_predicted_step(PredictedStep {
        predicted_players: [(0, Step::Custom(SampleStep::MoveRight(3)))].into(),
    });

    {
        let commands = client_logic.send();
        assert_eq!(commands.len(), 1);
        if let ClientToHostCommands::Steps(StepsRequest {
            ack:
                StepsAck {
                    latest_received_step_tick_id: 0,
                    lost_steps_mask_after_last_received: 0b0,
                },
            combined_predicted_steps: PredictedStepsForAllPlayers { predicted_players },
        }) = &commands[0]
        {
            assert_eq!(predicted_players.len(), 1);
        } else {
            panic!("Command did not match expected structure or pattern");
        }

        client_logic.update(&mut game);

        assert_eq!(game.predicted.x, 3);
        assert_eq!(game.predicted.y, 0);
    }
}
