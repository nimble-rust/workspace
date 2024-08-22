/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use std::io;
use std::io::Result;

pub trait DatagramSender {
    /// Sends a UDP datagram of up to 1200 octets to the specified address.
    /// Returns the number of bytes sent on success.
    fn send_datagram(&mut self, data: &[u8]) -> Result<()>;
}

pub trait DatagramReceiver {
    /// Receives a datagram and stores it into the provided buffer.
    /// Returns the number of bytes read on success.
    ///
    /// # Arguments
    /// * `buffer` - A mutable reference to a slice of u8 where the datagram will be stored.
    ///
    /// # Returns
    /// A `Result` containing either the number of bytes that were written to the buffer, or an I/O error.
    fn receive_datagram(&mut self, buffer: &mut [u8]) -> Result<usize>;
}

pub trait DatagramCommunicator {
    /// Sends a UDP datagram of up to 1200 octets to the specified address.
    /// Returns the number of bytes sent on success.
    fn send_datagram(&mut self, data: &[u8]) -> Result<()>;

    /// Receives a datagram and stores it into the provided buffer.
    /// Returns the number of bytes read on success.
    ///
    /// # Arguments
    /// * `buffer` - A mutable reference to a slice of u8 where the datagram will be stored.
    ///
    /// # Returns
    /// A `Result` containing either the number of bytes that were written to the buffer, or an I/O error.
    fn receive_datagram(&mut self, buffer: &mut [u8]) -> Result<usize>;
}

pub trait DatagramProcessor {
    fn send_datagram(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn receive_datagram(&mut self, buffer: &[u8]) -> io::Result<Vec<u8>>;
}
