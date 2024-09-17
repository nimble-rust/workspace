/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use flood_rs::{Deserialize, Serialize};
use nimble_assent::AssentCallback;
use nimble_client::err::ClientError;
use nimble_client::logic::ClientLogic;
use nimble_participant::ParticipantId;
use nimble_protocol::client_to_host::{
    AuthoritativeCombinedStepForAllParticipants, AuthoritativeStepRangeForAllParticipants,
    PredictedStep, PredictedStepsForAllPlayers, StepsAck, StepsRequest,
};
use nimble_protocol::host_to_client::{
    AuthoritativeStepRange, AuthoritativeStepRanges, GameStepResponse, GameStepResponseHeader,
};
use nimble_protocol::prelude::{ClientToHostCommands, HostToClientCommands};
use nimble_rectify::RectifyCallback;
use nimble_sample_step::{SampleGame, SampleStep};
use nimble_seer::SeerCallback;
use nimble_steps::Step;
use nimble_steps::Step::Custom;
use secure_random::GetRandom;
use std::collections::HashMap;
use std::fmt::Debug;
use test_log::test;
use tick_id::TickId;

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

fn setup_logic<
    GameT: SeerCallback<AuthoritativeCombinedStepForAllParticipants<StepT>>
        + AssentCallback<AuthoritativeCombinedStepForAllParticipants<StepT>>
        + RectifyCallback,
    StepT: Clone + Deserialize + Serialize + Debug,
>() -> ClientLogic<GameT, StepT> {
    let random = GetRandom;
    let random_box = Box::new(random);

    ClientLogic::<GameT, StepT>::new(random_box)
}

#[test]
fn send_steps() {
    let mut game = SampleGame::default();

    let mut client_logic = setup_logic::<SampleGame, Step<SampleStep>>();

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

fn setup_sample_steps() -> AuthoritativeStepRanges<Step<SampleStep>> {
    let mut range_for_all_participants = HashMap::<ParticipantId, Vec<Step<SampleStep>>>::new();

    let first_steps = vec![
        Custom(SampleStep::Jump),
        Custom(SampleStep::MoveLeft(-10)),
        Custom(SampleStep::MoveRight(32000)),
    ];
    let first_participant_id = ParticipantId(255);
    range_for_all_participants.insert(first_participant_id, first_steps.clone());

    let second_steps = vec![
        Custom(SampleStep::MoveLeft(42)),
        Custom(SampleStep::Jump),
        Custom(SampleStep::Jump),
    ];
    let second_participant_id = ParticipantId(1);
    range_for_all_participants.insert(second_participant_id, second_steps.clone());

    let range_to_send = AuthoritativeStepRange::<Step<SampleStep>> {
        delta_steps_from_previous: 0,
        step_count: first_steps.len() as u8,
        authoritative_steps: AuthoritativeStepRangeForAllParticipants {
            authoritative_participants: range_for_all_participants,
        },
    };

    const EXPECTED_TICK_ID: TickId = TickId(0);
    let ranges_to_send = AuthoritativeStepRanges {
        start_tick_id: EXPECTED_TICK_ID,
        ranges: vec![range_to_send],
    };

    ranges_to_send
}
#[test]
fn receive_authoritative_steps() -> Result<(), ClientError> {
    let mut client_logic = setup_logic::<SampleGame, Step<SampleStep>>();

    // Create a GameStep command
    let response = GameStepResponse::<Step<SampleStep>> {
        response_header: GameStepResponseHeader {
            // We ignore the response for now
            connection_buffer_count: 0,
            delta_buffer: 0,
            last_step_received_from_client: 0,
        },
        authoritative_steps: setup_sample_steps(),
    };
    let command = HostToClientCommands::GameStep(response);

    // Receive
    client_logic.receive(&[command])?;

    // TODO: Verify

    Ok(())
}
