use async_graphql::{Object, SimpleObject};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::utils::get_timestamp;

pub enum RustyPayload {
    Connected,
    Disconnect,
    KDEConnectPayload(Payload),
}

pub enum PayloadType {
    Broadcast(Payload),
    ConnectionPayload(String, RustyPayload),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    pub id: u128,
    pub r#type: String,
    pub body: serde_json::Value,
}

impl Payload {
    pub fn generate_new(payload_type: &str, value: Value) -> Self {
        Self {
            id: get_timestamp(),
            r#type: payload_type.to_string(),
            body: value,
        }
    }
}

#[Object]
impl Payload {
    pub async fn id(&self) -> String {
        self.id.to_string()
    }

    pub async fn r#type(&self) -> &str {
        &self.r#type
    }

    pub async fn body(&self) -> &serde_json::Value {
        &self.body
    }
}

#[derive(Debug, Serialize, Deserialize, SimpleObject, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IdentityPayloadBody {
    pub device_name: String,
    pub device_id: String,
    pub device_type: String,
    pub incoming_capabilities: Vec<String>,
    pub outgoing_capabilities: Vec<String>,
    pub protocol_version: u32,
    pub tcp_port: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, SimpleObject, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PairPayloadBody {
    pub pair: bool,
}
