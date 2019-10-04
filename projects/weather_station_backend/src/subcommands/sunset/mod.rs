use crate::SunriseSunsetCalculator;
use log::info;
use crate::configuration::Configuration;

pub fn run_subcommand() {
    //
    let config = Configuration::from_defaut_locations();

    //
    let mut sunset_calculator = SunriseSunsetCalculator::from(config);
    sunset_calculator.set_current_date(2019.0, 10, 4);

    //
    info!(
        "Sunset: {} seconds from midnight",
        sunset_calculator.calc_sunset_utc()
    );
}
