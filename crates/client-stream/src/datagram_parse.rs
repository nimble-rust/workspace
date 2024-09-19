/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use datagram_pinger::{client_in_ping, ClientTime};
use flood_rs::prelude::InOctetStream;
use nimble_connection_layer::{verify_hash, ConnectionLayerMode, ConnectionSecretSeed};
use nimble_protocol::SessionConnectionId;
use ordered_datagram::OrderedIn;
use std::io;
use std::io::Error;
use std::io::ErrorKind;

pub struct NimbleDatagramParser {
    ordered_in: OrderedIn,
    seed: Option<ConnectionSecretSeed>,
}

pub enum DatagramType {
    Oob,
    Connection(SessionConnectionId, ClientTime),
}

impl NimbleDatagramParser {
    pub fn new() -> Self {
        Self {
            ordered_in: OrderedIn::default(),
            seed: None,
        }
    }

    pub(crate) fn set_seed(&mut self, seed: ConnectionSecretSeed) {
        self.seed = Some(seed);
    }

    pub fn parse(&mut self, datagram: &[u8]) -> io::Result<(DatagramType, InOctetStream)> {
        let mut in_stream = InOctetStream::new(datagram);

        let connection_mode = ConnectionLayerMode::from_stream(&mut in_stream)?;

        match connection_mode {
            ConnectionLayerMode::OOB => Ok((DatagramType::Oob, in_stream)),
            ConnectionLayerMode::Connection(connection_layer) => {
                if self.seed.is_none() {
                    Err(Error::new(ErrorKind::InvalidData, "must have a session_connection_secret to receive connection layer datagrams"))?;
                }
                // First verify hash, so it is even "safe" to check the other values
                verify_hash(
                    connection_layer.murmur3_hash,
                    self.seed.unwrap(),
                    &datagram[5..],
                )?;

                self.ordered_in.read_and_verify(&mut in_stream)?;
                let client_time = client_in_ping(&mut in_stream)?;

                let datagram_type = DatagramType::Connection(
                    SessionConnectionId(connection_layer.connection_id.value),
                    client_time,
                );

                Ok((datagram_type, in_stream))
            }
        }
    }
}
