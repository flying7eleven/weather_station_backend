use crate::configuration::Configuration;
use chrono::{Datelike, TimeZone, Utc};
use log::info;
use solnedgang::SunriseSunsetCalculator;

pub fn run_subcommand() {
    //
    // get the current configuration to ensure that we know where we are currently at
    let config = Configuration::from_defaut_locations();

    // get and instance of the calculator
    let sunset_calculator = SunriseSunsetCalculator {
        latitude: f64::from(config.sunset_sunrise_annotations.latitude),
        longitude: f64::from(config.sunset_sunrise_annotations.longitude),
    };

    //
    let now = Utc::now();
    let minutes_since_midnight = sunset_calculator.calc_sunset_utc(now);
    let hour = (minutes_since_midnight / 60.0).trunc();
    let minute = (minutes_since_midnight - minutes_since_midnight.trunc()) * 60.0;
    let seconds = (minute - minute.trunc()) * 60.0;

    //
    let sunset = Utc.ymd(now.year(), now.month(), now.day()).and_hms(
        hour as u32,
        minute as u32,
        seconds as u32,
    );

    //
    info!("Sunset at the configured location is at {}", sunset);
}
