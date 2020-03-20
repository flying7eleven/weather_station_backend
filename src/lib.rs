#![feature(proc_macro_hygiene, decl_macro)]

use crate::configuration::Configuration;
use chrono::Local;
use log::{debug, error};
use reqwest::blocking::Client;
use std::clone::Clone;
use std::collections::BTreeMap;
use std::time::Duration;

pub mod boundary;
pub mod configuration;
pub mod routes;

////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

// this part is used from https://github.com/driftluo/InfluxDBClient-rs by the github user driftluo

/// Influxdb value, Please look at [this address](https://docs.influxdata.com/influxdb/v1.3/write_protocols/line_protocol_reference/)
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

pub struct DataPoint {
    pub measurement: String,
    pub tags: BTreeMap<String, Value>,
    pub fields: BTreeMap<String, Value>,
    pub timestamp: Option<i64>,
}

impl DataPoint {
    pub fn new(measurement: &str) -> DataPoint {
        DataPoint {
            measurement: String::from(measurement),
            tags: BTreeMap::new(),
            fields: BTreeMap::new(),
            timestamp: None,
        }
    }

    pub fn add_tag<T: ToString>(&mut self, tag: T, value: Value) -> &mut Self {
        self.tags.insert(tag.to_string(), value);
        self
    }

    pub fn add_field<T: ToString>(&mut self, field: T, value: Value) -> &mut Self {
        self.fields.insert(field.to_string(), value);
        self
    }

    pub fn add_timestamp(&mut self, timestamp: i64) -> &mut Self {
        self.timestamp = Some(timestamp);
        self
    }
}

#[inline]
fn escape_measurement(value: &str) -> String {
    value.replace(",", "\\,").replace(" ", "\\ ")
}

#[inline]
fn escape_keys_and_tags(value: &str) -> String {
    value
        .replace(",", "\\,")
        .replace("=", "\\=")
        .replace(" ", "\\ ")
}

#[inline]
fn escape_string_field_value(value: &str) -> String {
    format!("\"{}\"", value.replace("\"", "\\\""))
}

pub fn line_serialization(point: DataPoint) -> String {
    let mut line = Vec::new();
    line.push(escape_measurement(&point.measurement));

    for (tag, value) in point.tags {
        line.push(",".to_string());
        line.push(escape_keys_and_tags(&tag));
        line.push("=".to_string());

        match value {
            Value::String(s) => line.push(escape_keys_and_tags(&s)),
            Value::Float(f) => line.push(f.to_string()),
            Value::Integer(i) => line.push(i.to_string() + "i"),
            Value::Boolean(b) => line.push({
                if b {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }),
        }
    }

    let mut was_first = true;

    for (field, value) in point.fields {
        line.push(
            {
                if was_first {
                    was_first = false;
                    " "
                } else {
                    ","
                }
            }
            .to_string(),
        );
        line.push(escape_keys_and_tags(&field));
        line.push("=".to_string());

        match value {
            Value::String(s) => line.push(escape_string_field_value(&s.replace("\\\"", "\\\\\""))),
            Value::Float(f) => line.push(f.to_string()),
            Value::Integer(i) => line.push(i.to_string() + "i"),
            Value::Boolean(b) => line.push({
                if b {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }),
        }
    }

    if let Some(t) = point.timestamp {
        line.push(" ".to_string());
        line.push(t.to_string());
    }

    line.push("\n".to_string());

    line.join("")
}

////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct AfluenciaClient {
    host: String,
    database: String,
    port: u32,
    user: Option<String>,
    password: Option<String>,
    use_ssl: bool,
}

pub struct AfluenciaResponse {
    pub status: u16,
    pub body: String,
}

impl AfluenciaClient {
    pub fn new(hostname: &str, port: u32, database: &str) -> AfluenciaClient {
        AfluenciaClient {
            host: String::from(hostname),
            database: String::from(database),
            port,
            user: None,
            password: None,
            use_ssl: false,
        }
    }

    pub fn user(&mut self, user: String) -> &mut AfluenciaClient {
        self.user = Some(user);
        self
    }

    pub fn password(&mut self, password: String) -> &mut AfluenciaClient {
        self.password = Some(password);
        self
    }

    pub fn write_measurement(&self, measurement: DataPoint) {
        let target_url = self.get_write_base_url();
        let measurement_line = line_serialization(measurement);

        //
        static APP_USER_AGENT: &str =
            concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

        //
        match Client::builder()
            .user_agent(APP_USER_AGENT)
            .timeout(Duration::from_secs(10))
            .gzip(true)
            .build()
        {
            Ok(client) => match client.post(&target_url).body(measurement_line).send() {
                Ok(response) => {
                    if response.status().is_server_error() {
                        error!("Could not store entry since the InfluxDB responded with an error")
                    } else {
                        debug!("Successfully stored entry in InfluxDB");
                    }
                }
                Err(_) => {
                    error!("Failed to send data to the InfluxDB server");
                }
            },
            Err(_) => {
                error!("Could not write measurement since the client could not be initialized");
            }
        }
    }

    fn get_write_base_url(&self) -> String {
        let prefix = if self.use_ssl { "https" } else { "http" };

        let mut generated_url = format!(
            "{}://{}:{}/write?db={}",
            prefix, self.host, self.port, self.database
        );

        if let Some(username) = &self.user {
            generated_url = format!("{}&u={}", generated_url, username);
        }

        if let Some(password) = &self.password {
            generated_url = format!("{}&p={}", generated_url, password);
        }

        generated_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_valid_base_url_with_all_parameters_set() {
        let mut client = AfluenciaClient::new("hostname", 1234, "test");
        client
            .user(String::from("username"))
            .password(String::from("password"));

        assert_eq!(
            "http://hostname:1234/write?db=test&u=username&p=password",
            client.get_write_base_url()
        );
    }

    #[test]
    fn generate_valid_base_url_without_authentication() {
        let client = AfluenciaClient::new("hostname", 1234, "test");

        assert_eq!(
            "http://hostname:1234/write?db=test",
            client.get_write_base_url()
        );
    }

    #[test]
    fn generate_valid_base_url_with_just_username_set() {
        let mut client = AfluenciaClient::new("hostname", 1234, "test");
        client.user(String::from("username"));

        assert_eq!(
            "http://hostname:1234/write?db=test&u=username",
            client.get_write_base_url()
        );
    }

    #[test]
    fn generate_valid_base_url_with_just_password_set() {
        let mut client = AfluenciaClient::new("hostname", 1234, "test");
        client.password(String::from("password"));

        assert_eq!(
            "http://hostname:1234/write?db=test&p=password",
            client.get_write_base_url()
        );
    }

    #[test]
    fn line_serialization_of_valid_datapoint_works() {
        let mut test_data_point = DataPoint::new("measurement_name");
        test_data_point.add_tag("testtag", Value::String(String::from("tagvalue")));
        test_data_point.add_field("string_field", Value::String(String::from("string_value")));
        test_data_point.add_field("float_field", Value::Float(1.2345));
        test_data_point.add_field("int_field", Value::Integer(12345));
        test_data_point.add_field("bool_field", Value::Boolean(true));
        test_data_point.add_timestamp(1_234_567_890);

        let serialized_data_point = line_serialization(test_data_point);

        assert_eq!("measurement_name,testtag=tagvalue bool_field=true,float_field=1.2345,int_field=12345i,string_field=\"string_value\" 1234567890\n", serialized_data_point);
    }
}

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
