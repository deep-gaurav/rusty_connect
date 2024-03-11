use api::all_devices::DeviceWithStateFields;
use tokio::sync::RwLock;

pub struct Devices {
    pub devices: RwLock<Vec<DeviceWithStateFields>>,
}
