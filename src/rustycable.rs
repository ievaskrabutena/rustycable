use crate::hub::Hub;
use crate::rpc_controller::RpcController;

/// The data structure representing the whole server
pub struct RustyCable {
    pub controller: RpcController,
    pub hub: Hub,
}

impl RustyCable {
    pub async fn new() -> Result<RustyCable, Box<dyn std::error::Error>> {
        let controller = RpcController::new().await?;

        Ok(RustyCable {
            controller,
            hub: Hub::new(),
        })
    }
}
