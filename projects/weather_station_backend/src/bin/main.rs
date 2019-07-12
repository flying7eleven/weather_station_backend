use chrono::Local;
use futures::future::Future;
use hyper::Server;
use log::{error, info, LevelFilter};
use weather_station_backend::server::RestService;
use weather_station_backend::WeatherStationConfiguration;

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

fn main() {
    // load the current configuration into memory
    let configuration: WeatherStationConfiguration =
        confy::load("weather_station_backend").unwrap();

    // configure the logging framework and set the corresponding log level
    let log_initialization = fern::Dispatch::new()
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
        .level_for("tokio_reactor", LevelFilter::Info)
        .level_for("tokio_threadpool", LevelFilter::Info)
        .level_for("hyper", LevelFilter::Info)
        .level_for("mio", LevelFilter::Info)
        .level_for("want", LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file("weather_station_backend.log").unwrap())
        .apply();

    // ensure the logging engine works, otherwise we should rather terminate here
    if log_initialization.is_err() {
        panic!("Could not initialize logging. Terminating!");
    }

    // tell the user that we started to spin up the API
    info!(
        "Starting up the REST API for the Weather Station in version {}...",
        get_version_str()
    );

    // check if the database part should be enabled or not
    if !configuration.rational_db_enabled {
        info!("Classical rational database support is disabled by configuration.");
    }

    // configure the server and start it up
    let server_address = ([0, 0, 0, 0], 8000).into();
    let server = Server::bind(&server_address)
        .serve(move || RestService::new(&configuration))
        .map_err(|e| error!("server error: {}", e));
    hyper::rt::run(server);
}
