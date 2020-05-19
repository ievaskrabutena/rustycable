use super::session::Session;
use super::RustyCable;

use http::header::HeaderValue;
use std::collections::HashMap;
use std::{net::SocketAddr, sync::Arc};

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    accept_hdr_async,
    tungstenite::handshake::server::{ErrorResponse, Request, Response},
    tungstenite::Error,
    tungstenite::Result as TungsteniteResult,
};

/// Starts the WebSocket server
pub async fn start(app: Arc<RustyCable>) -> Result<(), Box<dyn std::error::Error>> {
    let addr = String::from("127.0.0.1:8081");

    let try_socket = TcpListener::bind(&addr).await;
    let mut listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream
            .peer_addr()
            .expect("connected streams should have a peer address");

        tokio::spawn(accept_connection(peer, stream, app.clone()));
    }

    Ok(())
}

/// Handles the potential errors of a WebSocket connection
async fn accept_connection(peer: SocketAddr, stream: TcpStream, app: Arc<RustyCable>) {
    if let Err(e) = handle_connection(peer, stream, app).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => eprintln!("Error processing connection: {}", err),
        }
    }
}

/// Makes a handshake with the WebSocket request, then creates a session
async fn handle_connection(
    peer: SocketAddr,
    stream: TcpStream,
    app: Arc<RustyCable>,
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

            let (mut parts, body) = response.into_parts();

            parts.headers.insert(
                "Sec-Websocket-Protocol",
                HeaderValue::from_str("actioncable-v1-json").unwrap(),
            );

            let response_with_protocol = Response::from_parts(parts, body);

            Ok(response_with_protocol)
        },
    )
    .await
    .expect("Failed to accept");

    let session = Arc::new(Session::new(app.clone(), headers, uri, ws_stream).await);

    tokio::try_join!(
        session.clone().read_messages(),
        session.clone().schedule_ping()
    )?;

    Ok(())
}
