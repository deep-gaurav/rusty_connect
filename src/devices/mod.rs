use std::collections::HashMap;

use async_graphql::{Object, SimpleObject};
use flume::{Receiver, Sender};

use crate::payloads::{IdentityPayloadBody, PairPayloadBody, Payload};

pub struct DeviceManager {
    pub devices: HashMap<String, DeviceWithState>,
    pub sender: flume::Sender<(String, Payload)>,
    pub receiver: flume::Receiver<(String, Payload)>,
}

impl DeviceManager {
    pub fn connected_to(
        &mut self,
        identity: IdentityPayloadBody,
    ) -> anyhow::Result<(Sender<(String, Payload)>, Receiver<Payload>)> {
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
        match state {
            DeviceState::InActive => {
                let (tx, rx) = flume::bounded(0);
                *state = DeviceState::Active(tx);
                Ok((self.sender.clone(), rx))
            }
            DeviceState::Active(_) => Err(anyhow::anyhow!("Device already connected")),
        }
    }

    pub fn pair(&mut self, id: &str) -> anyhow::Result<&DeviceWithState> {
        let device = self
            .devices
            .get_mut(id)
            .ok_or(anyhow::anyhow!("No device with given id"))?;
        match &device.state {
            DeviceState::InActive => Err(anyhow::anyhow!("Device not connected?")),
            DeviceState::Active(sender) => {
                let value = serde_json::to_value(PairPayloadBody { pair: true })?;
                sender.try_send(Payload::generate_new("kdeconnect.pair", value))?;
                device.device.paired = true;
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

#[derive(SimpleObject, Clone)]
pub struct Device {
    pub id: String,
    pub identity: IdentityPayloadBody,
    pub paired: bool,
}

#[derive(Clone)]
pub enum DeviceState {
    InActive,
    Active(Sender<Payload>),
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
