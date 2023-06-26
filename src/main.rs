async fn run_server() {
    use log::info;
    use rocket::config::{Shutdown, Sig};
    use rocket::routes;
    use rocket::Config as RocketConfig;

    // tell the user that we started to spin up the API
    info!(
        "Starting up the REST API for the Weather Station in version {}.{}.{}{}...",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH"),
        option_env!("CARGO_PKG_VERSION_PRE").unwrap_or("")
    );

    // rocket configuration figment
    let rocket_configuration_figment = RocketConfig::figment()
        .merge(("port", 5471))
        .merge(("address", std::net::Ipv4Addr::new(0, 0, 0, 0)))
        .merge((
            "shutdown",
            Shutdown {
                ctrlc: true,
                signals: {
                    let mut set = std::collections::HashSet::new();
                    set.insert(Sig::Term);
                    set
                },
                grace: 2,
                mercy: 3,
                force: true,
                __non_exhaustive: (),
            },
        ));

    // initialize the REST part
    let _ = rocket::custom(rocket_configuration_figment)
        .mount(
            "/v1",
            routes![weather_station_backend::store_new_measurement],
        )
        .launch()
        .await;
}

async fn setup_logger(verbosity_level: u8) {
    use fenrir_rs::{Fenrir, NetworkingBackend, SerializationFormat};
    use log::LevelFilter;
    use reqwest::Url;

    // create an instance for the Dispatcher to create a new logging configuration
    let mut base_config = fern::Dispatch::new();

    // determine the logging level based on the verbosity the user chose
    base_config = match verbosity_level {
        0 => base_config.level(LevelFilter::Warn),
        1 => base_config.level(LevelFilter::Info),
        2 => base_config.level(LevelFilter::Debug),
        _3_or_more => base_config.level(LevelFilter::Trace),
    };

    //
    let fenrir_builder = Fenrir::builder()
        .endpoint(Url::parse("http://192.168.1.50:3100").unwrap())
        .network(NetworkingBackend::Ureq)
        .format(SerializationFormat::Json)
        .include_level()
        .tag("app", "weather_station_backend");

    #[cfg(debug_assertions)]
    let fenrir = fenrir_builder.tag("environment", "development").build();
    #[cfg(not(debug_assertions))]
    let fenrir = fenrir_builder.tag("environment", "production").build();

    //
    base_config
        .chain(std::io::stdout())
        .chain(Box::new(fenrir) as Box<dyn log::Log>)
        .level_for("hyper", LevelFilter::Off)
        .level_for("rocket", LevelFilter::Off)
        .level_for("reqwest", LevelFilter::Off)
        .level_for("want", LevelFilter::Off)
        .level_for("mio", LevelFilter::Off)
        .level_for("ureq", LevelFilter::Off)
        .apply()
        .unwrap();
}

#[rocket::main]
async fn main() {
    setup_logger(3).await;
    run_server().await;
}
