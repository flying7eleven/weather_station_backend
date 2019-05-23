use log::info;
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, WriteLogger};
use std::fs::File;

#[cfg(debug_assertions)]
fn get_logging_level() -> LevelFilter {
    LevelFilter::Trace
}

#[cfg(not(debug_assertions))]
fn get_logging_level() -> LevelFilter {
    LevelFilter::Info
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
        TermLogger::new(get_logging_level(), Config::default()).unwrap(),
        WriteLogger::new(
            get_logging_level(),
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
}
