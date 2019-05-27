use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Measurement {
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
}
