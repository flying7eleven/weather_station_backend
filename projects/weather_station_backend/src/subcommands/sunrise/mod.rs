use crate::configuration::Configuration;
use crate::SunriseSunsetCalculator;
use chrono::{Datelike, TimeZone, Utc};
use log::info;

pub fn run_subcommand() {
    // get the current configuration to ensure that we know where we are currently at
    let config = Configuration::from_defaut_locations();

    // get and instance of the calculator
    let sunrise_calculator = SunriseSunsetCalculator::from(config);

    //
    let now = Utc::now();
    let minutes_since_midnight = sunrise_calculator.calc_sunrise_utc(now);
    let hour = (minutes_since_midnight / 60.0).trunc();
    let minute = (minutes_since_midnight - minutes_since_midnight.trunc()) * 60.0;
    let seconds = (minute - minute.trunc()) * 60.0;

    //
    let sunrise = Utc.ymd(now.year(), now.month(), now.day()).and_hms(
        hour as u32,
        minute as u32,
        seconds as u32,
    );

    //
    info!("Sunrise at the configured location is at {}", sunrise);
}
