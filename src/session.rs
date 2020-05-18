use super::RustyCable;

use chrono::Utc;
use futures_util::SinkExt;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::stream::StreamExt;
use tokio::sync::Mutex;
use tokio_tungstenite::{
    tungstenite::{Message, Result as TungsteniteResult},
    WebSocketStream,
};
use uuid::Uuid;

const PING_INTERVAL: Duration = Duration::from_secs(3);

fn fetch_or_create_uid(headers: &HashMap<String, String>) -> String {
    headers
        .get("X-Request-ID")
        .unwrap_or(&Uuid::new_v4().to_string())
        .into()
}

/// Session represents an active client
pub struct Session {
    subscriptions: HashMap<String, bool>,
    ws_stream: Mutex<WebSocketStream<TcpStream>>,
    closed: AtomicBool,

    pub uid: String,
    pub identifiers: String,
}

impl Session {
    pub async fn new(
        app: Arc<RustyCable>,
        headers: HashMap<String, String>,
        uri: String,
        ws_stream: Mutex<WebSocketStream<TcpStream>>,
    ) -> Session {
        let uid = fetch_or_create_uid(&headers);

        let response = app
            .controller
            .connect(headers, uri)
            .await
            .expect("Failed gRPC connection");

        println!("gRPC server responded with {:?}", response);

        let hen = serde_json::to_vec(&serde_json::json!({ "type": "welcome" }))
            .expect("unable to convert JSON");

        {
            let mut ws = ws_stream.lock().await;

            ws.send(hen.into())
                .await
                .expect("Error when trying to send response to client");
        }

        Session {
            uid,
            ws_stream: ws_stream,
            closed: AtomicBool::new(false),
            subscriptions: HashMap::new(),
            identifiers: String::new(),
        }
    }

    pub async fn read_messages(&self) -> TungsteniteResult<()> {
        let mut ws = self.ws_stream.lock().await;

        while let Some(msg) = ws.next().await {
            let msg: Message = msg?;

            if msg.is_text() || msg.is_binary() {
                ws.send(msg.into()).await?;
            } else if msg.is_close() {
                self.closed.store(true, Ordering::Relaxed);
                ws.close(None).await?;
            }
        }

        Ok(())
    }

    pub async fn send_ping(&self) -> TungsteniteResult<()> {
        let mut interval = tokio::time::interval(PING_INTERVAL);

        while let Some(_) = interval.next().await {
            if self.closed.load(Ordering::Relaxed) {
                return Ok(());
            }

            let mut ws = self.ws_stream.lock().await;

            let hen = serde_json::to_vec(
                &serde_json::json!({ "type": "ping", "message": Utc::now().timestamp().to_string()}),
            )
            .expect("unable to convert JSON");

            ws.send(hen.into()).await?;
        }

        Ok(())
    }
}
