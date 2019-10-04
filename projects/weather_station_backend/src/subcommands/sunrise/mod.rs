use crate::SunriseSunsetCalculator;
use log::info;

pub fn run_subcommand() {
    let mut test_date = SunriseSunsetCalculator {
        latitude: 51.21875,
        longitude: 6.76341,
        julian_date: 0.0,
    };
    test_date.set_current_date(2019.0, 10, 4);

    info!(
        "Sunrise: {} seconds from midnight",
        test_date.calc_sunrise_utc()
    );
    info!(
        "Sunset: {} seconds from midnight",
        test_date.calc_sunset_utc()
    );
}
