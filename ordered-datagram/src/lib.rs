use std::{fmt, io};

use flood_rs::{ReadOctetStream, WriteOctetStream};

#[derive(Debug, Default, Copy, Clone)]
pub struct DatagramId(u16);

impl DatagramId {
    fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u16(self.0)
    }

    fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<DatagramId> {
        Ok(Self(stream.read_u16()?))
    }
}

impl fmt::Display for DatagramId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DatagramId({:X})", self.0)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct OrderedOut {
    pub sequence_to_send: DatagramId,
}

impl OrderedOut {
    pub fn new() -> Self {
        Self {
            sequence_to_send: DatagramId(0),
        }
    }

    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        self.sequence_to_send.to_stream(stream)
    }
}


#[cfg(test)]
mod tests {
    use crate::{DatagramId, OrderedOut};

    #[test]
    fn ordered_out() {
        let out = OrderedOut {
            sequence_to_send: DatagramId(32),
        };
        assert_eq!(out.sequence_to_send.0, 32);
    }
}
