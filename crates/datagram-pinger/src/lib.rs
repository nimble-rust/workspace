/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use flood_rs::prelude::*;

#[derive(Debug)]
pub struct ClientTime(u16);

impl ClientTime {
    pub fn new(time: u16) -> Self {
        Self(time)
    }
}

pub fn client_out_ping(
    client_time: ClientTime,
    stream: &mut impl WriteOctetStream,
) -> std::io::Result<()> {
    stream.write_u16(client_time.0)
}

pub fn client_in_ping(stream: &mut InOctetStream) -> std::io::Result<ClientTime> {
    Ok(ClientTime(stream.read_u16()?))
}
