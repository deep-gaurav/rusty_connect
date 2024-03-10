use async_graphql::{Object, SimpleObject};
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
}

#[derive(SimpleObject)]
pub struct PingPayload {
    pinged: bool,
}