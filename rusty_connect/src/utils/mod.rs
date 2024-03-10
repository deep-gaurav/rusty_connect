use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_timestamp() -> u128 {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).expect("???");
    since_epoch.as_millis()
}
