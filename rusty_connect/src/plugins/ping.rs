use std::net::SocketAddr;

use async_graphql::{Object, SimpleObject};
use serde::{Deserialize, Serialize};


use super::Plugin;

#[derive(Default)]
pub struct Ping;

#[Object]
impl Ping {
    pub async fn send_ping(&self) -> anyhow::Result<&str> {
        Ok("Success")
    }
}

impl Plugin for Ping {
    type PluginPayload = PingPayload;
    type PluginConfig = PingConfig;
    type PluginState = PingState;

    fn init(_device_mangager: &crate::devices::DeviceManager) -> Self {
        Self
    }

    fn incoming_capabilities(&self) -> Vec<String> {
        vec!["kdeconnect.ping".to_string()]
    }

    fn outgoing_capabilities(&self) -> Vec<String> {
        vec!["kdeconnect.ping".to_string()]
    }

    async fn parse_payload(
        &self,
        payload: &crate::payloads::Payload,
        _address: SocketAddr,
    ) -> Option<Self::PluginPayload> {
        if payload.r#type == "kdeconnect.ping" {
            let payload = serde_json::from_value::<Self::PluginPayload>(payload.body.clone());
            if let Ok(payload) = payload {
                return Some(payload);
            }
        }
        None
    }

    fn is_enabled(&self, config: &Option<Self::PluginConfig>) -> bool {
        if let Some(config) = config {
            config.enabled
        } else {
            true
        }
    }

    fn should_send(
        &self,
        config: &Option<Self::PluginConfig>,
        _state: &mut Self::PluginState,
        _payload: &Self::PluginPayload,
    ) -> bool {
        if let Some(config) = config {
            config.send_enabled
        } else {
            true
        }
    }
}

#[derive(SimpleObject, Serialize, Deserialize)]
pub struct PingPayload {
    message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, SimpleObject)]
pub struct PingConfig {
    enabled: bool,
    send_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, SimpleObject)]
pub struct PingState {
    enabled: bool,
}
