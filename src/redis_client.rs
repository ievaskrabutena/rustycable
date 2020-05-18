use crate::RustyCable;
use futures_util::StreamExt as _;
use std::sync::Arc;

/// Connects to Redis server and subscribes to the appropriate channel
pub async fn start(app: Arc<RustyCable>) -> Result<(), Box<dyn std::error::Error>> {
    let client = redis::Client::open("redis://127.0.0.1/").expect("Failed to connect to redis");

    let mut pubsub_conn = client.get_async_connection().await?.into_pubsub();
    pubsub_conn.subscribe("__anycable__").await?;
    println!("Subscribed to Redis channel: __anycable__");

    let mut pubsub_stream = pubsub_conn.on_message();

    while let Some(msg) = pubsub_stream
        .next()
        .await
        .unwrap()
        .get_payload::<Option<String>>()?
    {
        println!("{:?}", msg);
    }

    Ok(())
}
