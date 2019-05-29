use futures::{future, Future};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use log::{error, info};
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, WriteLogger};
use std::fs::File;
use std::str;

// a (currently) hard coded list of all valid sensor IDs
static VALID_SENSORS: [&str; 3] = ["DEADBEEF", "DEADC0DE", "ABAD1DEA"];

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

fn service_handler(
    req: Request<Body>,
) -> Box<Future<Item = Response<Body>, Error = hyper::Error> + Send> {
    let mut response = Response::new(Body::empty());

    match (req.method(), req.uri().path()) {
        (&Method::POST, "/v1/measurement") => {
            *response.status_mut() = StatusCode::NO_CONTENT;
        }

        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };

    Box::new(future::ok(response))
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

    // configure the server and start it up
    let server_address = ([0, 0, 0, 0], 8000).into();
    let new_service = || service_fn(service_handler);
    let server = Server::bind(&server_address)
        .serve(new_service)
        .map_err(|e| error!("server error: {}", e));
    hyper::rt::run(server);
}
