use std::sync::Arc;

use async_graphql::{SimpleObject, Subscription};
use async_stream::stream;
use futures::{Stream, StreamExt};
use tokio::sync::RwLock;
use tracing::debug;

use crate::{
    devices::DeviceManager,
    payloads::PayloadType,
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
        let device_manager = self.device_manager.clone();

        let stream = stream! {
            debug!("Listening for new payload from channel");
            while let Ok(payload_type) = receiver.recv_async().await {
                match payload_type {
                    PayloadType::Broadcast(payload) => {
                        if let Ok(payload) = plugin_manager.parse_payload(payload, None){
                            yield ReceivedMessage { device_id:None, payload }
                        }
                    }
                    PayloadType::ConnectionPayload(device_id,payload) => {
                        let mut dm =  device_manager.write().await;
                        let device =  dm.devices.get_mut(&device_id);

                        if let Some(device) = device {

                            match payload {
                                crate::payloads::RustyPayload::Connected => yield ReceivedMessage {
                                    device_id:Some(device_id.clone()),
                                    payload:ReceivedPayload::Connected(
                                        Connected{
                                            id:device_id
                                        }
                                    )
                                },
                                crate::payloads::RustyPayload::Disconnect => yield ReceivedMessage {
                                    device_id:Some(device_id.clone()),
                                    payload:ReceivedPayload::Disconnected(
                                        Disconnected{
                                            id:device_id
                                        }
                                    )
                                },
                                crate::payloads::RustyPayload::KDEConnectPayload(payload) => {
                                    if let Ok(payload) = plugin_manager.parse_payload(payload,Some(&mut device.device)){
                                        yield ReceivedMessage { device_id:Some(device_id), payload }
                                    }
                                },
                            }
                        }
                    }
                }
            }
            debug!("Stopped listening for payload")
        };
        stream
    }
}

#[derive(SimpleObject)]
pub struct ReceivedMessage {
    pub device_id: Option<String>,
    pub payload: ReceivedPayload,
}
