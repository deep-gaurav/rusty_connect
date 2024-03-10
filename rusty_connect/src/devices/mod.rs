use std::{collections::HashMap, path::PathBuf};

use async_graphql::{Object, SimpleObject};
use flume::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::payloads::{IdentityPayloadBody, PairPayloadBody, Payload, RustyPayload};

pub struct DeviceManager {
    pub devices: HashMap<String, DeviceWithState>,
    pub sender: flume::Sender<(String, RustyPayload)>,
    pub receiver: flume::Receiver<(String, RustyPayload)>,
    config_path: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct DeviceConfig {
    devices: Vec<Device>,
}

impl DeviceManager {
    pub async fn load_or_create(
        device_config: &PathBuf,
        sender: flume::Sender<(String, RustyPayload)>,
        receiver: flume::Receiver<(String, RustyPayload)>,
    ) -> anyhow::Result<Self> {
        let config = 'config: {
            if let Ok(data) = tokio::fs::read(device_config).await {
                if let Ok(config) = serde_json::from_slice(&data) {
                    break 'config config;
                }
            }
            DeviceConfig { devices: vec![] }
        };
        let mut devices = HashMap::new();
        for device in config.devices.into_iter() {
            devices.insert(
                device.id.clone(),
                DeviceWithState {
                    device,
                    state: DeviceState::InActive,
                },
            );
        }
        Ok(Self {
            devices,
            sender,
            receiver,
            config_path: device_config.clone(),
        })
    }

    pub async fn connected_to(
        &mut self,
        identity: IdentityPayloadBody,
    ) -> anyhow::Result<(
        Sender<(String, RustyPayload)>,
        Receiver<Payload>,
        uuid::Uuid,
    )> {
        let device_id = identity.device_id.clone();
        let DeviceWithState { device: _, state } = self
            .devices
            .entry(identity.device_id.clone())
            .or_insert(DeviceWithState {
                device: Device {
                    paired: false,
                    id: identity.device_id.clone(),
                    identity,
                },
                state: DeviceState::InActive,
            });

        let (tx, rx) = flume::bounded(0);
        let id = uuid::Uuid::new_v4();
        *state = DeviceState::Active(id, tx);
        if let Err(err) = self.sender.try_send((device_id, RustyPayload::Connected)) {
            debug!("Error sending connected message {err:?}");
        }
        self.save().await?;
        Ok((self.sender.clone(), rx, id))
    }

    pub async fn save(&self) -> anyhow::Result<()> {
        let devices = self
            .devices
            .values()
            .cloned()
            .map(|d| d.device)
            .collect::<Vec<_>>();
        let data = serde_json::to_vec(&DeviceConfig { devices })?;
        tokio::fs::write(&self.config_path, data).await?;
        Ok(())
    }

    pub fn disconnect(
        &mut self,
        device_id: &str,
        connection_id: &uuid::Uuid,
    ) -> anyhow::Result<()> {
        let entry = self
            .devices
            .get_mut(device_id)
            .ok_or(anyhow::anyhow!("No device with given id"))?;
        match &entry.state {
            DeviceState::InActive => Err(anyhow::anyhow!("Already disconnected")),
            DeviceState::Active(id, _) => {
                if id == connection_id {
                    entry.state = DeviceState::InActive;
                    if let Err(err) = self
                        .sender
                        .try_send((device_id.to_string(), RustyPayload::Disconnect))
                    {
                        debug!("Error sending disconnected message {err:?}");
                    }
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Invalid connection id"))
                }
            }
        }
    }

    pub async fn pair(&mut self, id: &str, pair: bool) -> anyhow::Result<DeviceWithState> {
        let device = self
            .devices
            .get_mut(id)
            .ok_or(anyhow::anyhow!("No device with given id"))?;
        match &device.state {
            DeviceState::InActive => Err(anyhow::anyhow!("Device not connected?")),
            DeviceState::Active(_, sender) => {
                let value = serde_json::to_value(PairPayloadBody { pair })?;
                sender.try_send(Payload::generate_new("kdeconnect.pair", value))?;
                device.device.paired = pair;
                let device = device.clone();
                self.save().await?;

                Ok(device)
            }
        }
    }
}

#[derive(Clone)]
pub struct DeviceWithState {
    device: Device,
    state: DeviceState,
}

#[Object]
impl DeviceWithState {
    pub async fn device(&self) -> &Device {
        &self.device
    }

    pub async fn is_connected(&self) -> bool {
        self.state.is_active()
    }
}

#[derive(SimpleObject, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub identity: IdentityPayloadBody,
    pub paired: bool,
}

#[derive(Clone)]
pub enum DeviceState {
    InActive,
    Active(uuid::Uuid, Sender<Payload>),
}

impl DeviceState {
    /// Returns `true` if the device state is [`Active`].
    ///
    /// [`Active`]: DeviceState::Active
    #[must_use]
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active(..))
    }
}
