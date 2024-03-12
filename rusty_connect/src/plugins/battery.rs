use async_graphql::{Context, Enum, Object, SimpleObject};
use serde::{Deserialize, Deserializer, Serialize};
use tracing::info;

use crate::plugins::PluginExt;

use super::Plugin;

#[derive(Default)]
pub struct Batttery;

#[Object]
impl Batttery {
    pub async fn send_batery<'ctx>(
        &self,
        context: &Context<'ctx>,
        current_charge: f32,
        is_charging: bool,
        device_id: Option<String>,
    ) -> anyhow::Result<&str> {
        let battery_payload = BatteryPayload {
            is_charging,
            current_charge,
            threshold_event: BatteryEventType::None,
        };
        // let payload =
        self.send_payload(
            context,
            device_id.as_deref(),
            "kdeconnect.battery",
            battery_payload,
        )
        .await?;
        Ok("success")
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

    fn should_send(
        &self,
        config: &Option<Self::PluginConfig>,
        state: &mut Self::PluginState,
        payload: &Self::PluginPayload,
    ) -> bool {
        let should_send = if let Some(config) = config {
            if config.send_enabled {
                if let Some(last_payload) = &state.last_sent_status {
                    last_payload != payload
                } else {
                    true
                }
            } else {
                false
            }
        } else {
            true
        };
        if should_send {
            state.last_sent_status = Some(payload.clone());
        }
        should_send
    }
}

#[derive(SimpleObject, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BatteryPayload {
    current_charge: f32,
    is_charging: bool,
    threshold_event: BatteryEventType,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, SimpleObject)]
pub struct BatteryConfig {
    enabled: bool,
    send_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, SimpleObject)]
pub struct BatteryState {
    last_status: Option<BatteryPayload>,
    last_sent_status: Option<BatteryPayload>,
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
