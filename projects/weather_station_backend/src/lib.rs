#[macro_use]
extern crate diesel;

use crate::models::NewMeasurement;
use afluencia::{AfluenciaClient, DataPoint, Value};
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

        // get the current time as an over-all time measurement
        let measurement_time = Local::now().naive_utc();

        // define the required data structure for the InfluxDB
        let mut influx_measurement = DataPoint::new("environment");
        influx_measurement.add_tag("sensor", Value::String(String::from(sensor)));
        influx_measurement.add_field("temperature", Value::Float(temperature as f64));
        influx_measurement.add_field("humidity", Value::Float(humidity as f64));
        influx_measurement.add_field("pressure", Value::Float(pressure as f64));
        influx_measurement.add_timestamp(measurement_time.timestamp_millis());

        // define the required datas tructure for the database
        let db_measurement = NewMeasurement {
            sensor,
            time: &measurement_time,
            temperature,
            humidity,
            pressure,
        };

        // write into the InfluxDB
        let influx_client = AfluenciaClient::default();
        influx_client.write_measurement(influx_measurement);

        // write into the database
        diesel::insert_into(measurements::table)
            .values(&db_measurement)
            .execute(&self.connection)
            .expect("Error saving new measurement!")
    }
}
