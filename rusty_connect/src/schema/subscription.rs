use std::sync::Arc;

use async_graphql::{SimpleObject, Subscription};
use async_stream::stream;
use futures::{Stream, StreamExt};
use tokio::sync::RwLock;
use tracing::debug;

use crate::{
    devices::DeviceManager,
    plugins::{Connected, Disconnected, PluginManager, ReceivedPayload},
};

pub struct Subscription {
    pub plugin_manager: Arc<PluginManager>,
    pub device_manager: Arc<RwLock<DeviceManager>>,
}

#[Subscription]
impl Subscription {
    async fn payloads(&self) -> impl Stream<Item = ReceivedMessage> {
        debug!("Requesting receiver");
        let receiver = {
            let dm = self.device_manager.read().await;
            dm.receiver.clone()
        };
        debug!("Got Receiver");

        let plugin_manager = { self.plugin_manager.clone() };

        let stream = stream! {
            debug!("Listening for new payload from channel");
            while let Ok((device_id,payload)) = receiver.recv_async().await {
                match payload {
                    crate::payloads::RustyPayload::Connected => yield ReceivedMessage {
                        device_id:device_id.clone(),
                        payload:ReceivedPayload::Connected(
                            Connected{
                                id:device_id
                            }
                        )
                    },
                    crate::payloads::RustyPayload::Disconnect => yield ReceivedMessage {
                        device_id:device_id.clone(),
                        payload:ReceivedPayload::Disconnected(
                            Disconnected{
                                id:device_id
                            }
                        )
                    },
                    crate::payloads::RustyPayload::KDEConnectPayload(payload) => if let Ok(payload) = plugin_manager.parse_payload(payload){
                        yield ReceivedMessage { device_id, payload }
                    },
                }
            }
            debug!("Stopped listening for payload")
        };
        stream
    }
}

#[derive(SimpleObject)]
pub struct ReceivedMessage {
    pub device_id: String,
    pub payload: ReceivedPayload,
}
