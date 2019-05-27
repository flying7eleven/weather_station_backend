#![feature(proc_macro_hygiene, decl_macro)]

use log::{debug, info};
use rocket::http::Status;
use rocket::{get, post};
use rocket_codegen::routes;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, WriteLogger};
use std::fs::File;

// a (currently) hard coded list of all valid sensor IDs
static VALID_SENSORS: [&str; 3] = ["DEADBEEF", "DEADC0DE", "ABAD1DEA"];

#[cfg(debug_assertions)]
const LOGGING_LEVEL: LevelFilter = LevelFilter::Trace;

#[cfg(not(debug_assertions))]
const LOGGING_LEVEL: LevelFilter = LevelFilter::Info;

#[derive(Serialize, Deserialize)]
pub struct TemperatureMeasurement {
    pub value: f32,
}

#[get("/temperature")]
pub fn get_current_temperature_measurements() -> Json<Value> {
    Json(json!({}))
}

#[post("/temperature/<sensor>", data = "<temperature>")]
pub fn store_temperature_measurement(
    sensor: String,
    temperature: Json<TemperatureMeasurement>,
) -> Status {
    if !VALID_SENSORS.contains(&&*sensor.to_uppercase()) {
        return Status::BadRequest;
    }
    debug!("Got temperature measurement for sensor {}", sensor);
    Status::Ok
}

fn get_version_str() -> String {
    format!(
        "{}.{}.{}{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH"),
        option_env!("CARGO_PKG_VERSION_PRE").unwrap_or("")
    )
}

fn main() {
    // configure the logging framework and set the corresponding log level
    let logger_init = CombinedLogger::init(vec![
        TermLogger::new(LOGGING_LEVEL, Config::default()).unwrap(),
        WriteLogger::new(
            LOGGING_LEVEL,
            Config::default(),
            File::create("weather_station_backend.log").unwrap(),
        ),
    ]);

    // if we could not configure the logger, terminate!
    if logger_init.is_err() {
        panic!(
            "Could not initialize logger. The reason was: {}",
            logger_init.err().unwrap()
        )
    }

    // tell the user that we started to spin up the API
    info!(
        "Starting up the REST API for the Weather Station in version {}...",
        get_version_str()
    );

    // print all valid sensors
    for sensor_id in VALID_SENSORS.iter() {
        info!("{} is a valid sensor identifier", sensor_id);
    }

    //
    rocket::ignite()
        .mount(
            "/v1",
            routes![
                get_current_temperature_measurements,
                store_temperature_measurement
            ],
        )
        .launch();
}
