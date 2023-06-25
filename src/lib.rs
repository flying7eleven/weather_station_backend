use log::{error, info, warn};
use rocket::catch;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post};
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs::metadata;
use std::fs::File;
use std::string::ToString;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Configuration {
    #[serde(default)]
    pub allowed_sensors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Measurement {
    pub sensor: String,
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub raw_voltage: f32,
    pub charge: f32,
    pub firmware_version: String,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            allowed_sensors: vec![
                "DEADBEEF".to_string(),
                "BEEFCACE".to_string(),
                "BADDCAFE".to_string(),
            ],
        }
    }
}

impl Configuration {
    pub fn from_defaut_locations() -> Configuration {
        if metadata("/etc/weather_station_backend/config.yml").is_ok() {
            info!("Found '/etc/weather_station_backend/config.yml' and using it as a configuration for this instance of the program");
            return Configuration::from_file("/etc/weather_station_backend/config.yml");
        } else if metadata("config.yml").is_ok() {
            info!("Found config.yml in the current directory and using it as a configuration for this instance of the program");
            return Configuration::from_file("config.yml");
        }
        info!("Could not find any configuration file, using default values for this instance of the program");
        Configuration::default()
    }

    pub fn from_file(config_file: &str) -> Configuration {
        match File::open(config_file) {
            Ok(file_handle) => {
                let read_configuration: Configuration =
                    serde_yaml::from_reader(file_handle).unwrap();
                return read_configuration;
            },
            Err(_) => error!("Could not load '{}' as a configuration file, falling back to default configuration", config_file),
        }
        Configuration::default()
    }

    pub fn from_yaml(configuration: &str) -> Configuration {
        let read_configuration: Configuration = serde_yaml::from_str(configuration).unwrap();
        read_configuration
    }
}

#[catch(500)]
pub fn internal_error() -> &'static str {
    ""
}

#[catch(401)]
pub fn unauthorized() -> &'static str {
    ""
}

#[catch(403)]
pub fn forbidden() -> &'static str {
    ""
}

#[catch(404)]
pub fn not_found() -> &'static str {
    ""
}

#[catch(422)]
pub fn unprocessable_entity() -> &'static str {
    ""
}

#[catch(400)]
pub fn bad_request() -> &'static str {
    ""
}

fn calculate_absolute_humidity(temperature: f32, rel_humidity: f32) -> f32 {
    let a = if temperature >= 0.0 { 7.5 } else { 7.6 };

    let b = if temperature >= 0.0 { 237.3 } else { 240.7 };

    let r_star = 8314.3;
    let m_w = 18.016;
    let t_k = temperature + 273.15;

    let ssd_t = 6.1078 * f32::powf(10.0, (a * temperature) / (b + temperature));
    let dd_t = rel_humidity / 100.0 * ssd_t;

    f32::powf(10.0, 5.0) * m_w / r_star * dd_t / t_k
}

#[post("/sensor/measurement", data = "<measurement>")]
pub fn store_new_measurement(measurement: Json<Measurement>) -> Status {
    let config = Configuration::from_defaut_locations();

    if !config.allowed_sensors.contains(&measurement.sensor) {
        error!("Got a request from sensor '{}' which is not allowed to post data here. Ignoring request.", measurement.sensor);
        return Status::Forbidden;
    }

    let abs_humidity = calculate_absolute_humidity(measurement.temperature, measurement.humidity);

    warn!(
        "sensor: {} ({}), temp.: {:02.2} °C, rel. hum.: {:02.2}%, rel. hum.: {:02.2} g/m³, press.: {:04.2} hPa, raw. voltage: {:.2} -> {:.2} %",
        measurement.sensor,
        measurement.firmware_version,
        measurement.temperature,
        measurement.humidity,
        abs_humidity,
        measurement.pressure,
        measurement.raw_voltage,
        measurement.charge,
    );

    // let storage_backend = StorageBackend::with_configuration(config);
    // storage_backend.store_measurement(
    //     measurement.sensor.borrow(),
    //     measurement.temperature,
    //     measurement.humidity,
    //     abs_humidity,
    //     measurement.pressure,
    //     measurement.raw_voltage,
    //     measurement.charge,
    // );
    // Status::NoContent
    Status::NotImplemented
}

#[get("/sensor/measurement/temperature")]
pub fn get_last_temperature() -> Status {
    Status::NotImplemented
}
