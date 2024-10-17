/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
//! This module provides a prelude with the most commonly used types from the crate.
//!
//! By importing this prelude, you gain easy access to the core types and traits
//! that are frequently used throughout the crate. This reduces the boilerplate
//! needed in user code.
pub use crate::{client_codec::ConnectionLayerClientCodec, host_codec::{
    ConnectionLayerHostCodec,
    DatagramHostDecoder,
    DatagramHostEncoder,
}, ConnectionId, ConnectionLayer,
                ConnectionLayerMode,
                RequestId,
};
