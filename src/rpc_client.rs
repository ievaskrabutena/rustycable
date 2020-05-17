use super::anycable::rpc_client::RpcClient;
use super::anycable::{
    CommandMessage, CommandResponse, ConnectionRequest, ConnectionResponse, DisconnectRequest,
    DisconnectResponse,
};
use std::collections::HashMap;

pub async fn connect() -> Result<ConnectionResponse, Box<dyn std::error::Error>> {
    let mut client = RpcClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(ConnectionRequest {
        headers: HashMap::new(),
        path: String::from(""),
    });

    let response = client.connect_client(request).await?;

    Ok(response.into_inner())
}

pub async fn disconnect() -> Result<DisconnectResponse, Box<dyn std::error::Error>> {
    let mut client = RpcClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(DisconnectRequest {
        identifiers: String::from(""),
        subscriptions: Vec::new(),
        headers: HashMap::new(),
        path: String::from(""),
    });

    let response = client.disconnect(request).await?;

    Ok(response.into_inner())
}

pub async fn subscribe() -> Result<CommandResponse, Box<dyn std::error::Error>> {
    let mut client = RpcClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(CommandMessage {
        command: String::from("subscribe"),
        identifier: String::from(""),
        connection_identifiers: String::from(""),
        data: String::from(""),
    });

    let response = client.command(request).await?;

    Ok(response.into_inner())
}

pub async fn unsubscribe() -> Result<CommandResponse, Box<dyn std::error::Error>> {
    let mut client = RpcClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(CommandMessage {
        command: String::from("unsubscribe"),
        identifier: String::from(""),
        connection_identifiers: String::from(""),
        data: String::from(""),
    });

    let response = client.command(request).await?;

    Ok(response.into_inner())
}

pub async fn perform() -> Result<CommandResponse, Box<dyn std::error::Error>> {
    let mut client = RpcClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(CommandMessage {
        command: String::from("message"),
        identifier: String::from(""),
        connection_identifiers: String::from(""),
        data: String::from(""),
    });

    let response = client.command(request).await?;

    Ok(response.into_inner())
}
