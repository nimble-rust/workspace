/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use std::io::{Error, ErrorKind, Result};
use std::net::UdpSocket;

use datagram::{DatagramReceiver, DatagramSender};

pub struct UdpClient {
    socket: UdpSocket,
}

impl UdpClient {
    pub fn new(host: &str) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;
        socket.connect(host)?;
        Ok(UdpClient { socket })
    }

    pub fn send_datagram(&self, data: &[u8]) -> Result<()> {
        let size = self.socket.send(data)?;
        if size != data.len() {
            return Err(Error::new(
                ErrorKind::WriteZero,
                "failed to send the entire datagram",
            ));
        }
        Ok(())
    }
}

impl DatagramSender for UdpClient {
    fn send(&mut self, data: &[u8]) -> Result<()> {
        self.socket.send(data)?;
        Ok(())
    }
}

impl DatagramReceiver for UdpClient {
    fn receive(&mut self, buffer: &mut [u8]) -> Result<usize> {
        self.socket.recv(buffer)
    }
}
