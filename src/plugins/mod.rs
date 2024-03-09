use async_graphql::Union;
use async_graphql::{Object, ObjectType};
use paste::paste;

use crate::payloads::{IdentityPayloadBody, PairPayloadBody, Payload};

use self::{clipboard::Clipboard, ping::Ping};

pub mod clipboard;
pub mod ping;

pub trait Plugin: async_graphql::ObjectType + Default {
    type PluginPayload: ObjectType;

    fn incoming_capabilities(&self) -> Vec<String>;
    fn outgoing_capabilities(&self) -> Vec<String>;

    fn parse_payload(&self, payload: &Payload) -> Option<Self::PluginPayload>;
}

macro_rules! register_plugins {
    ($($type:ident),*) => {
        paste! {
            pub struct PluginManager {

                pub device_name: String,
                pub device_id: String,
                pub device_type: String,
                $(
                    pub [<$type:lower>]: $type,
                )*
            }

            #[Object]
            impl PluginManager {
                $(
                    pub async fn [<$type:lower>](&self) -> &$type {
                        &self.[<$type:lower>]
                    }
                )*
            }

            impl PluginManager {

                pub fn incoming_capabilities(&self) -> Vec<String>{
                    let mut capabilities = vec![];
                    $(
                        capabilities.extend(self.[<$type:lower>].incoming_capabilities());
                    )*
                    capabilities
                }
                pub fn outgoing_capabilities(&self) -> Vec<String>{
                    let mut capabilities = vec![];
                    $(
                        capabilities.extend(self.[<$type:lower>].outgoing_capabilities());
                    )*
                    capabilities
                }
            }

            #[derive(Union)]
            pub enum ReceivedPayload{
                Identity(IdentityPayloadBody),
                Pair(PairPayloadBody),
                $(
                    $type(<$type as Plugin>::PluginPayload),
                )*
                Unknown(Payload)
            }


            impl PluginManager {

                pub fn new(device_id:String,device_name:String, device_type:String) -> Self {
                    Self {
                        device_id,
                        device_name,
                        device_type,
                        $(
                            [<$type:lower>]: Default::default(),
                        )*
                    }
                }

                pub fn parse_payload(&self, payload: Payload) -> anyhow::Result<ReceivedPayload> {
                    if payload.r#type == "kdeconnect.identity" {
                        let identity = serde_json::from_value::<IdentityPayloadBody>(payload.body)?;
                        Ok(ReceivedPayload::Identity(identity))
                    }
                    else if payload.r#type == "kdeconnect.pair" {
                        let pair = serde_json::from_value::<PairPayloadBody>(payload.body)?;
                        Ok(ReceivedPayload::Pair(pair))
                    }
                    $(
                        else if let Some([<$type:lower _payload>]) = self.[<$type:lower>].parse_payload(&payload) {
                            Ok(ReceivedPayload::$type([<$type:lower _payload>]))
                        }
                    )*
                    else {
                        Ok(ReceivedPayload::Unknown(payload))
                    }
                }
            }
        }
    };
}

impl PluginManager {
    pub fn get_identity_payload(&self, port: Option<u32>) -> anyhow::Result<Payload> {
        let value = serde_json::to_value(self.get_identity_payload_body(port))
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(Payload::generate_new("kdeconnect.identity", value))
    }

    pub fn get_identity_payload_body(&self, port: Option<u32>) -> IdentityPayloadBody {
        IdentityPayloadBody {
            device_id: self.device_id.clone(),
            device_name: self.device_name.clone(),
            device_type: self.device_type.clone(),
            incoming_capabilities: self.incoming_capabilities(),
            outgoing_capabilities: self.outgoing_capabilities(),
            protocol_version: 7,
            tcp_port: port,
        }
    }
}

register_plugins!(Ping, Clipboard);
