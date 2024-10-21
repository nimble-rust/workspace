use nimble_client_with_codec::ClientWithCodec;
use nimble_rust::{SampleGame, SampleStep};

#[test]
fn test_client_with_codec() {
    let x = ClientWithCodec::<SampleGame, SampleStep>::new("127.0.0.1:22000");

    assert!(x.client.game().is_none())
}
