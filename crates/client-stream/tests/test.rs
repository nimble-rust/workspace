use nimble_client::client::{ClientPhase, ClientStream};
use nimble_protocol::Version;
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
        self.counter += 1;
        self.counter
    }
}

#[test]
fn connect_stream() -> io::Result<()> {
    let random = FakeRandom {
        counter: 0x0001020304050607 - 1,
    };
    let application_version = Version {
        major: 0,
        minor: 1,
        patch: 2,
    };

    let mut stream: ClientStream<SampleGame, Step<SampleStep>> =
        ClientStream::new(Rc::new(RefCell::new(random)), &application_version);

    let cmd = stream.send()?;
    assert_eq!(cmd.len(), 1);

    assert_eq!(
        cmd[0],
        &[
            0,    // ConnectionId == 0 (OOB)
            0x05, // Connect Request: ClientToHostOobCommand::ConnectType = 0x05
            0, 0, 0, 0, 0, 5, // Nimble version
            0, // Flags (use debug stream)
            0, 0, 0, 1, 0, 2, // Application version
            0, 1, 2, 3, 4, 5, 6, 7 // Client Request Id (normally random u64)
        ]
    );

    let connect_response_from_host = [
        0x00, // ConnectionId = 0
        0x0D, // Connect Response
        0x00, // Flags
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
        0x07, // Client Request ID. This is normally random, but we know the expected value due to using FakeRandom.
        0x42, // Connection ID
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, // Connection Secret
    ];

    stream.receive(&connect_response_from_host)?;

    // Verify
    let phase = stream.debug_phase();

    println!("phase {phase:?}");

    assert!(matches!(phase, &ClientPhase::Connected(_)));

    Ok(())
}
