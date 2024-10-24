use flood_rs::BufferDeserializer;
use hexify::format_hex;
use log::debug;
use nimble_client_with_codec::{
    AssentCallback, ClientWithCodec, RectifyCallback, SeerCallback, Step, StepMap, Version,
    VersionProvider, WrappedOctetStep,
};
use crate::snake_c::{ExampleGame, ExamplePlayerInput};

pub mod snake_c;

#[derive(Debug)]
pub struct SnakeGame {
    authoritative: ExampleGame,
    predicted: ExampleGame,
}

impl VersionProvider for SnakeGame {
    fn version() -> Version {
        Version::new(1, 0, 0)
    }
}

impl BufferDeserializer for SnakeGame {
    fn deserialize(buf: &[u8]) -> std::io::Result<(Self, usize)> {
        debug!("{}", format_hex(&buf));
        let game: ExampleGame = buf.try_into().unwrap();
        debug!(
            "received game. arena:{:?} food:{:?}.\n{game:?}",
            game.area, game.food.position
        );
        Ok((
            SnakeGame {
                authoritative: game,
                predicted: game,
            },
            size_of::<ExampleGame>(),
        ))
    }
}

pub type SnakeStep = WrappedOctetStep<ExamplePlayerInput>;
pub type AuthSnakeStep = StepMap<Step<SnakeStep>>;

impl RectifyCallback for SnakeGame {
    fn on_copy_from_authoritative(&mut self) {
        self.predicted = self.authoritative.clone();
    }
}

impl AssentCallback<AuthSnakeStep> for SnakeGame {
    fn on_tick(&mut self, step: &AuthSnakeStep) {
        // TODO:
    }
}

impl SeerCallback<AuthSnakeStep> for SnakeGame {
    fn on_tick(&mut self, step: &AuthSnakeStep) {
        // TODO:
    }
}

pub struct SnakeClient {
    client: ClientWithCodec<SnakeGame, SnakeStep>,
}

impl SnakeClient {
    pub fn new(url: &str) -> Self {
        Self {
            client: ClientWithCodec::new(url),
        }
    }

    pub fn client(&self) -> &ClientWithCodec<SnakeGame, SnakeStep> {
        &self.client
    }

    pub fn client_mut(&mut self) -> &mut ClientWithCodec<SnakeGame, SnakeStep> {
        &mut self.client
    }
}
