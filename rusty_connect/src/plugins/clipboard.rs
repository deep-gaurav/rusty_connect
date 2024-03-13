use std::net::SocketAddr;

use async_graphql::{Context, Object, SimpleObject};
use serde::{Deserialize, Serialize};

use super::{Plugin, PluginExt};

#[derive(Default)]
pub struct Clipboard;

#[Object]
impl Clipboard {
    pub async fn send_clipboard<'ctx>(
        &self,
        context: &Context<'ctx>,
        device_id: Option<String>,
        content: String,
    ) -> anyhow::Result<&str> {
        let clipboard_payload = ClipboardPayload { content };
        self.send_payload(
            context,
            device_id.as_deref(),
            "kdeconnect.clipboard",
            clipboard_payload,
        )
        .await?;
        Ok("success")
    }
}

impl Plugin for Clipboard {
    type PluginPayload = ClipboardPayload;
    type PluginConfig = ClipboardConfig;
    type PluginState = ClipboardState;

    fn init(_device_mangager: &crate::devices::DeviceManager) -> Self {
        Self
    }

    fn incoming_capabilities(&self) -> Vec<String> {
        vec![
            "kdeconnect.clipboard".to_string(),
            // "kdeconnect.clipboard.connect".to_string(), // TODO: figure out what this does? autosync?
        ]
    }

    fn outgoing_capabilities(&self) -> Vec<String> {
        vec![
            "kdeconnect.clipboard".to_string(),
            // "kdeconnect.clipboard.connect".to_string(), // TODO: figure out what this does? autosync?
        ]
    }

    async fn parse_payload(
        &self,
        payload: &crate::payloads::Payload,
        _address: SocketAddr,
    ) -> Option<Self::PluginPayload> {
        if payload.r#type == "kdeconnect.clipboard" {
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

#[derive(SimpleObject, Deserialize, Serialize)]
pub struct ClipboardPayload {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, SimpleObject)]
pub struct ClipboardConfig {
    enabled: bool,
    send_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, SimpleObject)]
pub struct ClipboardState {
    enabled: bool,
}
