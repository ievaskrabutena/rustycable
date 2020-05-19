use crate::RustyCable;
use futures_util::StreamExt as _;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct RedisMessage {
    pub stream: String,
    pub data: serde_json::Value,
}

impl redis::FromRedisValue for RedisMessage {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<RedisMessage> {
        match *v {
            redis::Value::Data(ref value) => match serde_json::from_slice(value) {
                Ok(v) => Ok(v),
                Err(_) => {
                    Err(((redis::ErrorKind::TypeError, "Unable to parse given value")).into())
                }
            },
            _ => Err(((
                redis::ErrorKind::ResponseError,
                "Incorrect data received from Redis",
            ))
                .into()),
        }
    }
}

/// Connects to Redis server and subscribes to the appropriate channel
pub async fn start(app: Arc<RustyCable>) -> Result<(), Box<dyn std::error::Error>> {
    let client = redis::Client::open("redis://127.0.0.1/").expect("Failed to connect to redis");

    let mut pubsub_connection = client.get_async_connection().await?.into_pubsub();
    pubsub_connection.subscribe("__anycable__").await?;
    println!("Subscribed to Redis channel: __anycable__");

    let mut pubsub_stream = pubsub_connection.on_message();

    while let Some(msg) = pubsub_stream.next().await.unwrap().get_payload().unwrap() {
        tokio::spawn(app.clone().try_broadcast(msg));
    }

    Ok(())
}
