/*----------------------------------------------------------------------------------------------------------
 *  Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------------------*/
use crate::assent::{Assent, UpdateState};
use crate::seer::Seer;
use crate::transmute::TransmuteCallback;
use nimble_steps::Deserialize;

pub trait RectifyCallback {
    fn on_copy_from_authoritative(&mut self);
}

pub struct Rectify<AC, StepT>
    where
        StepT: Deserialize,
        AC: TransmuteCallback<StepT>,
{
    assent: Assent<AC, StepT>,
    seer: Seer<AC, StepT>,
}

impl<AC, StepT> Default for Rectify<AC, StepT>
    where
        StepT: Deserialize,
        AC: TransmuteCallback<StepT>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<AC, StepT> Rectify<AC, StepT>
    where
        StepT: Deserialize,
        AC: TransmuteCallback<StepT>,
{
    pub fn new() -> Self {
        let assent = Assent::new();
        let seer = Seer::new();

        Self { assent, seer }
    }

    pub fn update(
        &mut self,
        //      callback: &mut impl RectifyCallback,
        ac_callback: &mut AC,
        sc_callback: &mut AC,
    ) {
        let consumed_all_knowledge = self.assent.update(ac_callback);
        if consumed_all_knowledge == UpdateState::ConsumedAllKnowledge {
            //callback.on_copy_from_authoritative();
        }

        self.seer.update(sc_callback);
    }
}

#[cfg(test)]
mod tests {
    use nimble_steps::{Deserialize, ParticipantSteps, Step};

    use crate::{rectify::Rectify, transmute::TransmuteCallback};

    use super::RectifyCallback;

    pub struct TestGame {
        pub position_x: i32,
    }

    pub struct TestCallback {
        pub authoritative_game: TestGame,
        pub predicted_game: TestGame,
    }

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

    impl RectifyCallback for TestCallback {
        fn on_copy_from_authoritative(&mut self) {
            info!("on_copy_from_authoritative");
        }
    }

    impl TransmuteCallback<TestGameStep> for TestGame {
        fn on_tick(&mut self, steps: &ParticipantSteps<TestGameStep>) {
            for step in steps.steps.iter() {
                match step.step {
                    Step::Custom(TestGameStep::MoveLeft) => self.position_x -= 1,
                    Step::Custom(TestGameStep::MoveRight) => self.position_x += 1,
                    Step::Forced => todo!(),
                    Step::WaitingForReconnect => todo!(),
                }
            }
        }
    }

    #[test]
    fn verify_rectify() {
        let authoritative_game = TestGame { position_x: -44 };
        let predicted_game = TestGame { position_x: -44 };
        let mut callbacks = TestCallback {
            authoritative_game,
            predicted_game,
        };
        let mut rectify = Rectify::new();
        rectify.update(
            &mut callbacks.authoritative_game,
            &mut callbacks.predicted_game,
        );

        assert_eq!(callbacks.authoritative_game.position_x, -43);
    }
}
