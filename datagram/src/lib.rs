/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use std::io;
use std::io::Result;

/// Trait for sending datagrams.
///
/// The `DatagramSender` trait provides a standardized interface for transmitting
/// datagram-based messages, (e.g. UDP communication). Implementors
/// handle the specifics of datagram transmission, including serialization and
/// interfacing with underlying transport protocols.
pub trait DatagramSender {
    /// Sends a datagram containing the provided data.
    ///
    /// # Arguments
    ///
    /// * `data` - An octet slice to be sent as a datagram.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the datagram was sent successfully.
    /// * `Err(io::Error)` if an error occurred during sending.
    fn send(&mut self, data: &[u8]) -> Result<()>;
}

/// Trait for receiving datagrams.
///
/// The `DatagramReceiver` trait provides a standardized interface for receiving
/// datagram-based messages, (e.g. UDP communication). Implementors
/// of this trait handle the specifics of datagram reception, including deserialization
/// and interfacing with underlying transport protocols.
pub trait DatagramReceiver {
    /// Receives a datagram and stores it into the provided buffer.
    ///
    /// # Arguments
    ///
    /// * `buffer` - A mutable byte slice where the received datagram will be stored.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - The number of bytes written to the buffer.
    /// * `Err(io::Error)` - If an error occurred during reception.
    ///
    /// # Errors
    ///
    /// Returns an `io::Error` if the datagram could not be received. Common error
    /// scenarios include network failures, invalid data formats, or issues with
    /// the underlying transport mechanism.
    fn receive(&mut self, buffer: &mut [u8]) -> Result<usize>;
}

/// A trait that combines sending and receiving datagrams.
pub trait DatagramCommunicator: DatagramSender + DatagramReceiver {
    // Inherits `send_datagram` and `receive_datagram` methods from `DatagramSender` and `DatagramReceiver`.
}

impl<T> DatagramCommunicator for T where T: DatagramSender + DatagramReceiver {}

/// Trait for encoding datagrams.
pub trait DatagramEncoder {
    fn encode(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
}

/// Trait for decoding datagrams.
pub trait DatagramDecoder {
    fn decode(&mut self, buffer: &[u8]) -> io::Result<Vec<u8>>;
}

pub trait DatagramCodec: DatagramEncoder + DatagramDecoder {}
impl<T> DatagramCodec for T where T: DatagramEncoder + DatagramDecoder {}
