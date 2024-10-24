use log::debug;
use monotonic_time_rs::MonotonicClock;
use nimble_client_with_codec::{ClientError, ClientPhase, LocalIndex, StepMap};
use nimble_participant::ParticipantId;
use nimble_snake_client::snake_c::{ExamplePlayerInGameInput, ExamplePlayerInput, ExamplePlayerInputType, ExamplePlayerInputUnion};
use nimble_snake_client::{SnakeClient, SnakeStep};
use std::thread;
use tick_id::TickId;

fn main() -> Result<(), ClientError> {
    env_logger::init();

    let mut snake_client = SnakeClient::new("127.0.0.1:23000");
    let clock = monotonic_time_rs::InstantMonotonicClock::new();
    let client_with_codec = &mut snake_client.client_mut();

    let mut tick_id = TickId::default();
    loop {
        client_with_codec.update(clock.now())?;
        let result = client_with_codec.client.update(clock.now());
        if let Err(err) = result {
            println!("{err}");
        }

        match client_with_codec.client.phase() {
            ClientPhase::Normal => {}
            ClientPhase::CanSendPredicted => {
                if client_with_codec.client.can_join_player() && client_with_codec.client.local_players().is_empty() {
                    let local_indices: &[LocalIndex] = &[0x42 as LocalIndex];
                    debug!("requesting a participant from local player {local_indices:?}");
                    client_with_codec.client.request_join_player(local_indices)?;
                }

                if client_with_codec.client.required_prediction_count() > 0 {
                    debug!("predicted count {}", client_with_codec.client.required_prediction_count());
                    let mut step_map = StepMap::<SnakeStep>::new();

                    let example_step = ExamplePlayerInput {
                        inputType: ExamplePlayerInputType::Empty,
                        input: ExamplePlayerInputUnion {
                            inGameInput: ExamplePlayerInGameInput {
                                horizontalAxis: 0,
                                verticalAxis: 0,
                                abilityButton: Default::default(),
                            }
                        },
                        intentionalPadding: [0u8; 32],
                    };
                    
                    let first_participant_id = client_with_codec.client.local_players()[0].participant_id;

                    let _ = step_map.insert(first_participant_id, SnakeStep {
                        step: example_step,
                    });

                    let _ = client_with_codec.client.push_predicted_step(tick_id, &step_map);
                    tick_id += 1;
                }
            }
        }


        thread::sleep(std::time::Duration::from_millis(16));
    }
}
