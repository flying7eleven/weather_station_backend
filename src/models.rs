use super::schema::measurements;
use chrono::NaiveDateTime;

#[derive(Queryable)]
pub struct Measurements {
    pub id: i32,
    pub time: NaiveDateTime,
    pub sensor: String,
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
}

#[derive(Insertable)]
#[table_name = "measurements"]
pub struct NewMeasurement<'a> {
    pub time: &'a NaiveDateTime,
    pub sensor: &'a str,
    pub temperature: &'a f32,
    pub humidity: &'a f32,
    pub pressure: &'a f32,
}
