use std::{net::SocketAddr, sync::Arc};

use async_graphql::Object;
use tokio::{net::UdpSocket, sync::RwLock};

use crate::{
    devices::{DeviceManager, DeviceWithState},
    payloads::{IdentityPayloadBody, Payload},
    plugins::PluginManager,
};

pub struct Mutation {
    pub plugin_manager: Arc<PluginManager>,
    pub device_manager: Arc<RwLock<DeviceManager>>,
}

#[Object]
impl Mutation {
    pub async fn plugins(&self) -> &PluginManager {
        &self.plugin_manager
    }

    pub async fn broadcast_identity_udp(&self) -> anyhow::Result<IdentityPayloadBody> {
        let identity = self.plugin_manager.get_identity_payload_body(Some(1716));
        let udpsock = UdpSocket::bind("0.0.0.0:0").await?;
        udpsock.set_broadcast(true)?;

        let value = serde_json::to_value(identity.clone())?;
        let payload_string =
            serde_json::to_string(&Payload::generate_new("kdeconnect.identity", value))?;
        dbg!(&payload_string);
        let mut payload_bytes = payload_string.as_bytes().to_vec();
        payload_bytes.append(&mut b"\n".to_vec());
        let advertise_addr: SocketAddr = "255.255.255.255:1716".parse().expect("Invalid address");
        udpsock.send_to(&payload_bytes, advertise_addr).await?;

        Ok(identity)
    }

    pub async fn pair(&self, id: String) -> anyhow::Result<DeviceWithState> {
        let mut manager = self.device_manager.write().await;
        let device = manager.pair(&id)?;
        Ok(device.clone())
    }
}
