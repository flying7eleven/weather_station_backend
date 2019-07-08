use dotenv::dotenv;
use hyper::rt::Future;
use hyper::{Body, Client, Method, Request, StatusCode, Uri};
use log::{debug, error};
use std::env;

pub struct AfluenciaClient {
    host: String,
    database: String,
    port: u32,
    user: Option<String>,
    password: Option<String>,
}

impl AfluenciaClient {
    pub fn new(hostname: &str, port: u32, database: &str) -> AfluenciaClient {
        AfluenciaClient {
            host: String::from(hostname),
            database: String::from(database),
            port: port,
            user: None,
            password: None,
        }
    }

    pub fn user<'a>(&'a mut self, user: String) -> &'a mut AfluenciaClient {
        self.user = Some(user);
        self
    }

    pub fn password<'a>(&'a mut self, password: String) -> &'a mut AfluenciaClient {
        self.password = Some(password);
        self
    }

    pub fn write_measurement(&self) {
        let target_url: Uri = self.get_write_base_url().parse().unwrap();
        let mut client = Client::new();

        //
        let mut data_request = Request::new(Body::from("afluencia,mytag=1 myfield=90 1463683075"));
        *data_request.method_mut() = Method::POST;
        *data_request.uri_mut() = target_url.clone();

        //
        client.request(data_request).and_then(|response| {
            if response.status() != StatusCode::NO_CONTENT {
                error!(
                    "Received an invalid response from the InfluxDB backend: {}",
                    response.status()
                );
            } else {
                debug!("Wrote a new measurement into the attached InfluxDB instances.");
            }

            Ok(())
        });
    }

    fn get_write_base_url(&self) -> String {
        let mut generated_url = format!(
            "http://{}:{}/write?db={}",
            self.host, self.port, self.database
        );

        if let Some(username) = &self.user {
            generated_url = format!("{}&u={}", generated_url, username);
        }

        if let Some(password) = &self.password {
            generated_url = format!("{}&p={}", generated_url, password);
        }

        generated_url
    }
}

impl Default for AfluenciaClient {
    fn default() -> Self {
        dotenv().ok();

        AfluenciaClient {
            host: env::var("AFLUENCIA_HOST").expect("AFLUENCIA_HOST must be set"),
            database: env::var("AFLUENCIA_DB").expect("AFLUENCIA_DB must be set"),
            port: env::var("AFLUENCIA_PORT")
                .expect("AFLUENCIA_PORT must be set")
                .parse()
                .unwrap(),
            user: None,
            password: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_valid_write_base_url_with_default_initialization() {
        let client = AfluenciaClient::default();

        assert_eq!(
            "http://localhost:8086/write?db=default",
            client.get_write_base_url()
        );
    }

    #[test]
    fn generate_valid_base_url_with_individual_initialization_and_authentication() {
        let mut client = AfluenciaClient::new("hostname", 1234, "test");
        client
            .user(String::from("username"))
            .password(String::from("password"));

        assert_eq!(
            "http://hostname:1234/write?db=test&u=username&p=password",
            client.get_write_base_url()
        );
    }
}
