/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use flood_rs::{Deserialize, InOctetStream, ReadOctetStream};
use std::io;

/// A trait for parsing and interpreting datagrams.
pub trait DatagramParser {
    /// Parses the entire datagram, validating both the header and footer.
    ///
    /// # Arguments
    ///
    /// * `datagram` - A slice containing the entire datagram.
    ///
    /// # Returns
    ///
    /// * `io::Result<(usize, usize)>` - The range of the slice that contains the payload.
    fn parse<'a>(&mut self, datagram: &'a [u8]) -> io::Result<&'a [u8]>;
}

/// Deserializes a collection of datagrams into a vector of items.
///
/// Each datagram is parsed to extract its payload, and the payload is then
/// deserialized into items of type `T`. All successfully deserialized items
/// are aggregated into a single vector.
///
/// # Arguments
///
/// * `datagrams` - An iterable collection of byte slices, each representing a serialized datagram.
/// * `parser` - A mutable reference to an implementation of `DatagramParser`.
///
/// # Returns
///
/// * `io::Result<Vec<T>>` - A vector of deserialized items.
///
/// # Errors
///
/// Returns an `io::Error` if parsing any datagram or deserializing any item fails.
pub fn deserialize_datagrams<T, I>(
    datagrams: I,
    parser: &mut impl DatagramParser,
) -> io::Result<Vec<T>>
where
    T: Deserialize,
    I: IntoIterator<Item = Vec<u8>>,
{
    let mut items = Vec::new();

    for datagram in datagrams.into_iter() {
        let payload_slice = parser.parse(&datagram)?;

        let mut in_stream = InOctetStream::new(payload_slice);
        while !in_stream.has_reached_end() {
            let item = T::deserialize(&mut in_stream)?;
            items.push(item);
        }
    }

    Ok(items)
}
