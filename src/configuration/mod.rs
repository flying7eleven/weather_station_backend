use log::error;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs::metadata;
use std::fs::File;
use std::string::ToString;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct InfluxConfiguration {
    pub host: String,
    pub database: String,
    pub port: u32,
    pub user: Option<String>,
    pub password: Option<String>,
    pub use_ssl: bool,
}

impl Default for InfluxConfiguration {
    fn default() -> Self {
        InfluxConfiguration {
            host: "localhost".to_string(),
            database: "database_name".to_string(),
            port: 8086,
            user: None,
            password: None,
            use_ssl: true,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Configuration {
    #[serde(default)]
    pub influx_storage: InfluxConfiguration,

    #[serde(default)]
    pub allowed_sensors: Vec<String>,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            influx_storage: InfluxConfiguration::default(),
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
            return Configuration::from_file("/etc/weather_station_backend/config.yml");
        } else if metadata("config.yml").is_ok() {
            return Configuration::from_file("config.yml");
        }
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
