use std::collections::HashMap;

use api::all_devices::DeviceWithStateFields;
use tokio::sync::RwLock;

pub struct Devices {
    pub devices: RwLock<Vec<DeviceWithStateFields>>,
}

pub struct NotificationState {
    pub notifications: RwLock<HashMap<String, notify_rust::NotificationHandle>>,
}
