/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use nimble_client::client::{ClientPhase, ClientStream};
use nimble_protocol::{hex_output, Version};
use nimble_sample_step::{SampleGame, SampleStep};
use nimble_steps::Step;
use secure_random::SecureRandom;
use std::cell::RefCell;
use std::io;
use std::rc::Rc;

#[derive(Debug)]
pub struct FakeRandom {
    pub counter: u64,
}

impl SecureRandom for FakeRandom {
    fn get_random_u64(&mut self) -> u64 {
        let value = self.counter;
        self.counter += 1;
        value
    }
}

#[test]
fn connect_stream() -> io::Result<()> {
    let random = FakeRandom {
        counter: 0x0001020304050607,
    };
    let application_version = Version {
        major: 0,
        minor: 1,
        patch: 2,
    };

    let mut stream: ClientStream<SampleGame, Step<SampleStep>> =
        ClientStream::new(Rc::new(RefCell::new(random)), &application_version);

    let octet_vector = stream.send()?;
    assert_eq!(octet_vector.len(), 1);

    assert_eq!(
        octet_vector[0],
        &[
            0,    // ConnectionId == 0 (OOB)
            0x05, // Connect Request: ClientToHostOobCommand::ConnectType = 0x05
            0, 0, 0, 0, 0, 5, // Nimble version
            0, // Flags (use debug stream)
            0, 0, 0, 1, 0, 2, // Application version
            0, 1, 2, 3, 4, 5, 6, 7 // Client Request Id (normally random u64)
        ]
    );

    let phase = stream.debug_phase();

    println!("phase {phase:?}");

    assert!(matches!(phase, &ClientPhase::Connecting(_)));
    const EXPECTED_CONNECTION_ID: u8 = 0x42;

    let connect_response_from_host = [
        0x00, // ConnectionId == 0 (OOB)
        0x0D, // Connect Response
        0x00, // Flags
        0x00,
        0x01,
        0x02,
        0x03,
        0x04,
        0x05,
        0x06,
        0x07, // Client Request ID. This is normally random,
        // but we know the expected value due to using FakeRandom.
        EXPECTED_CONNECTION_ID, // Connection ID
        0x00,
        0x01,
        0x02,
        0x03,
        0x04,
        0x05,
        0x06,
        0x07, // Connection Secret
    ];

    stream.receive(&connect_response_from_host)?;

    // Verify
    let phase = stream.debug_phase();

    println!("phase {phase:?}");

    assert!(matches!(phase, &ClientPhase::Connected(_)));

    let connected_info = stream
        .debug_connect_info()
        .expect("connect info should be available when connected");

    assert_eq!(connected_info.connection_id.0, 0x42);
    assert_eq!(
        connected_info.session_connection_secret.value,
        0x0001020304050607
    );

    let datagrams_request_download_state = stream.send()?;
    assert_eq!(datagrams_request_download_state.len(), 1);
    let datagram_request_download_state = &datagrams_request_download_state[0];

    let expected_request_download_state_octets = &[
        EXPECTED_CONNECTION_ID,
        0x7B,
        0xC5,
        0x52,
        0xD8, // HASH
        0,
        0,
        0,
        0,    //?
        0x03, // Download Game State
        0x99, // Download Request id, //TODO: Hardcoded, but should not be
    ];
    assert_eq!(
        datagram_request_download_state,
        expected_request_download_state_octets
    );

    /*
    0x0b
           stream.write_u8(self.client_request)?;
        TickIdUtil::to_stream(self.tick_id, stream)?;
        stream.write_u16(self.blob_stream_channel)
     */

    let request_download_response = &[
        EXPECTED_CONNECTION_ID,
        0x86,
        0x2f,
        0x1b,
        0xe6, // HASH
        0,
        0,
        0,
        0,    // Sequence number
        0x0B, //Download Game State Response
        0x99, //Client Request Id // TODO: Hardcoded but should not be
        0x00,
        0x00,
        0x00,
        0x00, // TickID for state
        0x00,
        0x00, // Blob Stream channel to use
    ];

    stream.receive(request_download_response)?;

    let datagrams_request_step = stream.send()?;

    assert_eq!(datagrams_request_step.len(), 1);

    let only_datagram = &datagrams_request_step[0];
    println!("{}", hex_output(only_datagram));

    /*
        #[derive(Debug, Clone)]
    pub struct StepsRequest<StepT: Clone + Serialize + Deserialize + Debug> {
        pub ack: StepsAck,
        pub combined_predicted_steps: PredictedStepsForAllPlayers<StepT>,
    }
    StepsAck:
            stream.write_u32(self.waiting_for_tick_id)?;
            stream.write_u64(self.lost_steps_mask_after_last_received)?;

    Predicted:
     stream.write_u8(self.predicted_players.len() as u8)?;

            for (local_index, predicted_steps_for_one_player) in self.predicted_players.iter() {
                stream.write_u8(*local_index)?;
                predicted_steps_for_one_player.to_stream(stream)?;
            }

         */

    let expected_steps_request_octets = &[
        EXPECTED_CONNECTION_ID,
        0x1A,
        0x93,
        0x76,
        0x47, // HASH
        0,
        0,
        0,
        0,    //?
        0x02, // Steps Request
        // Steps Ack
        0x00,
        0x00,
        0x00,
        0x00, // Waiting for this tick ID
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // Receive mask
        // Predicted Steps
        0x00, // Number of local participants
    ];

    // assert_eq!(only_datagram, expected_steps_request_octets);

    Ok(())
}
