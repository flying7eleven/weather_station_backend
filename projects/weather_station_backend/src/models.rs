use chrono::NaiveDateTime;

pub struct Measurements {
    pub id: i32,
    pub time: NaiveDateTime,
    pub sensor: String,
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
}
