use crate::boundary::Measurement;
use crate::{StorageBackend, WeatherStationConfiguration};
use core::borrow::Borrow;
use core::fmt;
use futures::stream::Stream;
use futures::{future, Future, IntoFuture};
use hyper::service::Service;
use hyper::{Body, Request, Response, StatusCode};
use log::{error, warn};
use std::error::Error;

// a (currently) hard coded list of all valid sensor IDs
static VALID_SENSORS: [&str; 3] = ["DEADBEEF", "DEADC0DE", "ABAD1DEA"];

#[derive(Debug)]
pub enum Never {}

impl fmt::Display for Never {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        match *self {}
    }
}

impl Error for Never {
    fn description(&self) -> &str {
        match *self {}
    }
}

pub struct RestService {
    _configuration: WeatherStationConfiguration,
}

impl RestService {
    pub fn new(config: &WeatherStationConfiguration) -> RestService {
        RestService {
            _configuration: config.clone(),
        }
    }
}

impl IntoFuture for RestService {
    type Future = future::FutureResult<Self::Item, Self::Error>;
    type Item = Self;
    type Error = Never;

    fn into_future(self) -> Self::Future {
        future::ok(self)
    }
}

impl Service for RestService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Box<dyn Future<Item = Response<Self::ResBody>, Error = Self::Error> + Send>;

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


                    let storage_backend = StorageBackend::default();
                    let _measurement_entry = storage_backend.store_measurement(parsed_json_unwrapped.sensor.borrow(), parsed_json_unwrapped.temperature, parsed_json_unwrapped.humidity, parsed_json_unwrapped.pressure);

                    let response = Response::builder()
                        .status(StatusCode::NO_CONTENT)
                        .body(Body::empty())?;
                    return Ok(response);
                }),
        )
    }
}
