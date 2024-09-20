use log::info;
use secure_random::{GetRandom, SecureRandom};

#[test_log::test]
fn check_random() {
    let mut random = GetRandom;
    let result = random.get_random_u64();
    info!("result: {}", result)
}
