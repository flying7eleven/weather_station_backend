use crate::configuration::Configuration;
use afluencia::{AfluenciaClient, DataPoint, Value};
use chrono::Local;
use std::clone::Clone;

pub mod boundary;
pub mod configuration;

pub struct StorageBackend {
    configuration: Configuration,
}

impl StorageBackend {
    pub fn with_configuration(config: Configuration) -> StorageBackend {
        StorageBackend {
            configuration: config,
        }
    }

    pub fn store_measurement(
        &self,
        sensor: &str,
        temperature: f32,
        rel_humidity: f32,
        abs_humidity: f32,
        pressure: f32,
        voltage: f32,
        charge: f32,
    ) {
        // get the current time as an over-all time measurement
        let measurement_time = Local::now().naive_utc();

        // define the required data structure for the InfluxDB
        let mut influx_measurement = DataPoint::new("weather_measurement");
        influx_measurement.add_tag("sensor", Value::String(String::from(sensor)));
        influx_measurement.add_field("temperature", Value::Float(f64::from(temperature)));
        influx_measurement.add_field("rel_humidity", Value::Float(f64::from(rel_humidity)));
        influx_measurement.add_field("abs_humidity", Value::Float(f64::from(abs_humidity)));
        influx_measurement.add_field("pressure", Value::Float(f64::from(pressure)));
        influx_measurement.add_field("raw_battery_voltage", Value::Float(f64::from(voltage)));
        influx_measurement.add_field("battery_charge", Value::Float(f64::from(charge)));
        influx_measurement.add_field("on_battery", Value::Boolean(false));
        influx_measurement.add_timestamp(measurement_time.timestamp_nanos());

        // create an instance of the influx client
        let mut influx_client = AfluenciaClient::new(
            self.configuration.influx_storage.host.as_str(),
            self.configuration.influx_storage.port,
            self.configuration.influx_storage.database.as_str(),
        );

        // check if a username and password can be set, if so, do so :D
        if self.configuration.influx_storage.user.is_some() {
            let user_optional = self.configuration.influx_storage.user.clone();
            influx_client.user(user_optional.unwrap());
        }
        if self.configuration.influx_storage.password.is_some() {
            let password_optional = self.configuration.influx_storage.password.clone();
            influx_client.password(password_optional.unwrap());
        }

        // write the measurement to the database
        influx_client.write_measurement(influx_measurement);
    }
}
