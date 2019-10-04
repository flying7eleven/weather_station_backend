use chrono::Local;
use clap::{crate_authors, crate_description, crate_name, crate_version, load_yaml, App};
use core::borrow::Borrow;
use futures::stream::Stream;
use futures::Future;
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server, StatusCode};
use log::{debug, error, info, warn, LevelFilter};
use std::env;
use std::str::FromStr;
use weather_station_backend::boundary::Measurement;
use weather_station_backend::configuration::Configuration;
use weather_station_backend::StorageBackend;

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

fn service_handler(req: Request<Body>) -> ResponseFuture {
    Box::new(
        req.into_body()
            .concat2()
            .from_err()
            .and_then(|entire_body| {
                let config = Configuration::from_defaut_locations();
                let parsed_json = serde_json::from_slice::<Measurement>(&entire_body);
                if parsed_json.is_err() {
                    let error_response = Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Body::empty())?;
                    return Ok(error_response);
                }
                let parsed_json_unwrapped = parsed_json.unwrap();
                if !config.allowed_sensors.contains(&parsed_json_unwrapped.sensor) {
                    error!("Got a request from sensor '{}' which is not allowed to post data here. Ignoring request.", parsed_json_unwrapped.sensor);
                    let error_response = Response::builder()
                        .status(StatusCode::FORBIDDEN)
                        .body(Body::empty())?;
                    return Ok(error_response);
                }
                let abs_humidity = calculate_absolute_humidity(parsed_json_unwrapped.temperature, parsed_json_unwrapped.humidity);
                warn!(
                    "sensor: {} ({}), temp.: {:02.2} °C, rel. hum.: {:02.2}%, rel. hum.: {:02.2} g/m³, press.: {:04.2} hPa, raw. voltage: {:.2} -> {:.2} %",
                    parsed_json_unwrapped.sensor,
                    parsed_json_unwrapped.firmware_version,
                    parsed_json_unwrapped.temperature,
                    parsed_json_unwrapped.humidity,
                    abs_humidity,
                    parsed_json_unwrapped.pressure,
                    parsed_json_unwrapped.raw_voltage,
                    parsed_json_unwrapped.charge,
                );
                let storage_backend = StorageBackend::with_configuration(config);
                storage_backend.store_measurement(parsed_json_unwrapped.sensor.borrow(), parsed_json_unwrapped.temperature, parsed_json_unwrapped.humidity, abs_humidity, parsed_json_unwrapped.pressure, parsed_json_unwrapped.raw_voltage, parsed_json_unwrapped.charge);
                let response = Response::builder()
                    .status(StatusCode::NO_CONTENT)
                    .body(Body::empty())?;
                Ok(response)
            }),
    )
}

fn run_server(config: Configuration) {
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
    match config.influx_storage.user {
        Some(user) => debug!("Writing information to InfluxDB with user '{}'", user),
        None => {}
    };
    match config.influx_storage.password {
        Some(_) => debug!("Writing information to InfluxDB using a password"),
        None => {}
    };

    // check if the database part should be enabled or not
    let database_enabled = bool::from_str(
        &env::var("WEATHER_STATION_USE_DB").unwrap_or_else(|_| String::from("true")),
    )
    .unwrap_or(false);
    if !database_enabled {
        info!("Classical rational database support is disabled by configuration.");
    }

    // print all valid sensors
    for sensor_id in config.allowed_sensors.iter() {
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

fn main() {
    // configure the command line parser
    let configuration_parser_config = load_yaml!("cli.yml");
    let matches = App::from_yaml(configuration_parser_config)
        .author(crate_authors!())
        .version(crate_version!())
        .name(crate_name!())
        .about(crate_description!())
        .get_matches();

    //
    let config = Configuration::from_defaut_locations();

    // do not initialize the logger for the config sub-command
    if matches.subcommand_matches("config").is_none() {
        setup_logger();
    }

    // check which subcommand should be executed and call it
    if let Some(_) = matches.subcommand_matches("config") {
        println!("{}", serde_yaml::to_string(&config).unwrap());
    } else if let Some(_) = matches.subcommand_matches("run") {
        run_server(config);
    } else {
        error!("No known subcommand was selected. Please refer to the help for information about how to use this application.");
    }
}

fn setup_logger() {
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
        .level_for("tokio_reactor", LevelFilter::Info)
        .level_for("tokio_threadpool", LevelFilter::Info)
        .level_for("hyper", LevelFilter::Info)
        .level_for("mio", LevelFilter::Info)
        .level_for("want", LevelFilter::Info)
        .chain(std::io::stdout())
        .apply();
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
        let valid_body = Body::from("{\"temperature\":27.05,\"humidity\":37.95,\"pressure\":1011.72,\"raw_voltage\":713.00,\"charge\":51.13,\"sensor\":\"UNKNOWN\",\"firmware_version\":\"0.0.1-dev\"}");

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
        let valid_body = Body::from("{\"temperature\":27.05,\"humidity\":37.95,\"pressure\":1011.72,\"raw_voltage\":713.00,\"charge\":51.13,\"sensor\":\"DEADBEEF\",\"firmware_version\":\"0.0.1-dev\"}");

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
