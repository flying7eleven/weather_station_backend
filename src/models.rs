use diesel::sql_types::Timestamp;

#[derive(Queryable)]
pub struct Measurements {
    pub id: i32,
    pub time: Timestamp,
    pub sensor: String,
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
}
