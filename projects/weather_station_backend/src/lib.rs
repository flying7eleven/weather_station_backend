#[macro_use]
extern crate diesel;

use crate::models::NewMeasurement;
use chrono::Local;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use dotenv::dotenv;
use std::env;

pub mod boundary;
pub mod models;
pub mod schema;

pub struct StorageBackend {
    connection: MysqlConnection,
}

impl Default for StorageBackend {
    fn default() -> Self {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let tmp_connection = MysqlConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

        StorageBackend {
            connection: tmp_connection,
        }
    }
}

impl StorageBackend {
    pub fn store_measurement(
        &self,
        sensor: &str,
        temperature: f32,
        humidity: f32,
        pressure: f32,
    ) -> usize {
        use schema::measurements;

        let new_measurement = NewMeasurement {
            sensor,
            time: &Local::now().naive_utc(),
            temperature,
            humidity,
            pressure,
        };

        diesel::insert_into(measurements::table)
            .values(&new_measurement)
            .execute(&self.connection)
            .expect("Error saving new measurement!")
    }
}
