use nimble_protocol::prelude::*;

#[derive(Debug)]
pub enum ClientError {
    WrongConnectResponseNonce(Nonce),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ConnectedInfo {
    pub session_connection_secret: SessionConnectionSecret,
    pub connection_id: SessionConnectionId,
}

#[derive(Debug, PartialEq)]
pub struct ConnectingClient {
    nonce: Nonce,
    application_version: Version,
    nimble_version: Version,
    connected_info: Option<ConnectedInfo>,
}

/*

 */

impl ConnectingClient {
    #[must_use]
    pub const fn new(nonce: Nonce, application_version: Version, nimble_version: Version) -> Self {
        Self {
            application_version,
            nimble_version,
            nonce,
            connected_info: None,
        }
    }

    #[must_use]
    pub fn send(&mut self) -> ClientToHostOobCommands {
        let connect_cmd = ConnectRequest {
            nimble_version: self.nimble_version,
            use_debug_stream: false,
            application_version: self.application_version,
            nonce: self.nonce,
        };

        ClientToHostOobCommands::ConnectType(connect_cmd)
    }

    fn on_connect(&mut self, cmd: ConnectionAccepted) -> Result<(), ClientError> {
        if cmd.response_to_nonce != self.nonce {
            Err(ClientError::WrongConnectResponseNonce(cmd.response_to_nonce))?
        }
        self.connected_info = Some(ConnectedInfo {
            session_connection_secret: cmd.host_assigned_connection_secret,
            connection_id: cmd.host_assigned_connection_id,
        });
        //info!("connected: session_secret: {:?}", self.connected_info.unwrap());
        Ok(())
    }

    pub fn receive(&mut self, command: HostToClientOobCommands) -> Result<(), ClientError> {
        match command {
            HostToClientOobCommands::ConnectType(connect_command) => {
                self.on_connect(connect_command)
            }
        }
    }

    pub fn debug_nonce(&self) -> Nonce {
        self.nonce
    }

    pub fn connected_info(&self) -> &Option<ConnectedInfo> {
        &self.connected_info
    }
}
