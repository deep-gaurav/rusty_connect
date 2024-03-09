use std::sync::Arc;

use async_graphql::Object;
use tokio::sync::RwLock;

use crate::devices::{DeviceManager, DeviceWithState};

pub struct Query {
    pub device_manager: Arc<RwLock<DeviceManager>>,
}

#[Object]
impl Query {
    pub async fn howdy(&self) -> &str {
        "cowboy!"
    }

    pub async fn devices(&self) -> Vec<DeviceWithState> {
        let devices = {
            let manager = self.device_manager.read().await;
            manager.devices.values().cloned().collect::<Vec<_>>()
        };
        devices
    }

    pub async fn device(&self, id: String) -> anyhow::Result<DeviceWithState> {
        let device = {
            let manager = self.device_manager.read().await;
            manager.devices.get(&id).cloned()
        };
        device.ok_or(anyhow::anyhow!("Not device with givenId"))
    }
}
