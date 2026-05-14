use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct RawDevice {
    pub serial: String,
    pub state: String,
    pub product: Option<String>,
    pub model: Option<String>,
    pub device: Option<String>,
    pub transport_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidDeviceProps {
    pub model: String,
    pub brand: String,
    pub android_version: String,
    pub screen_size: Option<String>,
    pub battery_level: Option<i32>,
}
