use super::hub::Hub;
use super::rpc_controller::RpcController;
use super::session::Session;

use std::collections::HashMap;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    accept_hdr_async,
    tungstenite::handshake::server::{ErrorResponse, Request, Response},
    tungstenite::Error,
    tungstenite::Result as TungsteniteResult,
};

pub struct Server {
    pub controller: RpcController,
    hub: Hub,
}

impl Server {
    pub fn new(controller: RpcController) -> Server {
        Server {
            controller,
            hub: Hub::new(),
        }
    }
}

pub async fn start(controller: RpcController) -> Result<(), Box<dyn std::error::Error>> {
    let server: Arc<Server> = Arc::new(Server::new(controller));

    let addr = String::from("127.0.0.1:8081");

    let try_socket = TcpListener::bind(&addr).await;
    let mut listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream
            .peer_addr()
            .expect("connected streams should have a peer address");

        tokio::spawn(accept_connection(peer, stream, server.clone()));
    }

    Ok(())
}

async fn accept_connection(peer: SocketAddr, stream: TcpStream, server: Arc<Server>) {
    if let Err(e) = handle_connection(peer, stream, server).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => eprintln!("Error processing connection: {}", err),
        }
    }
}

async fn handle_connection(
    peer: SocketAddr,
    stream: TcpStream,
    server: Arc<Server>,
) -> TungsteniteResult<()> {
    let mut headers: HashMap<String, String> = HashMap::new();
    let mut uri = String::new();

    let ws_stream = accept_hdr_async(
        stream,
        |request: &Request, response: Response| -> Result<Response, ErrorResponse> {
            uri = request.uri().path().to_string();

            request
                .headers()
                .iter()
                .for_each(|(header_key, header_value)| {
                    headers.insert(
                        header_key.as_str().to_string(),
                        header_value.to_str().unwrap().to_string(),
                    );
                });

            Ok(response)
        },
    )
    .await
    .expect("Failed to accept");

    let session = Session::new(server.clone(), headers, uri, Mutex::new(ws_stream)).await;

    tokio::try_join!(session.read_messages(), session.send_ping())?;

    Ok(())
}
