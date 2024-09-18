use nimble_client::client::ClientStream;
use nimble_protocol::Version;
use nimble_sample_step::{SampleGame, SampleStep};
use nimble_steps::Step;
use secure_random::SecureRandom;


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
fn connect_stream() {
    let mut random = FakeRandom { counter: 0 };
    let application_version = Version {
        major: 0,
        minor: 1,
        patch: 2,
    };

    let mut stream: ClientStream<SampleGame, Step<SampleStep>> = ClientStream::new(&mut random, &application_version);
}