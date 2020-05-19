use crate::hub::{Hub, HubAction};
use crate::redis_client::RedisMessage;
use crate::rpc_controller::RpcController;
use crate::session::Session;

use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

/// The data structure representing the whole server
pub struct RustyCable {
    pub controller: RpcController,
    pub hub: UnboundedSender<HubAction>,
}

impl RustyCable {
    pub async fn new() -> Result<RustyCable, Box<dyn std::error::Error>> {
        let controller = RpcController::new().await?;

        Ok(RustyCable {
            controller,
            hub: Hub::new(),
        })
    }

    pub fn add_session(
        self: Arc<Self>,
        session: Arc<Session>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.hub.send(HubAction::Add { session });
        Ok(())
    }

    pub fn subscribe_session(
        self: Arc<Self>,
        session: Arc<Session>,
        stream: String,
        identifier: String,
    ) -> Result<(), Box<std::error::Error>> {
        self.hub.send(HubAction::Subscribe {
            session_id: session.uid.clone(),
            stream,
            identifier,
        });
        Ok(())
    }

    pub async fn try_broadcast(self: Arc<Self>, message: RedisMessage) -> tokio::io::Result<()> {
        self.hub.send(HubAction::Broadcast {
            stream: message.stream,
            data: message.data,
        });
        Ok(())
    }
}
