#[macro_use]
extern crate diesel;

use crate::models::NewMeasurement;
use chrono::Local;
use core::borrow::Borrow;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::SqliteConnection;
use dotenv::dotenv;
use std::env;

pub mod boundary;
pub mod models;
pub mod schema;

fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn store_measurement<'a>(
    sensor: &'a str,
    temperature: &'a f32,
    humidity: &'a f32,
    pressure: &'a f32,
) -> usize {
    use schema::measurements;

    let db_connection = establish_connection();

    let new_measurement = NewMeasurement {
        sensor: sensor,
        time: &Local::now().naive_local(),
        temperature: temperature,
        humidity: humidity,
        pressure: pressure,
    };

    diesel::insert_into(measurements::table)
        .values(&new_measurement)
        .execute(db_connection.borrow())
        .expect("Error saving new measurement!")
}
