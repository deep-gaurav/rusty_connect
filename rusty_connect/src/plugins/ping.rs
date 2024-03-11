use async_graphql::{Object, SimpleObject};
use serde::{Deserialize, Serialize};
use tracing::info;

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

    fn incoming_capabilities(&self) -> Vec<String> {
        vec!["kdeconnect.ping".to_string()]
    }

    fn outgoing_capabilities(&self) -> Vec<String> {
        vec!["kdeconnect.ping".to_string()]
    }

    fn parse_payload(&self, payload: &crate::payloads::Payload) -> Option<Self::PluginPayload> {
        if payload.r#type == "kdeconnect.ping" {
            Some(PingPayload { pinged: true })
        } else {
            None
        }
    }

    fn is_enabled(&self, config: &Option<Self::PluginConfig>) -> bool {
        if let Some(config) = config {
            config.enabled
        } else {
            true
        }
    }
}

#[derive(SimpleObject, Serialize)]
pub struct PingPayload {
    pinged: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, SimpleObject)]
pub struct PingConfig {
    enabled: bool,
}
