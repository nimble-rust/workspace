/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use nimble_assent::{Assent, AssentCallback, UpdateState};
use nimble_seer::{Seer, SeerCallback};
use nimble_steps::Deserialize;

/// A callback trait that allows a game to handle the event when the authoritative state
pub trait RectifyCallback {
    fn on_copy_from_authoritative(&mut self);
}

/// The `Rectify` struct coordinates between the [`Assent`] and [`Seer`] components, managing
/// authoritative and predicted game states.
pub struct Rectify<
    Game: AssentCallback<StepT> + SeerCallback<StepT> + RectifyCallback,
    StepT: Deserialize + Clone,
> {
    assent: Assent<Game, StepT>,
    seer: Seer<Game, StepT>,
}

impl<
        Game: AssentCallback<StepT> + SeerCallback<StepT> + RectifyCallback,
        StepT: Clone + Deserialize,
    > Default for Rectify<Game, StepT>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<
        Game: AssentCallback<StepT> + SeerCallback<StepT> + RectifyCallback,
        StepT: Clone + Deserialize,
    > Rectify<Game, StepT>
{
    /// Creates a new `Rectify` instance, initializing both [`Assent`] and [`Seer`] components.
    ///
    /// # Returns
    ///
    /// A new `Rectify` instance.
    pub fn new() -> Self {
        let assent = Assent::new();
        let seer = Seer::new();

        Self { assent, seer }
    }

    /// Pushes a predicted step into the [`Seer`] component.
    ///
    /// # Arguments
    ///
    /// * `step` - The predicted step to be pushed.
    pub fn push_predicted(&mut self, step: StepT) {
        self.seer.push(step)
    }

    /// Pushes an authoritative step into the [`Assent`] component. This method is used to
    /// add new steps that have been determined by the authoritative host.
    ///
    /// # Arguments
    ///
    /// * `step` - The authoritative step to be pushed.
    pub fn push_authoritative(&mut self, step: StepT) {
        self.assent.push(step);
        self.seer
            .received_authoritative(self.assent.end_tick_id().unwrap());
    }

    /// Updates the authoritative state. If all the authoritative state has been calculated
    /// it predicts from the last authoritative state.
    /// # Arguments
    ///
    /// * `game` - A mutable reference to the game implementing the necessary callback traits.
    pub fn update(&mut self, game: &mut Game) {
        let consumed_all_knowledge = self.assent.update(game);
        if consumed_all_knowledge == UpdateState::ConsumedAllKnowledge {
            game.on_copy_from_authoritative();
        }

        self.seer.update(game);
    }
}
