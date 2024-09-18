/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use flood_rs::{Deserialize, Serialize};
use nimble_assent::AssentCallback;
use nimble_client_logic::err::ClientError;
use nimble_client_logic::logic::ClientLogic;
use nimble_participant::ParticipantId;
use nimble_protocol::client_to_host::{
    AuthoritativeStep, AuthoritativeStepRangeForAllParticipants, PredictedStep,
    PredictedStepsForAllPlayers, StepsAck, StepsRequest,
};
use nimble_protocol::host_to_client::{
    AuthoritativeStepRange, AuthoritativeStepRanges, GameStepResponse, GameStepResponseHeader,
};
use nimble_protocol::prelude::{ClientToHostCommands, HostToClientCommands};
use nimble_rectify::RectifyCallback;
use nimble_sample_step::{SampleGame, SampleStep};
use nimble_seer::SeerCallback;
use nimble_steps::Step::{Custom, Forced};
use nimble_steps::{Step, StepInfo};
use secure_random::GetRandom;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use test_log::test;
use tick_id::TickId;

#[test]
fn basic_logic() {
    let random = GetRandom;
    let random_box = Rc::new(RefCell::new(random));
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
    GameT: SeerCallback<AuthoritativeStep<StepT>>
        + AssentCallback<AuthoritativeStep<StepT>>
        + RectifyCallback,
    StepT: Clone + Deserialize + Serialize + Debug,
>() -> ClientLogic<GameT, StepT> {
    let random = GetRandom;
    let random_box = Rc::new(RefCell::new(random));

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
        Forced,
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

    // Verify
    let assent = &client_logic.debug_rectify().assent();
    assert_eq!(
        assent
            .end_tick_id()
            .expect("should have end_tick_id by now"),
        TickId(2)
    ); // Should have received TickId 0, 1, and 2.

    let auth_steps = assent.debug_steps();
    assert_eq!(auth_steps.len(), 3);

    let first_participant_id = ParticipantId(255);
    let second_participant_id = ParticipantId(1);

    let mut expected_hash_map = HashMap::<ParticipantId, Step<SampleStep>>::new();
    expected_hash_map.insert(first_participant_id, Custom(SampleStep::MoveLeft(-10)));
    expected_hash_map.insert(second_participant_id, Forced);

    let expected_step = AuthoritativeStep::<Step<SampleStep>> {
        authoritative_participants: expected_hash_map,
    };

    let expected_step_with_step_info = StepInfo::<AuthoritativeStep<Step<SampleStep>>> {
        step: expected_step,
        tick_id: TickId(1),
    };

    assert_eq!(
        *auth_steps
            .debug_get(1)
            .expect("should be able to get index 1"),
        expected_step_with_step_info
    );

    let mut game = SampleGame::default();

    assert_eq!(game.authoritative.x, 0);
    assert_eq!(game.authoritative.y, 0);

    client_logic.update(&mut game);

    assert_eq!(game.authoritative.x, 32000 - 42 + 10); // Right(32000) + Left(42) + Left(-10)
    assert_eq!(game.authoritative.y, 1 + 1 + 1); // Two jumps and a forced

    Ok(())
}