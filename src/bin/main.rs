use futures::future::FlattenStream;
use futures::stream::Stream;
use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use log::{debug, error, info, warn};
use serde_json::json;
use serde_json::Value;
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, WriteLogger};
use std::convert::TryInto;
use std::fs::File;
use std::str;
use weather_station_backend::boundary::Measurement;

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

fn service_handler(request: Request<Body>) -> Response<Body> {
    let (parts, body) = request.into_parts();
    if parts.uri.path().starts_with("/v1/measurement") && parts.method == Method::POST {
        let sensor_id = parts.uri.path().rsplit('/').next().unwrap_or("");

        // be sure that the sensor is allowed to post to this endpoint
        if !VALID_SENSORS.contains(&sensor_id) {
            warn!(
                "A unknown sensor with the ID {} tried to push its data to this server instance.",
                sensor_id
            );
            return Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Body::empty())
                .unwrap();
        }

        // wait until we received all the content of the body
        let entire_body = body.concat2();
        let response = entire_body.map(|body| {
            match str::from_utf8(body.as_ref()) {
                Ok(v) => {
                    // convert the received measurement to the corresponding object
                    let measurement: Measurement = serde_json::from_str(v).unwrap_or(Measurement {
                        temperature: 0.0,
                        humidity: 0.0,
                        pressure: 0.0,
                    });

                    // temporary log the received values
                    error!(
                        "Temp: {} :: Humidity: {} :: Pressure: {}",
                        measurement.temperature, measurement.humidity, measurement.pressure
                    );

                    // everything was successfully done, we can return a successful state
                    return Response::builder()
                        .status(StatusCode::NO_CONTENT)
                        .body(Body::empty())
                        .unwrap();
                }
                Err(_e) => {
                    error!("Could not convert received request body to a string!");
                    return Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Body::empty())
                        .unwrap();
                }
            }
        });

        return response.wait().unwrap();
    }
    return Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::empty())
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
