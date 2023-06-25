use crate::boundary::Measurement;
use crate::configuration::Configuration;
use crate::StorageBackend;
use log::{error, warn};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post};
use std::borrow::Borrow;

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

    let storage_backend = StorageBackend::with_configuration(config);
    storage_backend.store_measurement(
        measurement.sensor.borrow(),
        measurement.temperature,
        measurement.humidity,
        abs_humidity,
        measurement.pressure,
        measurement.raw_voltage,
        measurement.charge,
    );
    Status::NoContent
}

#[get("/sensor/measurement/temperature")]
pub fn get_last_temperature() -> Status {
    Status::NotImplemented
}
