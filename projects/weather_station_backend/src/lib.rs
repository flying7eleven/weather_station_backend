use afluencia::{AfluenciaClient, DataPoint, Value};
use chrono::Local;

pub mod boundary;

pub struct StorageBackend;

impl Default for StorageBackend {
    fn default() -> Self {
        StorageBackend
    }
}

impl StorageBackend {
    pub fn store_measurement(
        &self,
        sensor: &str,
        temperature: f32,
        rel_humidity: f32,
        abs_humidity: f32,
        pressure: f32,
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
        influx_measurement.add_field("on_battery", Value::Boolean(false));
        influx_measurement.add_field("battery_voltage", Value::Float(4.20));
        influx_measurement.add_timestamp(measurement_time.timestamp_nanos());

        // write into the InfluxDB
        let influx_client = AfluenciaClient::default();
        influx_client.write_measurement(influx_measurement);
    }
}
