use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Measurement {
    pub sensor: String,
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub raw_voltage: f32,
    pub charge: f32,
    pub firmware_version: String,
}
