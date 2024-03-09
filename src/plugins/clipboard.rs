use async_graphql::{Object, SimpleObject};

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
        None
    }
}

#[derive(SimpleObject)]
pub struct ClipboardPayload {
    pub content: String,
}
