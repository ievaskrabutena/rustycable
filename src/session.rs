use super::RustyCable;

use chrono::Utc;
use futures_util::stream::{self, SplitSink, SplitStream, StreamExt};
use futures_util::SinkExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{
    tungstenite::{Message, Result as TungsteniteResult},
    WebSocketStream,
};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct ClientMessage {
    command: String,
    identifier: String,
    data: Option<String>,
}

/// The duration between pings to client
const PING_INTERVAL: Duration = Duration::from_secs(3);

/// Fetch an UID from `X-Request-ID` header or create one for a session
fn fetch_or_create_uid(headers: &HashMap<String, String>) -> String {
    headers
        .get("X-Request-ID")
        .unwrap_or(&Uuid::new_v4().to_string())
        .into()
}

/// Session represents an active WebSocket connection with a client
pub struct Session {
    app: Arc<RustyCable>,
    subscriptions: HashMap<String, bool>,
    ws_reader: Mutex<SplitStream<WebSocketStream<TcpStream>>>,
    ws_writer: Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>,
    closed: AtomicBool,

    pub uid: String,
    pub identifiers: String,
}

impl Session {
    /// Authentificate the WebSocket stream through gRPC server and create a session
    pub async fn new(
        app: Arc<RustyCable>,
        headers: HashMap<String, String>,
        uri: String,
        ws_stream: WebSocketStream<TcpStream>,
    ) -> Session {
        let uid = fetch_or_create_uid(&headers);

        let response = app
            .controller
            .connect(headers, uri)
            .await
            .expect("Failed gRPC connection");

        println!("[gRPC response] - {:?}", response);

        let (ws_writer, ws_reader) = ws_stream.split();

        let session = Session {
            app,
            uid,
            ws_reader: Mutex::new(ws_reader),
            ws_writer: Mutex::new(ws_writer),
            closed: AtomicBool::new(false),
            subscriptions: HashMap::new(),
            identifiers: response.identifiers,
        };

        session
            .process_transmissions(response.transmissions)
            .await
            .expect("Failure trying to respond to client");

        session
    }

    /// Setup session to process messages received from the client
    pub async fn read_messages(&self) -> TungsteniteResult<()> {
        let mut reader = self.ws_reader.lock().await;

        while let Some(msg) = reader.next().await {
            let msg: Message = msg?;

            println!("[WS] - Received message from client: {:?}", msg);

            if msg.is_text() {
                let client_message: ClientMessage = serde_json::from_str(&msg.into_text()?)
                    .expect("Incorrect JSON received from client");
                println!("{:?}", client_message);
                let response = self
                    .app
                    .controller
                    .send_command(
                        client_message.command,
                        client_message.identifier,
                        self.identifiers.clone(),
                        client_message.data.unwrap_or(String::from("")),
                    )
                    .await
                    .expect("Failed gRPC connection");

                println!("[gRPC response] - {:?}", response);

                self.process_transmissions(response.transmissions)
                    .await
                    .expect("Failure trying to respond to client");
            } else if msg.is_close() {
                self.closed.store(true, Ordering::Relaxed);
            }
        }

        Ok(())
    }

    /// iterate trough a vector of messages and send each of them to client
    async fn process_transmissions(
        &self,
        transmissions: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = self.ws_writer.lock().await;

        let mut future = stream::iter(transmissions);

        for transmission in future.next().await {
            writer.send(Message::text(transmission)).await?;
        }

        Ok(())
    }

    /// Setup session to ping the client every `PING_INTERVAL` seconds while connection is open
    pub async fn send_ping(&self) -> TungsteniteResult<()> {
        let mut interval = tokio::time::interval(PING_INTERVAL);

        while let Some(_) = interval.next().await {
            if self.closed.load(Ordering::Relaxed) {
                return Ok(());
            }

            let mut writer = self.ws_writer.lock().await;

            let hen = serde_json::to_string(
                &serde_json::json!({ "type": "ping", "message": Utc::now().timestamp()}),
            )
            .expect("unable to convert JSON");

            writer.send(hen.into()).await?;
        }

        Ok(())
    }
}
