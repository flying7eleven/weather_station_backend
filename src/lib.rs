use rocket::http::Status;
use rocket::post;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

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
        use log::{debug, warn};
        use std::fs::metadata;

        if metadata("/etc/weather_station_backend/config.yml").is_ok() {
            debug!("Found '/etc/weather_station_backend/config.yml' and using it as a configuration for this instance of the program");
            return Configuration::from_file("/etc/weather_station_backend/config.yml");
        } else if metadata("config.yml").is_ok() {
            debug!("Found config.yml in the current directory and using it as a configuration for this instance of the program");
            return Configuration::from_file("config.yml");
        }
        warn!("Could not find any configuration file, using default values for this instance of the program");
        Configuration::default()
    }

    pub fn from_file(config_file: &str) -> Configuration {
        use log::error;
        use std::fs::File;

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
    use log::{error, info};

    let config = Configuration::from_defaut_locations();

    if !config.allowed_sensors.contains(&measurement.sensor) {
        error!(
            sensor_id=measurement.sensor;
            "Got a request from sensor '{}' which is not allowed to post data here; ignoring request.",
            measurement.sensor
        );
        return Status::Forbidden;
    }

    let abs_humidity = calculate_absolute_humidity(measurement.temperature, measurement.humidity);

    info!(
        sensor_id=measurement.sensor;
        "Received measurement: Sensor: {} ({}), Temperature: {:02.2} °C, Rel. humidity.: {:02.2}%, Abs. humidity.: {:02.2} g/m³, Pressure: {:04.2} hPa, Raw voltage: {:.2} -> {:.2} %",
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
