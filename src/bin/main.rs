use chrono::Local;
use core::borrow::Borrow;
use futures::stream::Stream;
use futures::Future;
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server, StatusCode};
use log::{error, info, warn, LevelFilter};
use std::str;
use weather_station_backend::boundary::Measurement;
use weather_station_backend::store_measurement;

// a (currently) hard coded list of all valid sensor IDs
static VALID_SENSORS: [&str; 3] = ["DEADBEEF", "DEADC0DE", "ABAD1DEA"];

#[cfg(debug_assertions)]
const LOGGING_LEVEL: LevelFilter = LevelFilter::Trace;

#[cfg(not(debug_assertions))]
const LOGGING_LEVEL: LevelFilter = LevelFilter::Info;

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type ResponseFuture = Box<Future<Item = Response<Body>, Error = GenericError> + Send>;

fn get_version_str() -> String {
    format!(
        "{}.{}.{}{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH"),
        option_env!("CARGO_PKG_VERSION_PRE").unwrap_or("")
    )
}

fn service_handler(req: Request<Body>) -> ResponseFuture {
    Box::new(
        req.into_body()
            .concat2()
            .from_err()
            .and_then(|entire_body| {
                let parsed_json = serde_json::from_slice::<Measurement>(&entire_body);
                if parsed_json.is_err() {
                    let error_response = Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Body::empty())?;
                    return Ok(error_response);
                }
                let parsed_json_unwrapped = parsed_json.unwrap();
                if !VALID_SENSORS.contains(&&*parsed_json_unwrapped.sensor) {
                    error!("Got a request from sensor '{}' which is not allowed to post data here. Ignoring request.", parsed_json_unwrapped.sensor);
                    let error_response = Response::builder()
                        .status(StatusCode::FORBIDDEN)
                        .body(Body::empty())?;
                    return Ok(error_response);
                }
                warn!(
                    "sensor: {}, temp.: {:02.2}, hum.: {:02.2}, press.: {:04.2}",
                    parsed_json_unwrapped.sensor,
                    parsed_json_unwrapped.temperature,
                    parsed_json_unwrapped.humidity,
                    parsed_json_unwrapped.pressure
                );
                let _measurement_entry = store_measurement(parsed_json_unwrapped.sensor.borrow(), parsed_json_unwrapped.temperature.borrow(), parsed_json_unwrapped.humidity.borrow(), parsed_json_unwrapped.pressure.borrow());
                let response = Response::builder()
                    .status(StatusCode::NO_CONTENT)
                    .body(Body::empty())?;
                Ok(response)
            }),
    )
}

fn main() {
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
