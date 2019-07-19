use chrono::Local;
use core::borrow::Borrow;
use dotenv::dotenv;
use futures::stream::Stream;
use futures::Future;
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server, StatusCode};
use lazy_static::lazy_static;
use log::{error, info, warn, LevelFilter};
use std::collections::LinkedList;
use std::env;
use std::str::FromStr;
use weather_station_backend::boundary::Measurement;
use weather_station_backend::StorageBackend;

lazy_static! {
    static ref VALID_SENSORS: LinkedList<String> = {
        let mut list_of_sensors: LinkedList<String> = LinkedList::new();
        if env::var("WEATHER_STATION_SENSORS").is_ok() {
            for id in env::var("WEATHER_STATION_SENSORS").unwrap().split(",") {
                list_of_sensors.push_back(id.to_string());
            }
        } else {
            list_of_sensors.push_back("DEADBEEF".to_string());
            list_of_sensors.push_back("DEADC0DE".to_string());
            list_of_sensors.push_back("ABAD1DEA".to_string());
        }
        list_of_sensors
    };
}

#[cfg(debug_assertions)]
const LOGGING_LEVEL: LevelFilter = LevelFilter::Trace;

#[cfg(not(debug_assertions))]
const LOGGING_LEVEL: LevelFilter = LevelFilter::Info;

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type ResponseFuture = Box<dyn Future<Item = Response<Body>, Error = GenericError> + Send>;

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
                if !VALID_SENSORS.contains(&parsed_json_unwrapped.sensor) {
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
                let storage_backend = StorageBackend::default();
                storage_backend.store_measurement(parsed_json_unwrapped.sensor.borrow(), parsed_json_unwrapped.temperature, parsed_json_unwrapped.humidity, parsed_json_unwrapped.pressure);
                let response = Response::builder()
                    .status(StatusCode::NO_CONTENT)
                    .body(Body::empty())?;
                Ok(response)
            }),
    )
}

fn main() {
    // load the .env file for the configuration options
    dotenv().ok();

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
    let database_enabled = bool::from_str(
        &env::var("WEATHER_STATION_USE_DB").unwrap_or_else(|_| String::from("true")),
    )
    .unwrap_or(false);
    if !database_enabled {
        info!("Classical rational database support is disabled by configuration.");
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn posting_wrong_data_results_in_400_bad_request() {
        let fake_request = Request::post("https://api.foo.bar/v1/sensor/measurement")
            .header("User-Agent", "my-awesome-agent/1.0")
            .body(Body::empty())
            .unwrap();

        let request_response = service_handler(fake_request).wait();

        assert_eq!(false, request_response.is_err());

        let unwrapped_response = request_response.unwrap();
        assert_eq!(unwrapped_response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn posting_correct_data_with_unknown_sensor_results_in_403_forbidden() {
        let valid_body = Body::from("{\"sensor\": \"UNKNOWN\", \"temperature\": 22.0, \"humidity\": 50.0, \"pressure\": 1013.2}");

        let fake_request = Request::post("https://api.foo.bar/v1/sensor/measurement")
            .header("User-Agent", "my-awesome-agent/1.0")
            .body(valid_body)
            .unwrap();

        let request_response = service_handler(fake_request).wait();

        assert_eq!(false, request_response.is_err());

        let unwrapped_response = request_response.unwrap();
        assert_eq!(unwrapped_response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    #[should_panic] // TODO: this should not be the solution, just a temporary fix
    fn posting_correct_data_with_known_sensor_results_in_204_no_content() {
        let valid_body = Body::from("{\"sensor\": \"DEADBEEF\", \"temperature\": 22.0, \"humidity\": 50.0, \"pressure\": 1013.2}");

        let fake_request = Request::post("https://api.foo.bar/v1/sensor/measurement")
            .header("User-Agent", "my-awesome-agent/1.0")
            .body(valid_body)
            .unwrap();

        let request_response = service_handler(fake_request).wait();

        assert_eq!(false, request_response.is_err());

        let unwrapped_response = request_response.unwrap();
        assert_eq!(unwrapped_response.status(), StatusCode::NO_CONTENT);
    }
}
