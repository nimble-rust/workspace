use crate::snake_c::{ExampleGame, ExamplePlayerInput};
use flood_rs::BufferDeserializer;
use nimble_client_with_codec::AssentCallback;
use nimble_client_with_codec::Step;
use nimble_client_with_codec::StepMap;
use nimble_client_with_codec::{ClientWithCodec, Version, VersionProvider, WrappedOctetStep};
use nimble_client_with_codec::{RectifyCallback, SeerCallback};

mod snake_c;

#[derive(Debug)]
pub struct SnakeGame {
    authoritative: ExampleGame,
    predicted: ExampleGame,
}

impl VersionProvider for SnakeGame {
    fn version() -> Version {
        Version::new(0, 0, 1)
    }
}

impl BufferDeserializer for SnakeGame {
    fn deserialize(buf: &[u8]) -> std::io::Result<(Self, usize)> {
        todo!()
    }
}

pub type SnakeStep = WrappedOctetStep<ExamplePlayerInput>;

impl RectifyCallback for SnakeGame {
    fn on_copy_from_authoritative(&mut self) {
        todo!()
    }
}

impl AssentCallback<StepMap<Step<SnakeStep>>> for SnakeGame {
    fn on_tick(&mut self, step: &StepMap<Step<SnakeStep>>) {
        todo!()
    }
}

impl SeerCallback<StepMap<Step<SnakeStep>>> for SnakeGame {
    fn on_tick(&mut self, step: &StepMap<Step<SnakeStep>>) {
        todo!()
    }
}

pub struct SnakeClient {
    pub client: ClientWithCodec<SnakeGame, SnakeStep>,
}

impl SnakeClient {
    pub fn new(url: &str) -> Self {
        Self {
            client: ClientWithCodec::new(url),
        }
    }
}
