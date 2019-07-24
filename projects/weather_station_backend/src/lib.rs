#[macro_use]
extern crate diesel;

use crate::models::NewMeasurement;
use afluencia::{AfluenciaClient, DataPoint, Value};
use chrono::Local;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use log::debug;
use std::env;
use std::str::FromStr;

pub mod boundary;
pub mod models;
pub mod schema;

pub struct StorageBackend {
    connection: Option<MysqlConnection>,
}

impl Default for StorageBackend {
    fn default() -> Self {
        let rational_db_enabled = bool::from_str(
            &env::var("WEATHER_STATION_USE_DB").unwrap_or_else(|_| String::from("true")),
        )
        .unwrap_or(true);

        if rational_db_enabled {
            let database_url =
                env::var("WEATHER_DATABASE_URL").expect("WEATHER_DATABASE_URL must be set");
            let tmp_connection = MysqlConnection::establish(&database_url)
                .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

            return StorageBackend {
                connection: Some(tmp_connection),
            };
        }

        StorageBackend { connection: None }
    }
}

impl StorageBackend {
    pub fn store_measurement(&self, sensor: &str, temperature: f32, humidity: f32, pressure: f32) {
        use schema::measurements;

        // get the current time as an over-all time measurement
        let measurement_time = Local::now().naive_utc();

        // define the required data structure for the InfluxDB
        let mut influx_measurement = DataPoint::new("weather_measurement");
        influx_measurement.add_tag("sensor", Value::String(String::from(sensor)));
        influx_measurement.add_field("temperature", Value::Float(f64::from(temperature)));
        influx_measurement.add_field("humidity", Value::Float(f64::from(humidity)));
        influx_measurement.add_field("pressure", Value::Float(f64::from(pressure)));
        influx_measurement.add_field("on_battery", Value::Boolean(false));
        influx_measurement.add_field("battery_voltage", Value::Float(4.20));
        influx_measurement.add_timestamp(measurement_time.timestamp_nanos());

        // write into the InfluxDB
        let influx_client = AfluenciaClient::default();
        influx_client.write_measurement(influx_measurement);

        // just execute the rest if the rational database support was enabled
        match &self.connection {
            Some(connection) => {
                let db_measurement = NewMeasurement {
                    sensor,
                    time: &measurement_time,
                    temperature,
                    humidity,
                    pressure,
                };

                // write into the database
                diesel::insert_into(measurements::table)
                    .values(&db_measurement)
                    .execute(connection)
                    .expect("Error saving new measurement!");
            }
            None => debug!("Not writing to the local database since it was disabled."),
        }
    }
}
