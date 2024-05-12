//! Command Line Interface for moon.rs.

use moontool::moon::{MoonCalendar, MoonPhase, UTCDateTime};
use std::{env, process};

#[cfg(not(tarpaulin_include))]
fn main() {
    let mut args = env::args().skip(1);

    let Some(arg) = args.next() else {
        for_now();
        return;
    };

    // TODO: --json, --pretty
    if arg == "--help" || arg == "-h" {
        print_help();
        return;
    }

    let datetime = if let Some(datetime) = try_from_timestamp(&arg) {
        datetime
    } else if let Some(datetime) = try_from_datetime(&arg) {
        datetime
    } else {
        eprintln!("Error reading date and time from input.");
        process::exit(2);
    };

    for_custom_datetime(&datetime);
}

#[cfg(not(tarpaulin_include))]
fn print_help() {
    println!("usage: moontool [-h] [] [DATETIME] [±TIMESTAMP]\n");
    println!("optional arguments:");
    println!("  -h, --help            show this help message and exit");
    println!("  []                    without arguments, defaults to now");
    println!("  [DATETIME]            local datetime (e.g., 1994-12-22T14:53:34+01:00)");
    println!("  [±TIMESTAMP]          Unix timestamp (e.g., 788104414)");
}

#[cfg(not(tarpaulin_include))]
fn for_now() {
    println!("\n{}\n", MoonPhase::now());

    if let Ok(mcal) = MoonCalendar::now() {
        println!("{mcal}\n");
    }
}

#[cfg(not(tarpaulin_include))]
fn for_custom_datetime(datetime: &UTCDateTime) {
    println!("\n{}\n", MoonPhase::for_datetime(datetime));

    if let Ok(mcal) = MoonCalendar::for_datetime(datetime) {
        println!("{mcal}\n");
    };
}

fn try_from_timestamp(timestamp: &str) -> Option<UTCDateTime> {
    let Ok(timestamp) = timestamp.parse::<i64>() else {
        return None;
    };
    UTCDateTime::try_from(timestamp).ok()
}

fn try_from_datetime(datetime: &str) -> Option<UTCDateTime> {
    UTCDateTime::try_from(datetime).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_timestamp_positive() {
        let dt = try_from_timestamp("966600000").unwrap();

        assert_eq!(dt, UTCDateTime::from((2000, 8, 18, 5, 12, 0, 0)));
    }

    #[test]
    fn try_from_timestamp_zero() {
        let dt = try_from_timestamp("0").unwrap();

        assert_eq!(dt, UTCDateTime::from((1970, 1, 1, 4, 0, 0, 0)));
    }

    #[test]
    fn try_from_timestamp_negative() {
        let dt = try_from_timestamp("-58200600").unwrap();

        assert_eq!(dt, UTCDateTime::from((1968, 2, 27, 2, 9, 10, 0)));
    }

    #[test]
    fn try_from_timestamp_error_too_big_for_i64() {
        let dt = try_from_timestamp("99999999999999999999");

        assert!(dt.is_none());
    }

    #[test]
    fn try_from_timestamp_error_bad_timestamp() {
        let dt = try_from_timestamp(&i64::MAX.to_string());

        assert!(dt.is_none());
    }

    #[test]
    fn try_from_datetime_regular() {
        let dt = try_from_datetime("1964-12-20T04:35:00Z").unwrap();

        assert_eq!(dt, UTCDateTime::from((1964, 12, 20, 0, 4, 35, 0)));
    }

    #[test]
    fn try_from_datetime_implicit_utc() {
        let dt = try_from_datetime("1964-12-20T04:35:00").unwrap();

        assert_eq!(dt, UTCDateTime::from((1964, 12, 20, 0, 4, 35, 0)));
    }

    #[test]
    fn try_from_datetime_offset() {
        let dt = try_from_datetime("1964-12-20T05:35:00+01:00").unwrap();

        assert_eq!(dt, UTCDateTime::from((1964, 12, 20, 0, 4, 35, 0)));
    }

    #[test]
    fn try_from_datetime_error_invalid_string() {
        let dt = try_from_datetime("1964-12-20T05-35-00");

        assert!(dt.is_none());
    }

    #[test]
    fn from_date() {
        let d = try_from_datetime("1938-07-15").unwrap();

        assert_eq!(d, UTCDateTime::from((1938, 7, 15, 5, 0, 0, 0)));
    }

    #[test]
    fn from_date_error_invalid_string() {
        let d = try_from_datetime("1938:07:15");

        assert!(d.is_none());
    }
}
