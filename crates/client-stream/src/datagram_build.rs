/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use datagram::{DatagramBuilder, DatagramError};
use datagram_pinger::{client_out_ping, ClientTime};
use flood_rs::prelude::OutOctetStream;
use flood_rs::WriteOctetStream;
use hexify::format_hex;
use log::trace;
use nimble_connection_layer::{
    prepare_out_stream, write_to_stream, ConnectionId, ConnectionSecretSeed,
};
use ordered_datagram::OrderedOut;
use std::io;

pub struct NimbleDatagramBuilder {
    ordered_datagram_out: OrderedOut,
    max_size: usize,
    stream: OutOctetStream,
    is_empty: bool, // TODO: Add on OutOctetStream
    connection_id: ConnectionId,
    seed: ConnectionSecretSeed,
}

impl NimbleDatagramBuilder {
    pub fn new(max_size: usize) -> Self {
        Self {
            ordered_datagram_out: Default::default(),
            stream: OutOctetStream::new(),
            max_size,
            is_empty: true,
            connection_id: Default::default(),
            seed: ConnectionSecretSeed(0),
        }
    }

    pub fn set_secrets(&mut self, connection_id: ConnectionId, seed: ConnectionSecretSeed) {
        self.connection_id = connection_id;
        self.seed = seed;
    }
    fn finalize_header(
        &mut self,
        connection_id: ConnectionId,
        seed: ConnectionSecretSeed,
    ) -> io::Result<Vec<u8>> {
        let mut payload = self.stream.octets();
        trace!(
            "datagram. finalize: original payload: {}",
            format_hex(&payload)
        );
        let mut hash_stream = OutOctetStream::new();
        let payload_to_calculate_on = &payload[5..];
        write_to_stream(
            &mut hash_stream,
            connection_id,
            seed,
            payload_to_calculate_on,
        )?; // Write hash connection layer header
        payload[..hash_stream.octets().len()].copy_from_slice(hash_stream.octets_ref());
        trace!(
            "datagram. finalize: total    payload: {}",
            format_hex(&payload)
        );
        Ok(payload)
    }
}

impl DatagramBuilder for NimbleDatagramBuilder {
    fn push(&mut self, data: &[u8]) -> Result<(), DatagramError> {
        const FOOTER_SIZE: usize = 1;

        if data.len() > self.max_size - FOOTER_SIZE {
            return Err(DatagramError::ItemSizeTooBig);
        }

        if self.stream.octets().len() + data.len() > self.max_size - FOOTER_SIZE {
            return Err(DatagramError::BufferFull);
        }

        self.stream.write(data)?;
        Ok(())
    }

    fn finalize(&mut self) -> io::Result<Vec<u8>> {
        // Finalize header
        self.finalize_header(self.connection_id, self.seed)
    }

    fn is_empty(&self) -> bool {
        self.is_empty // self.stream.is_empty()
    }

    fn clear(&mut self) -> io::Result<()> {
        self.stream = OutOctetStream::new(); // TODO: implement self.stream.clear()

        prepare_out_stream(&mut self.stream)?; // Connection-Layer: Add hash stream

        self.ordered_datagram_out.to_stream(&mut self.stream)?; // Ordered datagrams

        trace!(
            "datagram header. sequence:{}",
            self.ordered_datagram_out.sequence_to_send
        );

        let client_time = ClientTime::new(0);
        client_out_ping(client_time, &mut self.stream)?;
        self.is_empty = false;

        self.ordered_datagram_out.commit();

        Ok(())
    }
}

pub struct NimbleOobDatagramBuilder {
    max_size: usize,
    stream: OutOctetStream,
    is_empty: bool, // TODO: Add on OutOctetStream
}

impl NimbleOobDatagramBuilder {
    pub fn new(max_size: usize) -> Self {
        Self {
            stream: OutOctetStream::new(),
            max_size,
            is_empty: true,
        }
    }
}

impl DatagramBuilder for NimbleOobDatagramBuilder {
    fn push(&mut self, data: &[u8]) -> Result<(), DatagramError> {
        const FOOTER_SIZE: usize = 1;

        if data.len() > self.max_size - FOOTER_SIZE {
            return Err(DatagramError::ItemSizeTooBig);
        }

        if self.stream.octets().len() + data.len() > self.max_size - FOOTER_SIZE {
            return Err(DatagramError::BufferFull);
        }

        self.stream.write(data)?;
        Ok(())
    }

    fn finalize(&mut self) -> io::Result<Vec<u8>> {
        // Finalize header
        Ok(self.stream.octets())
    }

    fn is_empty(&self) -> bool {
        self.is_empty // self.stream.is_empty()
    }

    fn clear(&mut self) -> io::Result<()> {
        self.stream = OutOctetStream::new(); // TODO: implement self.stream.clear()
        let zero_connection_id = ConnectionId { value: 0 }; // oob
        zero_connection_id.to_stream(&mut self.stream)?; // OOB

        self.is_empty = false;

        Ok(())
    }
}
