use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use log::{debug, error, info};
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

fn get_version_str() -> String {
    format!(
        "{}.{}.{}{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH"),
        option_env!("CARGO_PKG_VERSION_PRE").unwrap_or("")
    )
}

fn store_measurement(sensor: &str, body: &Body) -> Response<Body> {
    // be sure that the sensor is allowed to post to this endpoint
    if !VALID_SENSORS.contains(&sensor) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::from(""))
            .unwrap();
    }

    // everything was successfully done, we can return a successful state
    return Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(Body::from(""))
        .unwrap();
}

fn service_handler(request: Request<Body>) -> Response<Body> {
    if request.uri().path().starts_with("/v1/measurement") && request.method() == Method::POST {
        let sensor_id = request.uri().path().rsplit('/').next().unwrap_or("");
        return store_measurement(sensor_id, request.body());
    }
    return Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from(""))
        .unwrap();
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
    let new_service = || service_fn_ok(service_handler);
    let server = Server::bind(&server_address)
        .serve(new_service)
        .map_err(|e| error!("server error: {}", e));
    hyper::rt::run(server);
}
