use getrandom::getrandom;

pub fn get_random_u64() -> u64 {
    let mut buf = [0u8; 8];  // Create a buffer for 8 bytes
    getrandom(&mut buf).expect("Failed to get random bytes");  // Fill buffer with random bytes

    // Convert bytes to u64
    u64::from_le_bytes(buf)
}


#[cfg(test)]
mod tests {
    use crate::get_random_u64;

    #[test]
    fn check_random() {
        let result = get_random_u64();
        println!("result: {}", result)
    }
}
