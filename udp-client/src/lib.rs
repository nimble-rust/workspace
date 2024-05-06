use std::io::{Error, ErrorKind, Result};
use std::net::UdpSocket;

use datagram::{DatagramSender, ReceiveDatagram};

pub struct UdpClient {
    socket: UdpSocket,
}

impl UdpClient {
    fn new(host: &str) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(host)?;
        Ok(UdpClient {
            socket,
        })
    }
}

impl DatagramSender for UdpClient {
    fn send_datagram(&self, data: &[u8]) -> Result<()> {
        let size = self.socket.send(data)?;
        if size != data.len() {
            return Err(Error::new(ErrorKind::WriteZero, "failed to send the entire datagram"));
        }
        Ok(())
    }
}

impl ReceiveDatagram for UdpClient {
    fn receive_datagram(&self, buffer: &mut [u8]) -> Result<usize> {
        self.socket.recv(buffer)
    }
}


#[cfg(test)]
mod tests {
    use datagram::DatagramSender;
    use crate::UdpClient;

    #[test]
    fn it_works() {
        let client = UdpClient::new("localhost:23000").unwrap();
        client.send_datagram(&[0x18, 0x28]).unwrap();
    }
}
