use std::collections::HashMap;

use api::all_devices::DeviceWithStateFields;
use tokio::sync::RwLock;

pub struct Devices {
    pub devices: RwLock<Vec<DeviceWithStateFields>>,
}

pub struct NotificationState {
    pub notifications: RwLock<HashMap<String, NotificationShown>>,
}

#[derive(Debug, PartialEq)]
pub struct NotificationShown {
    pub id: String,
    pub title: Option<String>,
    pub content: Option<String>,
}
