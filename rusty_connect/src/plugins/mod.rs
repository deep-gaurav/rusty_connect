use std::any::TypeId;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Ok;
use async_graphql::{Context, OutputType, Union};
use async_graphql::{Object, ObjectType, SimpleObject};
use paste::paste;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::warn;

use crate::devices::{DeviceManager, DeviceState, DeviceWithState};
use crate::payloads::{IdentityPayloadBody, PairPayloadBody, Payload};

use self::battery::Batttery;
use self::notification::Notification;
use self::{clipboard::Clipboard, ping::Ping};

pub mod battery;
pub mod clipboard;
pub mod notification;
pub mod ping;

pub trait Plugin: async_graphql::ObjectType + Sized {
    type PluginPayload: ObjectType + Serialize;
    type PluginConfig: OutputType + Clone + Serialize + Deserialize<'static> + Default;
    type PluginState: OutputType + Clone + Default;

    fn init(device_mangager: &DeviceManager) -> Self;

    fn incoming_capabilities(&self) -> Vec<String>;
    fn outgoing_capabilities(&self) -> Vec<String>;

    fn is_enabled(&self, config: &Option<Self::PluginConfig>) -> bool;
    fn should_send(
        &self,
        config: &Option<Self::PluginConfig>,
        state: &mut Self::PluginState,
        payload: &Self::PluginPayload,
    ) -> bool;

    fn parse_payload(
        &self,
        payload: &Payload,
        device_address: SocketAddr,
    ) -> impl std::future::Future<Output = Option<Self::PluginPayload>> + Send;

    fn update_state(&self, _payload: &Self::PluginPayload, _state: &mut Self::PluginState) {}
}

trait PluginExt: Plugin {
    fn get_config_from_plugin_configs(configs: &PluginConfigs) -> &Option<Self::PluginConfig>;

    fn get_state_from_plugin_states(configs: &mut PluginStates) -> &mut Self::PluginState;

    fn get_config<'ctx>(
        &self,
        context: &Context<'ctx>,
        device_id: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<Option<Self::PluginConfig>>> + Send {
        async move {
            let device_manager = {
                context
                    .data::<Arc<RwLock<DeviceManager>>>()
                    .map_err(|e| anyhow::anyhow!("{e:?}"))?
                    .read()
                    .await
                    .devices
                    .clone()
            };
            let device = device_manager
                .get(device_id)
                .ok_or(anyhow::anyhow!("Device not found with given id"))?;
            Ok(Self::get_config_from_plugin_configs(&device.device.plugin_configs).clone())
        }
    }

    fn send_payload<'ctx>(
        &self,
        context: &Context<'ctx>,
        device_id: Option<&str>,
        payload_type: &str,
        payload: Self::PluginPayload,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send {
        async move {
            let mut device_manager = context
                .data::<Arc<RwLock<DeviceManager>>>()
                .map_err(|e| anyhow::anyhow!("{e:?}"))?
                .write()
                .await;
            let devices = &mut device_manager.devices;
            let serialized_value = serde_json::to_value(&payload)?;
            let serialized_payload = Payload::generate_new(payload_type, serialized_value);
            if let Some(device_id) = device_id {
                let device = devices
                    .get_mut(device_id)
                    .ok_or(anyhow::anyhow!("No device with given id"))?;
                if !device.device.paired {
                    return Err(anyhow::anyhow!("Device not paired"));
                }
                if !self.should_send(
                    Self::get_config_from_plugin_configs(&device.device.plugin_configs),
                    Self::get_state_from_plugin_states(&mut device.device.plugin_states),
                    &payload,
                ) {
                    return Err(anyhow::anyhow!("Plugin disabled for config"));
                }
                if let DeviceState::Active(_, _, sender) = &device.state {
                    sender.send_async(serialized_payload).await?;
                } else {
                    return Err(anyhow::anyhow!("Device not connected"));
                }
            } else {
                for (_, device) in devices.iter_mut() {
                    if device.device.paired
                        && self.should_send(
                            Self::get_config_from_plugin_configs(&device.device.plugin_configs),
                            Self::get_state_from_plugin_states(&mut device.device.plugin_states),
                            &payload,
                        )
                    {
                        if let DeviceState::Active(_, _, sender) = &device.state {
                            if let Err(err) = sender.send_async(serialized_payload.clone()).await {
                                warn!("Failed to send {err:?}")
                            }
                        }
                    }
                }
            }
            Ok(())
        }
    }
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

            #[derive(Debug,Serialize,Deserialize,Default,Clone, SimpleObject)]
            #[serde(default)]
            pub struct PluginConfigs{
                $(
                    pub [<$type:lower>]: Option<<$type as Plugin>::PluginConfig>,
                )*
            }

            #[derive(Debug, Default,Clone, SimpleObject)]
            pub struct PluginStates{
                $(
                    pub [<$type:lower>]: <$type as Plugin>::PluginState,
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
                Connected(Connected),
                Disconnected(Disconnected),
                Identity(IdentityPayloadBody),
                Pair(PairPayloadBody),
                $(
                    $type(<$type as Plugin>::PluginPayload),
                )*
                Unknown(Payload)
            }

            #[derive(SimpleObject)]
            pub struct Connected {
                pub id: String
            }

            #[derive(SimpleObject)]
            pub struct Disconnected {
                pub id: String
            }

            impl PluginManager {

                pub fn new(device_id:String,device_name:String, device_type:String, device_manager:&DeviceManager) -> Self {
                    Self {
                        device_id,
                        device_name,
                        device_type,
                        $(
                            [<$type:lower>]: <$type as Plugin>::init(&device_manager),
                        )*
                    }
                }

                pub async fn parse_payload(&self, payload: Payload, device: Option<&DeviceWithState>) -> anyhow::Result<ReceivedPayload> {
                    if payload.r#type == "kdeconnect.identity" {
                        let identity = serde_json::from_value::<IdentityPayloadBody>(payload.body)?;
                       return Ok(ReceivedPayload::Identity(identity))
                    }
                    else if payload.r#type == "kdeconnect.pair" {
                        let pair = serde_json::from_value::<PairPayloadBody>(payload.body)?;
                      return  Ok(ReceivedPayload::Pair(pair))
                    }
                    if let Some(device) = device {
                        $(
                            if let DeviceState::Active(_,address,_) = device.state {
                                if self.[<$type:lower>].is_enabled($type::get_config_from_plugin_configs(&device.device.plugin_configs)) {
                                    if let Some([<$type:lower _payload>]) = self.[<$type:lower>].parse_payload(&payload,address).await {
                                        return Ok(ReceivedPayload::$type([<$type:lower _payload>]))
                                    }
                                }
                            }
                        )*
                    }
                    Ok(ReceivedPayload::Unknown(payload))
                }

                pub fn update_state(&self,payload:&ReceivedPayload, device:&mut DeviceWithState){
                    match payload{
                        $(
                            ReceivedPayload::$type(data) => {
                                let state = $type::get_state_from_plugin_states(&mut device.device.plugin_states);
                                self.[<$type:lower>].update_state(&data, state);
                            }
                        )*,
                        _ => {}

                    }
                }
            }

            $(
                impl PluginExt for $type {
                    fn get_config_from_plugin_configs(configs: &PluginConfigs) -> &Option<Self::PluginConfig> {
                        &configs.[<$type:lower>]
                    }

                    fn get_state_from_plugin_states(states: &mut PluginStates) -> &mut Self::PluginState {
                        &mut states.[<$type:lower>]
                    }

                }
            )*

        }


    };
}

impl PluginManager {
    pub fn get_identity_payload(&self, port: Option<u16>) -> anyhow::Result<Payload> {
        let value = serde_json::to_value(self.get_identity_payload_body(port))
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(Payload::generate_new("kdeconnect.identity", value))
    }

    pub fn get_identity_payload_body(&self, port: Option<u16>) -> IdentityPayloadBody {
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

register_plugins!(Ping, Clipboard, Batttery, Notification);
