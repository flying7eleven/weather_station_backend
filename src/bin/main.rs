use chrono::Local;
use log::{debug, info, LevelFilter};
use rocket::{catchers, routes};
use std::env;
use weather_station_backend::configuration::Configuration;

#[cfg(debug_assertions)]
const LOGGING_LEVEL: LevelFilter = LevelFilter::Trace;

#[cfg(not(debug_assertions))]
const LOGGING_LEVEL: LevelFilter = LevelFilter::Info;

fn get_version_str() -> String {
    format!(
        "{}.{}.{}{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH"),
        option_env!("CARGO_PKG_VERSION_PRE").unwrap_or("")
    )
}

async fn run_server() {
    // read the configuration file for showing some useful information later on
    let config = Configuration::from_defaut_locations();

    // tell the user that we started to spin up the API
    info!(
        "Starting up the REST API for the Weather Station in version {}...",
        get_version_str()
    );

    // show some confoguration options
    debug!(
        "Writing information to InfluxDB host '{}:{}'",
        config.influx_storage.host, config.influx_storage.port
    );
    debug!(
        "Writing information to InfluxDB database '{}'",
        config.influx_storage.database
    );
    if let Some(user) = config.influx_storage.user {
        debug!("Writing information to InfluxDB with user '{}'", user)
    };
    if config.influx_storage.password.is_some() {
        debug!("Writing information to InfluxDB using a password")
    };

    // print all valid sensors
    for sensor_id in config.allowed_sensors.iter() {
        info!("{} is a valid sensor identifier", sensor_id);
    }

    // initialize the REST part
    let _ = rocket::build()
        .register(
            "/",
            catchers![
                weather_station_backend::routes::not_found,
                weather_station_backend::routes::internal_error,
                weather_station_backend::routes::unauthorized,
                weather_station_backend::routes::forbidden,
                weather_station_backend::routes::unprocessable_entity,
                weather_station_backend::routes::bad_request
            ],
        )
        .mount(
            "/v1",
            routes![
                weather_station_backend::routes::sensor::store_new_measurement,
                weather_station_backend::routes::sensor::get_last_temperature,
            ],
        )
        .launch()
        .await;
}

async fn setup_logger() {
    let _ = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(LOGGING_LEVEL)
        .level_for("hyper", LevelFilter::Warn)
        .level_for("launch", LevelFilter::Warn)
        .level_for("launch_", LevelFilter::Warn)
        .level_for("rocket", LevelFilter::Warn)
        .level_for("reqwest", LevelFilter::Warn)
        .level_for("mio", LevelFilter::Warn)
        .level_for("want", LevelFilter::Warn)
        .level_for("_", LevelFilter::Error)
        .chain(std::io::stdout())
        .apply();
}

#[rocket::main]
async fn main() {
    setup_logger().await;
    run_server().await;
}
