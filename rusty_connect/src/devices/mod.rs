use std::collections::HashMap;

use async_graphql::{Object, SimpleObject};
use flume::{Receiver, Sender};
use tracing::debug;

use crate::payloads::{IdentityPayloadBody, PairPayloadBody, Payload, RustyPayload};

pub struct DeviceManager {
    pub devices: HashMap<String, DeviceWithState>,
    pub sender: flume::Sender<(String, RustyPayload)>,
    pub receiver: flume::Receiver<(String, RustyPayload)>,
}

impl DeviceManager {
    pub fn connected_to(
        &mut self,
        identity: IdentityPayloadBody,
    ) -> anyhow::Result<(Sender<(String, RustyPayload)>, Receiver<Payload>)> {
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
        match state {
            DeviceState::InActive => {
                let (tx, rx) = flume::bounded(0);
                *state = DeviceState::Active(tx);
                if let Err(err) = self.sender.try_send((device_id, RustyPayload::Connected)) {
                    debug!("Error sending connected message {err:?}");
                }
                Ok((self.sender.clone(), rx))
            }
            DeviceState::Active(_) => Err(anyhow::anyhow!("Device already connected")),
        }
    }

    pub fn disconnect(&mut self, device_id: &str) -> anyhow::Result<()> {
        let entry = self
            .devices
            .get_mut(device_id)
            .ok_or(anyhow::anyhow!("No device with given id"))?;
        entry.state = DeviceState::InActive;
        if let Err(err) = self
            .sender
            .try_send((device_id.to_string(), RustyPayload::Disconnect))
        {
            debug!("Error sending disconnected message {err:?}");
        }
        Ok(())
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
