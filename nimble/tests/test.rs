/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use flood_rs::{Deserialize, Serialize};
use nimble_client::logic::ClientLogic;
use nimble_host::logic::{ConnectionId, HostLogic};
use nimble_host::state::State;
use nimble_protocol::client_to_host::{AuthoritativeCombinedStepForAllParticipants, JoinGameType};
use nimble_protocol::client_to_host::{JoinPlayerRequest, JoinPlayerRequests};
use nimble_protocol::prelude::*;
use nimble_protocol::Nonce;
use nimble_sample_step::{SampleGame, SampleStep};
use nimble_steps::Step;
use secure_random::GetRandom;
use std::fmt::Debug;
use std::time::Instant;
use test_log::test;
use tick_id::TickId;

mod types;

fn communicate<
    SampleGame: nimble_seer::SeerCallback<AuthoritativeCombinedStepForAllParticipants<Step<SampleStep>>>
        + nimble_assent::AssentCallback<AuthoritativeCombinedStepForAllParticipants<Step<SampleStep>>>
        + nimble_rectify::RectifyCallback
        + Clone
        + Deserialize
        + Serialize,
    SampleStep: Clone + Deserialize + Debug + Eq + PartialEq,
>(
    host: &mut HostLogic<Step<SampleStep>>,
    connection_id: ConnectionId,
    client: &mut ClientLogic<SampleGame, Step<SampleStep>>,
) where
    SampleStep: Serialize,
{
    let now = Instant::now();

    let to_host = client.send();
    let to_client: Vec<_> = to_host
        .iter()
        .flat_map(|to_host| {
            host.update(connection_id, now, to_host)
                .expect("should work in test")
        })
        .collect();
    client
        .receive(to_client.as_slice())
        .expect("TODO: panic message");
}

#[test]
fn client_host_integration() {
    let game = SampleGame::default();
    let state_octets = game
        .authoritative_octets()
        .expect("expect it possible to get state");
    let state = State::new(TickId(42), state_octets.as_slice());
    let mut host = HostLogic::<Step<SampleStep>>::new(state);
    let connection = host.create_connection().expect("should create connection");

    let random = GetRandom {};
    let random_box = Box::new(random);
    let mut client = ClientLogic::<SampleGame, Step<SampleStep>>::new(random_box);
    let joining_player = JoinPlayerRequest { local_index: 0 };

    let join_game_request = JoinGameRequest {
        nonce: Nonce(0),
        join_game_type: JoinGameType::NoSecret,
        player_requests: JoinPlayerRequests {
            players: vec![joining_player],
        },
    };
    client.set_joining_player(join_game_request);

    communicate(&mut host, connection, &mut client);
}
