/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
pub mod deserialize;

use flood_rs::{out_stream::OutOctetStream, Serialize};
use std::io;

/// A trait for building datagrams by adding headers and footers.
///
/// Implementations of this trait are responsible for writing the necessary
/// headers and footers to an octet buffer, defining where the payload can be
/// written, and finalizing the datagram so it is ready for transmission.
pub trait DatagramBuilder {
    /// Initializes a new datagram by writing any necessary headers.
    ///
    /// # Arguments
    ///
    /// * `buf` - A mutable slice where the datagram is being constructed.
    ///
    /// # Returns
    ///
    /// * `io::Result<(usize, usize)>` - A tuple containing:
    ///     - The end index of the header within the buffer.
    ///     - The start index of the footer within the buffer.
    fn start_datagram(&mut self, buf: &mut [u8]) -> io::Result<(usize, usize)>;

    /// Finalizes the datagram by performing any fixups and adding footers.
    ///
    /// # Arguments
    ///
    /// * `buf` - A mutable slice containing the entire datagram.
    /// * `payload_len` - The length of the payload written into the datagram.
    ///
    /// # Returns
    ///
    /// * `io::Result<usize>` - The total length of the datagram after adding the footer.
    fn end_datagram(&mut self, buf: &mut [u8], payload_len: usize) -> io::Result<usize>;
}

/// Serializes a list of items into datagrams.
///
/// Each datagram is constructed using the provided [`DatagramBuilder`]. Items are serialized
/// and packed into datagrams without exceeding the `max_packet_size`.
///
/// # Arguments
///
/// * `items` - A slice of items to serialize.
/// * `max_packet_size` - The maximum size of each packet.
/// * `builder` - A mutable reference to an implementation of `DatagramBuilder`.
///
/// # Returns
///
/// * `io::Result<Vec<Vec<u8>>>` - A vector of serialized datagrams.
///
/// # Errors
///
/// Returns an `io::Error` if any item is too large to fit in a packet or if serialization fails.
pub fn serialize_datagrams<T, I>(
    items: I,
    max_packet_size: usize,
    builder: &mut impl DatagramBuilder,
) -> io::Result<Vec<Vec<u8>>>
where
    T: Serialize,
    I: AsRef<[T]>,
{
    let mut packets = Vec::new();
    let mut buffer = vec![0u8; max_packet_size];
    let mut header_end = 0;
    let mut footer_start = 0;
    let mut payload_len = 0;

    for item in items.as_ref() {
        // Start a new packet if necessary (only the first time)
        if payload_len == 0 {
            (header_end, footer_start) = builder.start_datagram(&mut buffer)?;
            payload_len = 0;
        }

        // Serialize the item into the buffer
        let mut item_stream = OutOctetStream::new();
        item.serialize(&mut item_stream)?;
        let item_octets = item_stream.octets_ref();
        let item_len = item_octets.len();

        // Ensure the item fits within the payload space
        if item_len > (footer_start - header_end) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Item size exceeds available payload space",
            ));
        }

        if item_len == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "zero item length is illegal for serialization",
            ));
        }

        // Check if adding the item would exceed the payload capacity
        if payload_len + item_len > (footer_start - header_end) {
            // Finish the current packet
            let total_len = builder.end_datagram(&mut buffer, payload_len)?;
            packets.push(buffer[..total_len].to_vec());

            // Start a new packet
            buffer = vec![0u8; max_packet_size];
            (header_end, footer_start) = builder.start_datagram(&mut buffer)?;
            payload_len = 0;
        }

        // Add the item to the current packet
        buffer[header_end + payload_len..header_end + payload_len + item_len]
            .copy_from_slice(item_octets);
        payload_len += item_len;
    }

    // Finish the last packet if there is any data left
    if payload_len > 0 {
        let total_len = builder.end_datagram(&mut buffer, payload_len)?;
        packets.push(buffer[..total_len].to_vec());
    }

    Ok(packets)
}
