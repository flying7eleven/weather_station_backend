use lazy_static::lazy_static;
use rocket::http::Status;
use rocket::post;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

lazy_static! {
    pub static ref CONFIG: Configuration = Configuration::from_default_locations();
}

#[derive(Default, Serialize, Deserialize)]
pub struct Configuration {
    allowed_sensors: Vec<String>,
    database: DatabaseConfiguration,
}

#[derive(Default, Serialize, Deserialize)]
pub struct DatabaseConfiguration {
    host: String,
    port: u16,
    user: String,
    password: String,
    database: String,
}

#[derive(Serialize, Deserialize)]
pub struct Measurement {
    pub sensor: String,
    pub temperature: f64,
    pub humidity: f64,
    pub pressure: f64,
    pub raw_voltage: i32,
    pub charge: f64,
    pub firmware_version: String,
}

impl Configuration {
    pub fn from_default_locations() -> Configuration {
        use log::{debug, error, warn};
        use std::fs::metadata;
        use std::fs::File;

        // determine which configuration file to use
        let mut config_file = "/etc/weather_station_backend/config.yml";
        if metadata(config_file).is_ok() {
            debug!(
                "Found '{}' and using it as a configuration for this instance of the program",
                config_file
            );
        } else if metadata("config.yml").is_ok() {
            debug!("Found config.yml in the current directory and using it as a configuration for this instance of the program");
            config_file = "config.yml";
        } else {
            warn!("Could not find any configuration file, using default values for this instance of the program");
            return Configuration::default();
        }

        // do actually try to read the configuration file and return it if we succeed
        return match File::open(config_file) {
            Ok(file_handle) => {
                return match serde_yaml::from_reader(file_handle) {
                    Ok(configuration) => configuration,
                    Err(error) => {
                        error!("Could not parse '{}' as a configuration file, falling back to default configuration. The error message was: {}", config_file, error);
                        Configuration::default()
                    }
                };
            }
            Err(_) => {
                error!("Could not load '{}' as a configuration file, falling back to default configuration", config_file);
                Configuration::default()
            }
        };
    }

    pub fn is_sensor_allowed(&self, sensor_id: &str) -> bool {
        self.allowed_sensors.contains(&sensor_id.to_string())
    }
}

fn calculate_absolute_humidity(temperature: f64, rel_humidity: f64) -> f64 {
    let a = if temperature >= 0.0 { 7.5 } else { 7.6 };

    let b = if temperature >= 0.0 { 237.3 } else { 240.7 };

    let r_star = 8314.3;
    let m_w = 18.016;
    let t_k = temperature + 273.15;

    let ssd_t = 6.1078 * f64::powf(10.0, (a * temperature) / (b + temperature));
    let dd_t = rel_humidity / 100.0 * ssd_t;

    f64::powf(10.0, 5.0) * m_w / r_star * dd_t / t_k
}

#[post("/sensor/measurement", data = "<measurement>")]
pub async fn store_new_measurement(measurement: Json<Measurement>) -> Status {
    use log::{error, info};
    use sqlx::{query, PgPool};

    if !CONFIG.is_sensor_allowed(&measurement.sensor) {
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

    let pool = PgPool::connect(
        format!(
            "postgres://{}:{}@{}:{}/{}",
            CONFIG.database.user,
            CONFIG.database.password,
            CONFIG.database.host,
            CONFIG.database.port,
            CONFIG.database.database
        )
        .as_str(),
    )
    .await
    .unwrap();

    let insert_query_result = query("INSERT INTO measurements ( id, sensor_id, firmware, timestamp, temperature, rel_humidity, abs_humidity, pressure, raw_voltage, charge ) VALUES ( DEFAULT, $1, $2, NOW(), $3, $4, $5, $6, $7, $8 ) RETURNING id")
        .bind(measurement.sensor.clone())
        .bind(measurement.firmware_version.clone())
        .bind(measurement.temperature)
        .bind(measurement.humidity)
        .bind(abs_humidity)
        .bind(measurement.pressure)
        .bind(measurement.raw_voltage)
        .bind(measurement.charge)
        .fetch_one(&pool)
        .await;

    return match insert_query_result {
        Ok(_) => {
            info!(
                sensor_id=measurement.sensor;
                "Successfully stored measurement in database"
            );
            Status::NoContent
        }
        Err(e) => {
            error!(
                sensor_id=measurement.sensor;
                "Could not store measurement in database: {}", e
            );
            Status::InternalServerError
        }
    };
}
