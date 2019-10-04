use crate::configuration::Configuration;
use afluencia::{AfluenciaClient, DataPoint, Value};
use chrono::Local;
use std::clone::Clone;

pub mod boundary;
pub mod configuration;
pub mod subcommands;

pub struct StorageBackend {
    configuration: Configuration,
}

impl StorageBackend {
    pub fn with_configuration(config: Configuration) -> StorageBackend {
        StorageBackend {
            configuration: config,
        }
    }

    pub fn store_measurement(
        &self,
        sensor: &str,
        temperature: f32,
        rel_humidity: f32,
        abs_humidity: f32,
        pressure: f32,
        voltage: f32,
        charge: f32,
    ) {
        // get the current time as an over-all time measurement
        let measurement_time = Local::now().naive_utc();

        // define the required data structure for the InfluxDB
        let mut influx_measurement = DataPoint::new("weather_measurement");
        influx_measurement.add_tag("sensor", Value::String(String::from(sensor)));
        influx_measurement.add_field("temperature", Value::Float(f64::from(temperature)));
        influx_measurement.add_field("rel_humidity", Value::Float(f64::from(rel_humidity)));
        influx_measurement.add_field("abs_humidity", Value::Float(f64::from(abs_humidity)));
        influx_measurement.add_field("pressure", Value::Float(f64::from(pressure)));
        influx_measurement.add_field("raw_battery_voltage", Value::Float(f64::from(voltage)));
        influx_measurement.add_field("battery_charge", Value::Float(f64::from(charge)));
        influx_measurement.add_field("on_battery", Value::Boolean(false));
        influx_measurement.add_timestamp(measurement_time.timestamp_nanos());

        // create an instance of the influx client
        let mut influx_client = AfluenciaClient::new(
            self.configuration.influx_storage.host.as_str(),
            self.configuration.influx_storage.port,
            self.configuration.influx_storage.database.as_str(),
        );

        // check if a username and password can be set, if so, do so :D
        if self.configuration.influx_storage.user.is_some() {
            let user_optional = self.configuration.influx_storage.user.clone();
            influx_client.user(user_optional.unwrap());
        }
        if self.configuration.influx_storage.password.is_some() {
            let password_optional = self.configuration.influx_storage.password.clone();
            influx_client.password(password_optional.unwrap());
        }

        // write the measurement to the database
        influx_client.write_measurement(influx_measurement);
    }
}

// sunrise calculations based on https://github.com/buelowp/sunset/blob/master/src/SunSet.cpp
struct SunriseSunsetCalculator {
    julian_date: f64,
    latitude: f64,
    longitude: f64,
}

impl SunriseSunsetCalculator {
    fn calc_mean_obliquity_of_ecliptic(&self, t: f64) -> f64 {
        let seconds = 21.448 - t * (46.815_0 + t * (0.000_59 - t * (0.001_813)));
        23.0 + (26.0 + (seconds / 60.0)) / 60.0
    }

    fn calc_geom_mean_long_sun(&self, t: f64) -> f64 {
        //
        let mut l = 280.466_46 + t * (36_000.769_83 + 0.000_303_2 * t);

        //
        loop {
            if l > 360.0 {
                l -= 360.0;
            } else {
                break;
            }
        }

        //
        loop {
            if l < 0.0 {
                l += 360.0;
            } else {
                break;
            }
        }

        //
        l
    }

    fn calc_obliquity_correction(&self, t: f64) -> f64 {
        let e0 = self.calc_mean_obliquity_of_ecliptic(t);
        let omega = 125.04 - 1934.136 * t;
        e0 + 0.002_56 * omega.to_radians().cos()
    }

    fn calc_eccentricity_earth_orbit(&self, t: f64) -> f64 {
        0.016_708_634 - t * (0.000_042_037 + 0.000_000_126_7 * t)
    }

    fn calc_geom_mean_anomaly_sun(&self, t: f64) -> f64 {
        357.529_11 + t * (35_999.050_29 - 0.000_153_7 * t)
    }

    fn calc_equation_of_time(&self, t: f64) -> f64 {
        let epsilon = self.calc_obliquity_correction(t);
        let l0 = self.calc_geom_mean_long_sun(t);
        let e = self.calc_eccentricity_earth_orbit(t);
        let m = self.calc_geom_mean_anomaly_sun(t);
        let mut y = (epsilon.to_radians() / 2.0).tan();

        y *= y;

        let sin2l0 = (2.0 * l0.to_radians()).sin();
        let sinm = m.to_radians().sin();
        let cos2l0 = (2.0 * l0.to_radians()).cos();
        let sin4l0 = (4.0 * l0.to_radians()).sin();
        let sin2m = (2.0 * m.to_radians()).sin();
        let etime = y * sin2l0 - 2.0 * e * sinm + 4.0 * e * y * sinm * cos2l0
            - 0.5 * y * y * sin4l0
            - 1.25 * e * e * sin2m;

        etime.to_degrees() * 4.0 // in minutes of time
    }

    fn calc_time_julian_cent(&self, jd: f64) -> f64 {
        (jd - 2_451_545.0) / 36_525.0
    }

    fn calc_sun_eq_of_center(&self, t: f64) -> f64 {
        let m = self.calc_geom_mean_anomaly_sun(t);
        let mrad = m.to_radians();
        let sinm = mrad.sin();
        let sin2m = (mrad + mrad).sin();
        let sin3m = (mrad + mrad + mrad).sin();

        sinm * (1.914_602 - t * (0.004_817 + 0.000_014 * t))
            + sin2m * (0.019_993 - 0.000_101 * t)
            + sin3m * 0.000_289
    }

    fn calc_sun_true_long(&self, t: f64) -> f64 {
        let l0 = self.calc_geom_mean_long_sun(t);
        let c = self.calc_sun_eq_of_center(t);

        l0 + c // in degrees
    }

    fn calc_sun_apparent_long(&self, t: f64) -> f64 {
        let o = self.calc_sun_true_long(t);
        let omega = 125.04 - 1934.136 * t;
        o - 0.005_69 - 0.004_78 * omega.to_radians().sin() // in degrees
    }

    fn calc_sun_declination(&self, t: f64) -> f64 {
        let e = self.calc_obliquity_correction(t);
        let lambda = self.calc_sun_apparent_long(t);

        let sint = e.to_radians().sin() * lambda.to_radians().sin();
        sint.asin().to_degrees() // in degrees
    }

    fn calc_hour_angle_sunrise(&self, lat: f64, solar_dec: f64) -> f64 {
        let lat_rad = lat.to_radians();
        let sd_rad = solar_dec.to_radians();
        let z: f64 = 90.833;
        (z.to_radians().cos() / (lat_rad.cos() * sd_rad.cos()) - lat_rad.tan() * sd_rad.tan())
            .acos() // in radians
    }

    fn calc_hour_angle_sunset(&self, lat: f64, solar_dec: f64) -> f64 {
        let lat_rad = lat.to_radians();
        let sd_rad = solar_dec.to_radians();
        let z: f64 = 90.833;
        -(z.to_radians().cos() / (lat_rad.cos() * sd_rad.cos()) - lat_rad.tan() * sd_rad.tan())
            .acos() // in radians
    }

    fn calc_jd(&self, year_in: f64, month_in: i32, day: i32) -> f64 {
        let mut year = year_in;
        let mut month = month_in;

        if month <= 2 {
            year -= 1.0;
            month += 12;
        }

        let a = (year / 100.0).floor();
        let b = 2.0 - a + (a / 4.0).floor();

        (365.25 * (year + 4716.0)).floor()
            + (30.600_1 * (f64::from(month) + 1.0)).floor()
            + f64::from(day)
            + b
            - 1_524.5
    }

    fn calc_jdfrom_julian_cent(&self, t: f64) -> f64 {
        t * 36_525.0 + 2_451_545.0
    }

    fn calc_sunrise_utc(&self) -> f64 {
        let t = self.calc_time_julian_cent(self.julian_date);
        // first pass to approximate sunrise
        let mut eq_time = self.calc_equation_of_time(t);
        let mut solar_dec = self.calc_sun_declination(t);
        let mut hour_angle = self.calc_hour_angle_sunrise(self.latitude, solar_dec);
        let mut delta = self.longitude + hour_angle.to_degrees();
        let mut time_diff = 4.0 * delta; // in minutes of time
        let time_utc = 720.0 - time_diff - eq_time; // in minutes
        let newt = self.calc_time_julian_cent(self.calc_jdfrom_julian_cent(t) + time_utc / 1_440.0);

        eq_time = self.calc_equation_of_time(newt);
        solar_dec = self.calc_sun_declination(newt);

        hour_angle = self.calc_hour_angle_sunrise(self.latitude, solar_dec);
        delta = self.longitude + hour_angle.to_degrees();
        time_diff = 4.0 * delta;

        720.0 - time_diff - eq_time // return time in minutes from midnight
    }

    fn calc_sunset_utc(&self) -> f64 {
        let t = self.calc_time_julian_cent(self.julian_date);
        // first pass to approximate sunset
        let mut eq_time = self.calc_equation_of_time(t);
        let mut solar_dec = self.calc_sun_declination(t);
        let mut hour_angle = self.calc_hour_angle_sunset(self.latitude, solar_dec);
        let mut delta = self.longitude + hour_angle.to_degrees();
        let mut time_diff = 4.0 * delta; // in minutes of time
        let mut time_utc = 720.0 - time_diff - eq_time; // in minutes
        let newt = self.calc_time_julian_cent(self.calc_jdfrom_julian_cent(t) + time_utc / 1_440.0);

        eq_time = self.calc_equation_of_time(newt);
        solar_dec = self.calc_sun_declination(newt);

        hour_angle = self.calc_hour_angle_sunset(self.latitude, solar_dec);
        delta = self.longitude + hour_angle.to_degrees();
        time_diff = 4.0 * delta;
        time_utc = 720.0 - time_diff - eq_time; // in minutes

        time_utc
        // time_utc + (60 * tzOffset) // return time in minutes from midnight
    }

    pub fn set_current_date(&mut self, year: f64, month: i32, day: i32) {
        self.julian_date = self.calc_jd(year, month, day);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculating_sunrise_in_utc_seconds_works_for_duesseldorf_germany() {
        let mut test_date = SunriseSunsetCalculator {
            latitude: 51.21875,
            longitude: 6.76341,
            julian_date: 0.0,
        };
        test_date.set_current_date(2019.0, 10, 4);

        assert_eq!((test_date.calc_sunrise_utc() - 337.668) > 0.0, true);
        assert_eq!((test_date.calc_sunrise_utc() - 337.668) < 0.0001, true);
    }

    #[test]
    fn calculating_sunset_in_utc_seconds_works_for_duesseldorf_germany() {
        let mut test_date = SunriseSunsetCalculator {
            latitude: 51.21875,
            longitude: 6.76341,
            julian_date: 0.0,
        };
        test_date.set_current_date(2019.0, 10, 4);

        assert_eq!((test_date.calc_sunset_utc() - 1024.9458) > 0.0, true);
        assert_eq!((test_date.calc_sunset_utc() - 1024.9458) < 0.0001, true);
    }
}
