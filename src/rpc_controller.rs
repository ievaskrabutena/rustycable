pub use super::anycable::rpc_client::RpcClient;
use super::anycable::{
    CommandMessage, CommandResponse, ConnectionRequest, ConnectionResponse, DisconnectRequest,
    DisconnectResponse,
};
use std::collections::HashMap;
use tonic::transport::Channel;

/// The data structure that interacts with the anycable gRPC server as a client
pub struct RpcController {
    client: RpcClient<Channel>,
}

impl RpcController {
    /// Creates a connection to the Anycable gRPC server
    pub async fn new() -> Result<RpcController, Box<dyn std::error::Error>> {
        let client = RpcClient::connect("http://[::1]:50051").await?;

        Ok(RpcController { client })
    }

    /// Sends a `Connect` message to gRPC server
    pub async fn connect(
        &self,
        headers: HashMap<String, String>,
        uri: String,
    ) -> Result<ConnectionResponse, Box<dyn std::error::Error>> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(ConnectionRequest { headers, path: uri });

        let response = client.connect_client(request).await?;
        Ok(response.into_inner())
    }

    /// Sends a `Command` message to gRPC server
    pub async fn send_command(
        &self,
        command: String,
        identifier: String,
        connection_identifiers: String,
        data: String,
    ) -> Result<CommandResponse, Box<dyn std::error::Error>> {
        let mut client = self.client.clone();
        let message = CommandMessage {
            command,
            identifier,
            connection_identifiers,
            data: data,
        };

        let response = client.command(message).await?;
        Ok(response.into_inner())
    }
}
