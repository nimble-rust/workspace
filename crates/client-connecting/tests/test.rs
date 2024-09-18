use nimble_client_connecting::{ClientError, ConnectingClient};
use nimble_protocol::prelude::*;
use test_log::test;

fn create_connecting_client(
    application_version: Option<Version>,
    nimble_version: Option<Version>,
) -> ConnectingClient {
    let app_version = application_version.unwrap_or(Version { major: 1, minor: 0, patch: 0 });
    let nimble_ver = nimble_version.unwrap_or(Version { major: 0, minor: 0, patch: 5 });
    ConnectingClient::new(Nonce(42), app_version, nimble_ver)
}


#[test]
fn test_send_connect_command() {
    let mut client = create_connecting_client(None, None);
    let commands = client.send();

    let ClientToHostOobCommands::ConnectType(connect_cmd) = &commands;
    assert_eq!(connect_cmd.application_version, Version { major: 1, minor: 0, patch: 0 });
    assert_eq!(connect_cmd.nimble_version, Version { major: 0, minor: 0, patch: 5 });
    assert_eq!(connect_cmd.use_debug_stream, false);
    assert_eq!(connect_cmd.nonce, client.debug_nonce());
}

#[test]
fn test_receive_valid_connection_accepted() {
    let mut client = create_connecting_client(None, None);
    let response_nonce = client.debug_nonce();
    let connection_secret = SessionConnectionSecret { value: 12345 };
    let connection_id = SessionConnectionId(101);

    let accepted = ConnectionAccepted {
        flags: 0,
        response_to_nonce: response_nonce,
        host_assigned_connection_id: connection_id,
        host_assigned_connection_secret: connection_secret.clone(),
    };
    let command = HostToClientOobCommands::ConnectType(accepted);
    let result = client.receive(command);

    assert!(result.is_ok());
    let connected_info = client.connected_info().expect("should be set by this time");

    assert_eq!(connected_info.session_connection_secret, connection_secret);
    assert_eq!(connected_info.connection_id, connection_id);
}

#[test]
fn test_receive_invalid_connection_accepted_nonce() {
    let mut client = create_connecting_client(None, None);
    let wrong_nonce = Nonce(999);
    let connection_secret = SessionConnectionSecret { value: 12345 };
    let accepted = ConnectionAccepted {
        flags: 0,
        response_to_nonce: wrong_nonce,
        host_assigned_connection_id: SessionConnectionId(99),
        host_assigned_connection_secret: connection_secret,
    };
    let command = HostToClientOobCommands::ConnectType(accepted);
    let result = client.receive(command);

    match result {
        Err(ClientError::WrongConnectResponseNonce(n)) => {
            assert_eq!(n, wrong_nonce);
        }
        _ => panic!("Expected WrongConnectResponseNonce error"),
    }
}
