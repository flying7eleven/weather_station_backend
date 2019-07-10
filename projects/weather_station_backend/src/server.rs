use crate::boundary::Measurement;
use crate::{StorageBackend, WeatherStationConfiguration};
use core::borrow::Borrow;
use futures::stream::Stream;
use futures::Future;
use hyper::service::Service;
use hyper::{error, Body, Request, Response, StatusCode};
use log::{error, warn};
use std::net::SocketAddr;

// a (currently) hard coded list of all valid sensor IDs
static VALID_SENSORS: [&str; 3] = ["DEADBEEF", "DEADC0DE", "ABAD1DEA"];

pub struct RestApiServer {
    configuration: WeatherStationConfiguration,
    local_address: SocketAddr,
}

impl RestApiServer {
    pub fn new(config: WeatherStationConfiguration, local_address: SocketAddr) -> Self {
        RestApiServer {
            configuration: config,
            local_address,
        }
    }
}

impl Service for RestApiServer {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = error::Error;
    type Future = Box<Future<Item = Response<Self::ResBody>, Error = Self::Error>>;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        Box::new(
            req.into_body()
                .concat2()
                .from_err()
                .and_then(|entire_body| {
                    let parsed_json = serde_json::from_slice::<Measurement>(&entire_body);
                    if parsed_json.is_err() {
                        let error_response = Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::empty())
                            .unwrap();
                        return Ok(error_response);
                    }
                    let parsed_json_unwrapped = parsed_json.unwrap();
                    if !VALID_SENSORS.contains(&&*parsed_json_unwrapped.sensor) {
                        error!("Got a request from sensor '{}' which is not allowed to post data here. Ignoring request.", parsed_json_unwrapped.sensor);
                        let error_response = Response::builder()
                            .status(StatusCode::FORBIDDEN)
                            .body(Body::empty())
                            .unwrap();
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
                        .body(Body::empty())
                        .unwrap();
                    Ok(response)
                }),
        )
    }
}

// https://github.com/skade/hyper-server/blob/master/src/main.rs
