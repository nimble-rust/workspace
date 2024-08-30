use log::info;
use nimble_assent::AssentCallback;
use nimble_participant_steps::ParticipantSteps;
use nimble_rectify::{Rectify, RectifyCallback};
use nimble_seer::SeerCallback;
use nimble_steps::{Deserialize, Step};

#[derive(Clone)]
pub struct TestGame {
    pub position_x: i32,
}

impl TestGame {
    pub fn on_tick(&mut self, steps: &ParticipantSteps<TestGameStep>) {
        for (_, step) in steps.steps.iter() {
            match step {
                Custom(TestGameStep::MoveLeft) => self.position_x -= 1,
                Custom(TestGameStep::MoveRight) => self.position_x += 1,
                Step::Forced => todo!(),
                Step::WaitingForReconnect => todo!(),
                Step::Joined(_) => todo!(),
                Step::Left => todo!(),
            }
        }
    }
}

#[derive(Clone)]
pub enum TestGameStep {
    MoveLeft,
    MoveRight,
}

impl Deserialize for TestGameStep {
    fn deserialize(bytes: &[u8]) -> Self {
        match bytes[0] {
            0 => TestGameStep::MoveRight,
            _ => TestGameStep::MoveLeft,
        }
    }
}

pub struct CombinedGame {
    pub authoritative_game: TestGame,
    pub predicted_game: TestGame,
}

impl RectifyCallback for CombinedGame {
    fn on_copy_from_authoritative(&mut self) {
        info!("on_copy_from_authoritative");
        self.predicted_game = self.authoritative_game.clone();
    }
}

impl SeerCallback<ParticipantSteps<TestGameStep>> for CombinedGame {
    fn on_tick(&mut self, combined_step: &ParticipantSteps<TestGameStep>) {
        self.predicted_game.on_tick(combined_step);
    }
}

impl AssentCallback<ParticipantSteps<TestGameStep>> for CombinedGame {
    fn on_tick(&mut self, combined_step: &ParticipantSteps<TestGameStep>) {
        self.authoritative_game.on_tick(combined_step);
    }
}
use nimble_participant::ParticipantId;
use nimble_steps::Step::Custom;

#[test]
fn verify_rectify() {
    let authoritative_game = TestGame { position_x: -44 };
    let predicted_game = TestGame { position_x: -44 };

    let mut callbacks = CombinedGame {
        authoritative_game,
        predicted_game,
    };

    let mut rectify = Rectify::<CombinedGame, ParticipantSteps<TestGameStep>>::new();
    let mut participant_step_combined = ParticipantSteps::<TestGameStep>::new();
    participant_step_combined.insert(ParticipantId(0), Custom(TestGameStep::MoveLeft));

    rectify.push_predicted(participant_step_combined);

    rectify.update(&mut callbacks);

    assert_eq!(callbacks.authoritative_game.position_x, -44);
    assert_eq!(callbacks.predicted_game.position_x, -45);
}
