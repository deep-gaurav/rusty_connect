use async_graphql::{Object, SimpleObject};
use serde::Deserialize;

use super::Plugin;

#[derive(Default)]
pub struct Clipboard;

#[Object]
impl Clipboard {
    pub async fn send_clipboard(&self) -> anyhow::Result<&str> {
        Ok("success")
    }
}

impl Plugin for Clipboard {
    type PluginPayload = ClipboardPayload;

    fn incoming_capabilities(&self) -> Vec<String> {
        vec![
            "kdeconnect.clipboard".to_string(),
            // TODO: "kdeconnect.clipboard.connect".to_string()
        ]
    }

    fn outgoing_capabilities(&self) -> Vec<String> {
        vec![
            "kdeconnect.clipboard".to_string(),
            // TODO: "kdeconnect.clipboard.connect".to_string()
        ]
    }

    fn parse_payload(&self, payload: &crate::payloads::Payload) -> Option<Self::PluginPayload> {
        if payload.r#type == "kdeconnect.clipboard" {
            let payload = serde_json::from_value::<Self::PluginPayload>(payload.body.clone());
            if let Ok(payload) = payload {
                return Some(payload);
            }
        }
        None
    }
}

#[derive(SimpleObject, Deserialize)]
pub struct ClipboardPayload {
    pub content: String,
}
