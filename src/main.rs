mod anycable;
mod hub;
mod redis_client;
mod rpc_controller;
mod server;
mod session;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let controller = rpc_controller::RpcController::new().await?;

    tokio::try_join!(server::start(controller), redis_client::start()).expect("Redis failure");

    Ok(())
}
