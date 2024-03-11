use async_graphql::{Enum, Object, SimpleObject};
use serde::{Deserialize, Deserializer, Serialize};
use tracing::info;

use super::Plugin;

#[derive(Default)]
pub struct Batttery;

#[Object]
impl Batttery {
    pub async fn send_batery(&self) -> anyhow::Result<&str> {
        Ok("Success")
    }
}

impl Plugin for Batttery {
    type PluginPayload = BatteryPayload;
    type PluginConfig = BatteryConfig;
    type PluginState = BatteryState;

    fn incoming_capabilities(&self) -> Vec<String> {
        vec!["kdeconnect.battery".to_string()]
    }

    fn outgoing_capabilities(&self) -> Vec<String> {
        vec!["kdeconnect.battery".to_string()]
    }

    fn parse_payload(
        &self,
        payload: &crate::payloads::Payload,
        state: &mut Self::PluginState,
    ) -> Option<Self::PluginPayload> {
        if payload.r#type == "kdeconnect.battery" {
            let payload = serde_json::from_value::<Self::PluginPayload>(payload.body.clone());
            if let Ok(payload) = payload {
                state.last_status = Some(payload.clone());
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
}

#[derive(SimpleObject, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatteryPayload {
    current_charge: i32,
    is_charging: bool,
    threshold_event: BatteryEventType,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, SimpleObject)]
pub struct BatteryConfig {
    enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, SimpleObject)]
pub struct BatteryState {
    last_status: Option<BatteryPayload>,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Enum)]
pub enum BatteryEventType {
    None = 0,
    BatteryLow = 1,
    Unknown = 2,
}

impl serde::Serialize for BatteryEventType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

impl<'de> serde::Deserialize<'de> for BatteryEventType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            0 => Ok(BatteryEventType::None),
            1 => Ok(BatteryEventType::BatteryLow),
            _ => Ok(BatteryEventType::Unknown),
        }
    }
}
