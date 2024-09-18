struct ClientDatagramParser {
    ordered_datagram_in: OrderedIn,
}

impl DatagramParser for ClientDatagramParser {
    fn parse<'a>(&mut self, buf: &'a [u8]) -> std::io::Result<&'a [u8]> {
        let mut in_stream = OctetRefReader::new(buf);
        self.ordered_datagram_in.read_and_verify(&mut in_stream)?;

        Ok(&buf[3..]) // in_stream.len()
    }
}

struct ClientDatagramBuilder {
    ordered_datagram_out: OrderedOut,
    max_size: usize,
    stream: OutOctetStream,
}

impl ClientDatagramBuilder {
    pub fn new(max_size: usize) -> Self {
        let mut s = Self {
            ordered_datagram_out: Default::default(),
            stream: OutOctetStream::new(),
            max_size,
        };

        s.clear();
        s
    }
}

impl DatagramBuilder for ClientDatagramBuilder {
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

    fn finalize(&mut self) -> &[u8] {
        // Finalize header

    }

    fn is_empty(&self) -> bool {
        self.stream.is_empty()
    }

    fn clear(&mut self) {
        self.stream.clear();

        prepare_out_stream(&self.stream)?; // Add hash stream
        self.ordered_datagram_out.to_stream(&self.stream)?;
        info!(
                    "add connect header {}",
                    self.ordered_datagram_out.sequence_to_send
                );

        let client_time = ClientTime::new(0);
        client_out_ping(client_time, &self.stream)
    }
}

/*
#[derive(PartialEq, Debug)]
enum ClientPhase {
    Connecting(Nonce),
    Connected(ConnectionId, ConnectionSecretSeed),
}
 */

fn write_header(&self, stream: &mut impl WriteOctetStream) -> io::Result<()> {
    match self.phase {
        ClientPhase::Connected(_, _) => {
            x
        }
        _ => {
            info!("oob zero connection");
            let zero_connection_id = ConnectionId { value: 0 }; // oob
            zero_connection_id.to_stream(stream) // OOB
        }
    }
}

fn write_to_start_of_header(
    &self,
    connection_id: ConnectionId,
    seed: ConnectionSecretSeed,
    out_stream: &mut OutOctetStream,
) -> io::Result<()> {
    let mut payload = out_stream.octets();
    let mut hash_stream = OutOctetStream::new();
    let payload_to_calculate_on = &payload[5..];
    info!("payload: {:?}", payload_to_calculate_on);
    write_to_stream(
        &mut hash_stream,
        connection_id,
        seed,
        payload_to_calculate_on,
    )?; // Write hash connection layer header
    payload[..hash_stream.octets().len()].copy_from_slice(hash_stream.octets_ref());
    Ok(())
}
