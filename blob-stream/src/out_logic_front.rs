/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use crate::out_logic::Logic;
use crate::out_stream::OutStreamError;
use crate::prelude::{
    ReceiverToSenderFrontCommands, SenderToReceiverFrontCommands, StartTransferData, TransferId,
};
use log::debug;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum Phase {
    StartTransfer,
    Transfer,
}

#[allow(unused)]
#[derive(Debug)]
pub struct OutLogicFront {
    out_stream: Logic,
    phase: Phase,
    transfer_id: TransferId,
}

impl OutLogicFront {
    #[allow(unused)]
    pub fn new(
        transfer_id: TransferId,
        fixed_chunk_size: usize,
        resend_duration: Duration,
        blob: Vec<u8>,
    ) -> Self {
        Self {
            out_stream: Logic::new(transfer_id, fixed_chunk_size, resend_duration, blob),
            phase: Phase::StartTransfer,
            transfer_id,
        }
    }

    pub fn receive(
        &mut self,
        command: ReceiverToSenderFrontCommands,
    ) -> Result<(), OutStreamError> {
        match self.phase {
            Phase::StartTransfer => if let ReceiverToSenderFrontCommands::AckStart(ack_transfer_id) = command {
                if self.transfer_id.0 == ack_transfer_id {
                    debug!("received ack, start transfer");
                    self.phase = Phase::Transfer;
                }
            },
            Phase::Transfer => match command {
                ReceiverToSenderFrontCommands::AckChunk(ack_chunk_front) => {
                    self.out_stream.set_waiting_for_chunk_index(
                        ack_chunk_front.data.waiting_for_chunk_index as usize,
                        ack_chunk_front.data.receive_mask_after_last,
                    )?;
                }
                ReceiverToSenderFrontCommands::AckStart(_) => {}
            },
        }
        Ok(())
    }

    #[allow(unused)]
    pub fn send(
        &mut self,
        now: Instant,
    ) -> Result<Vec<SenderToReceiverFrontCommands>, OutStreamError> {
        match self.phase {
            Phase::StartTransfer => Ok(vec![SenderToReceiverFrontCommands::StartTransfer(
                StartTransferData {
                    transfer_id: self.transfer_id.0,
                    total_octet_size: self.out_stream.octet_size() as u32,
                    chunk_size: self.out_stream.chunk_size() as u16,
                },
            )]),

            Phase::Transfer => {
                const MAX_CHUNK_COUNT_EACH_SEND: usize = 10;

                Ok(self
                    .out_stream
                    .send(now, MAX_CHUNK_COUNT_EACH_SEND)
                    .iter()
                    .map(|front_data| SenderToReceiverFrontCommands::SetChunk(front_data.clone()))
                    .collect())
            }
        }
    }

    #[must_use]
    pub fn is_received_by_remote(&self) -> bool {
        self.out_stream.is_received_by_remote()
    }
}