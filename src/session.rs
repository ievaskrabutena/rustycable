use super::RustyCable;

use super::anycable::CommandResponse;
use chrono::Utc;
use futures_util::stream::{self, SplitSink, SplitStream, StreamExt};
use futures_util::SinkExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::{Mutex, RwLock};
use tokio_tungstenite::{
    tungstenite::{Message, Result as TungsteniteResult},
    WebSocketStream,
};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ClientMessageType {
    Subscribe,
    Unsubscribe,
    Message,
}

impl ToString for ClientMessageType {
    fn to_string(&self) -> String {
        let message_str = match self {
            ClientMessageType::Subscribe => "subscribe",
            ClientMessageType::Unsubscribe => "unsubscribe",
            ClientMessageType::Message => "message",
        };

        String::from(message_str)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ClientMessage {
    command: ClientMessageType,
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
    subscriptions: RwLock<HashMap<String, bool>>,
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
            subscriptions: RwLock::new(HashMap::new()),
            identifiers: response.identifiers,
        };

        session
            .process_transmissions(response.transmissions)
            .await
            .expect("Failure trying to respond to client");

        session
    }

    /// Waits for incoming messages from the client
    pub async fn read_messages(self: Arc<Self>) -> TungsteniteResult<()> {
        let mut reader = self.ws_reader.lock().await;

        while let Some(msg) = reader.next().await {
            let msg: Message = msg?;

            println!("[WS] - Received message from client: {:?}", msg);

            if msg.is_text() {
                let client_message: ClientMessage = serde_json::from_str(&msg.into_text()?)
                    .expect("Incorrect JSON received from client");

                tokio::spawn(self.clone().process_client_message(client_message));
            } else if msg.is_close() {
                self.closed.store(true, Ordering::Relaxed);
                self.ws_writer.lock().await.send(msg).await?;
            }
        }

        Ok(())
    }

    /// Sends client messages to the gRPC server
    async fn process_client_message(self: Arc<Self>, message: ClientMessage) {
        let response: Option<CommandResponse> = match message.command {
            ClientMessageType::Subscribe => self.clone().handle_subscription(message).await,
            ClientMessageType::Unsubscribe => self.clone().handle_unsub(message).await,
            ClientMessageType::Message => self.clone().handle_message(message).await,
        };

        if response.is_some() {
            self.process_transmissions(response.unwrap().transmissions)
                .await
                .expect("Failure trying to respond to client");
        }
    }

    async fn handle_subscription(
        self: Arc<Self>,
        message: ClientMessage,
    ) -> Option<CommandResponse> {
        if let Some(true) = self.subscriptions.read().await.get(&message.identifier) {
            println!("Already subscribed to {}", message.identifier);
            return None;
        }

        let identifier = message.identifier.clone();

        let response = self
            .clone()
            .send_command(message)
            .await
            .expect("Error while subscribing");

        self.subscriptions
            .write()
            .await
            .insert(identifier.clone(), true);

        response.streams.iter().for_each(|stream| {
            self.app
                .clone()
                .subscribe_session(self.clone(), stream.clone(), identifier.clone())
                .expect("unable to subscribe to stream")
        });

        Some(response)
    }

    async fn handle_unsub(self: Arc<Self>, message: ClientMessage) -> Option<CommandResponse> {
        match self.subscriptions.read().await.get(&message.identifier) {
            Some(false) | None => {
                println!("Unrecognized subscription: {}", message.identifier);
                return None;
            }
            _ => (),
        }

        let identifier = message.identifier.clone();

        let response = self
            .clone()
            .send_command(message)
            .await
            .expect("Error while unsubscribing");

        self.subscriptions.write().await.remove(&identifier);

        Some(response)
    }

    async fn handle_message(self: Arc<Self>, message: ClientMessage) -> Option<CommandResponse> {
        match self.subscriptions.read().await.get(&message.identifier) {
            Some(false) | None => {
                println!("Unrecognized subscription: {}", message.identifier);
                return None;
            }
            _ => (),
        }

        let response = self
            .clone()
            .send_command(message)
            .await
            .expect("Error while sending message");

        Some(response)
    }

    async fn send_command(
        self: Arc<Self>,
        message: ClientMessage,
    ) -> Result<CommandResponse, Box<dyn std::error::Error>> {
        let response = self
            .app
            .controller
            .send_command(
                message.command.to_string(),
                message.identifier,
                self.identifiers.clone(),
                message.data.unwrap_or(String::from("")),
            )
            .await?;

        println!("[gRPC response] - {:?}", response);

        Ok(response)
    }

    /// iterate trough a vector of messages and send each of them to client
    async fn process_transmissions(
        &self,
        transmissions: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = self.ws_writer.lock().await;

        let mut future = stream::iter(transmissions);

        while let Some(transmission) = future.next().await {
            writer.send(Message::text(transmission)).await?;
        }

        Ok(())
    }

    pub async fn send_broadcast(
        self: Arc<Self>,
        data: &str,
        identifier: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.closed.load(Ordering::Relaxed) {
            return Ok(());
        }

        let json_data: serde_json::Value = serde_json::from_str(data).unwrap();

        let mut writer = self.ws_writer.lock().await;
        let message = serde_json::to_string(
            &serde_json::json!({ "identifier": identifier, "message": json_data }),
        )
        .expect("unable to convert JSON");

        writer.send(Message::text(message)).await?;

        Ok(())
    }

    /// Schedules session to ping the client every `PING_INTERVAL` seconds while connection is open
    pub async fn schedule_ping(self: Arc<Self>) -> TungsteniteResult<()> {
        let mut interval = tokio::time::interval(PING_INTERVAL);

        while let Some(_) = interval.next().await {
            if self.closed.load(Ordering::Relaxed) {
                return Ok(());
            }

            tokio::spawn(self.clone().send_ping());
        }

        Ok(())
    }

    /// Creates and sends a ping message
    pub async fn send_ping(self: Arc<Self>) -> TungsteniteResult<()> {
        let mut writer = self.ws_writer.lock().await;

        let hen = serde_json::to_string(
            &serde_json::json!({ "type": "ping", "message": Utc::now().timestamp()}),
        )
        .expect("unable to convert JSON");

        writer.send(hen.into()).await?;

        Ok(())
    }
}
