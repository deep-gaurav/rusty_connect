use std::sync::Arc;

use async_graphql::{SimpleObject, Subscription};
use async_stream::stream;
use futures::{Stream, StreamExt};
use tokio::sync::RwLock;
use tracing::debug;

use crate::{
    devices::DeviceManager,
    plugins::{share::DownloadProgress, Connected, Disconnected, PluginManager, ReceivedPayload},
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
        let stream = stream! {
            debug!("Listening for new payload from channel");
            while let Ok((device_id,payload)) = receiver.recv_async().await {
                debug!("Received payload from channel");
                yield ReceivedMessage { device_id, payload }
            }
            debug!("Stopped listening for payload")
        };
        stream
    }

    async fn download_update(
        &self,
        download_id: String,
    ) -> anyhow::Result<impl Stream<Item = DownloadProgress>> {
        let mut task_rx = {
            let dm = self.device_manager.read().await;
            let tasks = dm.download_tasks.lock().await;
            let task = tasks
                .get(&download_id)
                .ok_or(anyhow::anyhow!("No download task with given id"))?;
            task.clone()
        };
        let stream = stream! {
            loop {
                let val = {task_rx.borrow_and_update().clone()};
                yield val;
                if task_rx.changed().await.is_err() {
                    break;
                }
            }
        };

        Ok(stream)
    }
}

#[derive(SimpleObject)]
pub struct ReceivedMessage {
    pub device_id: String,
    pub payload: ReceivedPayload,
}
