use nimble_client_connecting::{ClientError, ConnectingClient};
use nimble_protocol::prelude::*;
use nimble_protocol::ClientRequestId;
use test_log::test;

fn create_connecting_client(
    application_version: Option<Version>,
    nimble_version: Option<Version>,
) -> ConnectingClient {
    let app_version = application_version.unwrap_or(Version {
        major: 1,
        minor: 0,
        patch: 0,
    });
    let nimble_ver = nimble_version.unwrap_or(Version {
        major: 0,
        minor: 0,
        patch: 5,
    });
    ConnectingClient::new(ClientRequestId(42), app_version, nimble_ver)
}

#[test]
fn test_send_connect_command() {
    let mut client = create_connecting_client(None, None);
    let commands = client.send();

    let ClientToHostOobCommands::ConnectType(connect_cmd) = &commands;
    assert_eq!(
        connect_cmd.application_version,
        Version {
            major: 1,
            minor: 0,
            patch: 0
        }
    );
    assert_eq!(
        connect_cmd.nimble_version,
        Version {
            major: 0,
            minor: 0,
            patch: 5
        }
    );
    assert_eq!(connect_cmd.use_debug_stream, false);
    assert_eq!(
        connect_cmd.client_request_id,
        client.debug_client_request_id()
    );
}

#[test]
fn receive_valid_connection_accepted() {
    let mut client = create_connecting_client(None, None);
    let response_nonce = client.debug_client_request_id();
    let connection_secret = SessionConnectionSecret { value: 12345 };
    let connection_id = SessionConnectionId(101);

    let accepted = ConnectionAccepted {
        flags: 0,
        response_to_request: response_nonce,
        host_assigned_connection_id: connection_id,
        host_assigned_connection_secret: connection_secret.clone(),
    };
    let command = HostToClientOobCommands::ConnectType(accepted);

    let _ = client.send(); // Just make it send once so it can try to accept the connection accepted

    let result = client.receive(&command);

    assert!(result.is_ok());
    let connected_info = client.connected_info().expect("should be set by this time");

    assert_eq!(connected_info.session_connection_secret, connection_secret);
    assert_eq!(connected_info.connection_id, connection_id);
}

#[test]
fn receive_invalid_connection_accepted_nonce() {
    let mut client = create_connecting_client(None, None);
    let wrong_request_id = ClientRequestId(999);
    let connection_secret = SessionConnectionSecret { value: 12345 };
    let accepted = ConnectionAccepted {
        flags: 0,
        response_to_request: wrong_request_id,
        host_assigned_connection_id: SessionConnectionId(99),
        host_assigned_connection_secret: connection_secret,
    };
    let command = HostToClientOobCommands::ConnectType(accepted);

    let _ = client.send(); // Just make it send once so it can try to accept the connection accepted

    let result = client.receive(&command);

    match result {
        Err(ClientError::WrongConnectResponseRequestId(n)) => {
            assert_eq!(n, wrong_request_id);
        }
        _ => panic!("Expected WrongConnectResponseNonce error"),
    }
}

#[test]
fn receive_response_without_request() {
    let mut client = create_connecting_client(None, None);
    let wrong_request_id = ClientRequestId(999);
    let connection_secret = SessionConnectionSecret { value: 12345 };
    let accepted = ConnectionAccepted {
        flags: 0,
        response_to_request: wrong_request_id,
        host_assigned_connection_id: SessionConnectionId(99),
        host_assigned_connection_secret: connection_secret,
    };
    let command = HostToClientOobCommands::ConnectType(accepted);

    let result = client.receive(&command);

    match result {
        Err(ClientError::ReceivedConnectResponseWithoutRequest) => {}
        _ => panic!("Expected WrongConnectResponseNonce error"),
    }
}
