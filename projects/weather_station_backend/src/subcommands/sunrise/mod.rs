use crate::SunriseSunsetCalculator;
use log::info;
use crate::configuration::Configuration;

pub fn run_subcommand() {
    //
    let config = Configuration::from_defaut_locations();

    //
    let mut sunrise_calculator = SunriseSunsetCalculator::from(config);
    sunrise_calculator.set_current_date(2019.0, 10, 4);

    //
    info!(
        "Sunrise: {} seconds from midnight",
        sunrise_calculator.calc_sunrise_utc()
    );
}
