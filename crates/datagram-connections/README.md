# Datagram Connections 🚀

[![Crates.io](https://img.shields.io/crates/v/datagram-connections)](https://crates.io/crates/datagram-connections)
[![Documentation](https://docs.rs/datagram-connections/badge.svg)](https://docs.rs/datagram-connections)

The `datagram-connections` crate implements a challenge-and-response protocol for securely acquiring connections 
over datagrams. It includes both a **client** and **host** part, making it easy to establish and maintain connections
with integrity and security.

## Features ✨

- **Challenge Protocol**: Secure your connections with a challenge-response handshake.
- **Client and Host Implementation**: Both sides of the connection are supported out of the box.
- **Efficient Datagram Handling**: Encode and decode datagrams with ease.
- **Secure Random Number Generation** 🎲: Leverage cryptographic random values for secure nonce generation.

## How it Works ⚙️

### Client Connection Flow 💻
1. **Challenge Phase**: The client initiates a challenge by sending a nonce.
2. **Connecting Phase**: After receiving the server's challenge response, the client sends a connect request.
3. **Connected Phase**: Once the server validates the request, the client is considered connected and can send/receive packets.

### Host Response Flow 🏠
1. **Challenge Response**: The host sends a challenge response with its own nonce.
2. **Connection Validation**: The host validates incoming connection requests using the nonce and establishes the connection.

## Get started

```toml
[dependencies]
datagram-connections = "0.0.2"
```
