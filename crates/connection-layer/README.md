# Connection Layer Codec for Rust Datagram Transports

## Overview

This Rust library provides a codec layer over datagrams (typically over UDP) to assign
connection IDs (u8) and use a Murmur3 hash to verify incoming datagrams.
It enables you to implement connection-oriented communication over a connectionless transports,
allowing for simple connection management and data integrity verification.

## Features

* Connection Management: Assigns unique connection IDs to clients, enabling the host to manage multiple connections.
* Data Integrity: Uses Murmur3 hashing with a seed to verify the integrity of incoming datagrams.
* Simple API: Provides easy-to-use encoders and decoders for both host and client sides.

## Installation

Add the following to your Cargo.toml:

```toml
[dependencies]
connection-layer = "^0.0.1"
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are not accepted. This project is exclusively maintained by the author to retain full control and
copyrights.
