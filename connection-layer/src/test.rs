/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
#[cfg(test)]
mod tests {
    use super::*;
    use crate::murmur3_x86_32;

    #[test]
    fn test_murmur3_x86_32() {
        let test_vectors: Vec<(&[u8], u32, u32)> = vec![
            (&[], 0, 0x00000000),
            (&[], 1, 0x514E28B7),
            (&[], 0xffffffff, 0x81F16F39),
            (&[0xFF, 0xFF, 0xFF, 0xFF], 0, 0x76293B50),
            (&[0x21, 0x43, 0x65, 0x87], 0, 0xF55B516B),
            (&[0x21, 0x43, 0x65, 0x87], 0x5082EDEE, 0x2362F9DE),
            (&[0x21, 0x43, 0x65], 0, 0x7E4A8634),
            (&[0x21, 0x43], 0, 0xA0F7B07A),
            (&[0x21], 0, 0x72661CF4),
            (&[0x00, 0x00, 0x00, 0x00], 0, 0x2362F9DE),
            (&[0x00, 0x00, 0x00], 0, 0x85F0B427),
            (&[0x00, 0x00], 0, 0x30F4C306),
            (&[0x00], 0, 0x514E28B7),
        ];

        for (input, seed, expected) in test_vectors {
            let result = murmur3_x86_32(input, seed);
            assert_eq!(
                result, expected,
                "Failed for input: '{:?}', seed: {}, got: {:x}, expected: {:x}",
                input, seed, result, expected
            );
        }

        println!("All tests passed!");
    }
}
