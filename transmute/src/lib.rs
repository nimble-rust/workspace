/*----------------------------------------------------------------------------------------------------------
 *  Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/client
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------------------*/
use nimble_steps::{Deserialize, ParticipantSteps};

pub trait TransmuteCallback<StepT: Deserialize> {
    fn on_pre_ticks(&mut self) {}

    fn on_tick(&mut self, step: &ParticipantSteps<StepT>);

    fn on_post_ticks(&mut self) {}
}
