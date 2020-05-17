pub use super::anycable::rpc_client::RpcClient;
use super::anycable::{
    CommandMessage, CommandResponse, ConnectionRequest, ConnectionResponse, DisconnectRequest,
    DisconnectResponse,
};
use std::collections::HashMap;
use tonic::transport::Channel;

pub struct RpcController {
    client: RpcClient<Channel>,
}

impl RpcController {
    pub async fn new() -> Result<RpcController, Box<dyn std::error::Error>> {
        let client = RpcClient::connect("http://[::1]:50051").await?;

        Ok(RpcController { client })
    }

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
}
