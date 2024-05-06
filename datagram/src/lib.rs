use std::io::Result;

pub trait DatagramSender {
    /// Sends a UDP datagram of up to 1200 octets to the specified address.
    /// Returns the number of bytes sent on success.
    fn send_datagram(&self, data: &[u8]) -> Result<()>;
}


pub trait ReceiveDatagram {
    /// Receives a datagram and stores it into the provided buffer.
    /// Returns the number of bytes read on success.
    ///
    /// # Arguments
    /// * `buffer` - A mutable reference to a slice of u8 where the datagram will be stored.
    ///
    /// # Returns
    /// A `Result` containing either the number of bytes that were written to the buffer, or an I/O error.
    fn receive_datagram(&self, buffer: &mut [u8]) -> Result<usize>;
}
