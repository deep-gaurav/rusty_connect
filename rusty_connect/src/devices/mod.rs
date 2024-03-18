use std::{
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_graphql::{Object, SimpleObject};
use flume::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::debug;

use crate::{
    cert::CertPair,
    payloads::{IdentityPayloadBody, PairPayloadBody, Payload, PayloadType},
    plugins::{
        share::DownloadProgress, Connected, Disconnected, PluginConfigs, PluginStates,
        ReceivedPayload,
    },
};

pub struct DeviceManager {
    pub devices: HashMap<String, DeviceWithState>,
    pub sender: flume::Sender<PayloadType>,
    pub receiver: flume::Receiver<PayloadType>,
    config_path: PathBuf,
    pub icons_path: PathBuf,
    pub downloads_path: PathBuf,
    pub download_tasks: Arc<Mutex<HashMap<String, tokio::sync::watch::Receiver<DownloadProgress>>>>,
    pub certs: CertPair,
}

#[derive(Serialize, Deserialize)]
struct DeviceConfig {
    devices: Vec<Device>,
}

impl DeviceManager {
    pub async fn load_or_create(
        config_folder: &Path,
        sender: flume::Sender<PayloadType>,
        receiver: flume::Receiver<PayloadType>,
        certs: CertPair,
    ) -> anyhow::Result<Self> {
        let device_config = config_folder.join("devices");
        let icons_path = config_folder.join("icons_path");
        let downloads_path = config_folder.join("downloads");
        tokio::fs::create_dir_all(&icons_path).await?;
        tokio::fs::create_dir_all(&downloads_path).await?;
        let config = 'config: {
            if let Ok(data) = tokio::fs::read(&device_config).await {
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
            icons_path,
            downloads_path,
            certs,
            download_tasks: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn connected_to(
        &mut self,
        address: SocketAddr,
        identity: IdentityPayloadBody,
    ) -> anyhow::Result<(Sender<PayloadType>, Receiver<Payload>, uuid::Uuid)> {
        let device_id = identity.device_id.clone();
        let DeviceWithState { device: _, state } = self
            .devices
            .entry(identity.device_id.clone())
            .or_insert(DeviceWithState {
                device: Device {
                    paired: false,
                    id: identity.device_id.clone(),
                    identity,
                    plugin_configs: PluginConfigs::default(),
                    plugin_states: PluginStates::default(),
                },
                state: DeviceState::InActive,
            });

        let (tx, rx) = flume::bounded(0);
        let id = uuid::Uuid::new_v4();
        *state = DeviceState::Active(id, address, tx);
        if let Err(err) = self.sender.try_send((
            device_id.clone(),
            ReceivedPayload::Connected(Connected { id: device_id }),
        )) {
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
            DeviceState::Active(id, _, _) => {
                if id == connection_id {
                    entry.state = DeviceState::InActive;
                    if let Err(err) = self.sender.try_send((
                        device_id.to_string(),
                        ReceivedPayload::Disconnected(Disconnected {
                            id: device_id.to_string(),
                        }),
                    )) {
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
            DeviceState::Active(_, _, sender) => {
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
    pub device: Device,
    pub state: DeviceState,
}

#[Object]
impl DeviceWithState {
    pub async fn device<'a>(&self) -> &Device {
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
    pub plugin_configs: PluginConfigs,
    #[serde(skip)]
    pub plugin_states: PluginStates,
}

#[derive(Clone)]
pub enum DeviceState {
    InActive,
    Active(uuid::Uuid, SocketAddr, Sender<Payload>),
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
