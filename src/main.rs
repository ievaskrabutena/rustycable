mod anycable;
mod hub;
mod redis_client;
mod rpc_controller;
mod rustycable;
mod server;
mod session;

use rustycable::RustyCable;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Arc::new(RustyCable::new().await?);

    tokio::try_join!(server::start(app.clone()), redis_client::start(app.clone()))
        .expect("Redis failure");

    Ok(())
}
