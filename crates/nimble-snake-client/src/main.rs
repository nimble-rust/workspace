use monotonic_time_rs::MonotonicClock;
use nimble_snake_client::SnakeClient;
use std::thread;

fn main() {
    env_logger::init();

    let mut client = SnakeClient::new("127.0.0.1:23000");
    let clock = monotonic_time_rs::InstantMonotonicClock::new();
    loop {
        let result = client.client.update(clock.now());
        if let Err(err) = result {
            println!("{err}");
        }
        thread::sleep(std::time::Duration::from_millis(16));
    }
}
