/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
//! # hexify
//!
//! `hexify` is a Rust library for formatting byte slices (`[u8]`) into hexadecimal (`hex`) strings.
//! It provides utilities to convert bytes into hex strings.
//!
//! ## Installation
//!
//! Add `hexify` to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! hexify = "0.0.1"
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use hexify::{format_hex};
//!
//! fn main() {
//!     let data = [0x42, 0xA4, 0xAE, 0x09, 0xAF, 0x00, 0x01, 0x00, 0x00, 0x04, 0x03, 0x00, 0x00];
//!
//!     let output = format_hex(&data);
//!     println!("{}", output); // Outputs: 42 A4 AE 09 AF 00 01 00 00 04 03 00 00
//! }
//! ```

/// Formats a byte slice into an uppercase, space-separated hexadecimal string.
///
/// This is a convenience function that formats the input bytes into an uppercase
/// hexadecimal string with spaces separating each byte.
///
/// # Parameters
///
/// - `bytes`: A slice of bytes (`[u8]`) to format.
///
/// # Returns
///
/// A `String` containing the uppercase, space-separated hexadecimal representation.
///
/// # Examples
///
/// ```rust
/// use hexify::format_hex;
///
/// let data = [0x42, 0xA4, 0xAE];
/// let hex = format_hex(&data);
/// assert_eq!(hex, "42 A4 AE");
/// ```
pub fn format_hex(bytes: &[u8]) -> String {
    format_hex_with_prefix_and_separator(bytes, "", " ")
}

pub fn format_hex_with_prefix_and_separator(bytes: &[u8], prefix: &str, separator: &str) -> String {
    bytes
        .iter()
        .map(|b| format!("{prefix}{:02X}", b))
        .collect::<Vec<String>>()
        .join(separator)
}

pub fn format_hex_u32_be(num: u32) -> String {
    format_hex_with_prefix_and_separator(&num.to_be_bytes(), "0x", ",")
}
