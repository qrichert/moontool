//! Command Line Interface for moon.rs.

use moontool::moon::{MoonCalendar, MoonPhase, UTCDateTime};
use std::{env, process};

#[derive(Debug, Eq, PartialEq)]
struct Config {
    datetime: Option<String>,
    help: bool,
    version: bool,
}

impl Config {
    fn new(args: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut config = Self {
            datetime: None,
            help: false,
            version: false,
        };

        for arg in args.skip(1) {
            if arg == "-h" || arg == "--help" {
                config.help = true;
                continue;
            }

            if arg == "-v" || arg == "--version" {
                config.version = true;
                continue;
            }

            if arg.starts_with("--") || arg.starts_with('-') && arg.parse::<i64>().is_err() {
                return Err(format!("Unknown argument '{arg}'."));
            }

            // Silently ignore extra datetime arguments.
            if config.datetime.is_none() {
                config.datetime = Some(arg);
            }
        }

        Ok(config)
    }
}

#[cfg(not(tarpaulin_include))]
fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(2);
    });

    // TODO: --json, --pretty
    if config.help {
        println!("{}", help_message());
        return;
    }
    if config.version {
        println!("{}", version_message());
        return;
    }

    let datetime = if let Some(ref datetime) = config.datetime {
        try_parse_datetime(datetime)
    } else {
        Some(get_now())
    };

    let Some(datetime) = datetime else {
        eprintln!("Error reading date and time from input.");
        process::exit(2);
    };

    for_datetime(&datetime);
}

fn help_message() -> String {
    format!(
        "\
usage: {bin} [-h] [] [DATETIME] [±TIMESTAMP]

optional arguments:
  -h, --help            show this help message and exit
  -v, --version         show the version and exit
  []                    without arguments, defaults to now
  [DATETIME]            local datetime (e.g., 1994-12-22T14:53:34+01:00)
  [±TIMESTAMP]          Unix timestamp (e.g., 788104414)",
        bin = env!("CARGO_BIN_NAME")
    )
}

fn version_message() -> String {
    format!("{} {}", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION"))
}

#[cfg(not(tarpaulin_include))]
fn get_now() -> UTCDateTime {
    UTCDateTime::now()
}

fn try_parse_datetime(datetime: &str) -> Option<UTCDateTime> {
    if let Some(datetime) = try_from_timestamp(datetime) {
        Some(datetime)
    } else {
        try_from_datetime(datetime)
    }
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

#[cfg(not(tarpaulin_include))]
fn for_datetime(datetime: &UTCDateTime) {
    let mphase = MoonPhase::for_datetime(datetime);
    let mcal = MoonCalendar::for_datetime(datetime);

    println!("\n{mphase}\n");
    if let Ok(mcal) = mcal {
        println!("{mcal}\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Config.

    #[test]
    fn no_arguments() {
        let args = vec![String::new()].into_iter();
        let config = Config::new(args).unwrap();

        assert_eq!(
            config,
            Config {
                datetime: None,
                help: false,
                version: false,
            }
        );
    }

    #[test]
    fn no_arguments_and_no_executable() {
        let args = vec![].into_iter();
        let config = Config::new(args).unwrap();

        assert_eq!(
            config,
            Config {
                datetime: None,
                help: false,
                version: false,
            }
        );
    }

    #[test]
    fn help_full() {
        let args = vec![String::new(), String::from("--help")].into_iter();
        let config = Config::new(args).unwrap();

        assert!(config.help);
    }

    #[test]
    fn help_short() {
        let args = vec![String::new(), String::from("-h")].into_iter();
        let config = Config::new(args).unwrap();

        assert!(config.help);
    }

    #[test]
    fn help_message_contains_options() {
        let message = help_message();

        dbg!(&message);
        assert!(message.contains("-h, --help"));
        assert!(message.contains("-v, --version"));
        assert!(message.contains("[DATETIME]"));
        assert!(message.contains("[±TIMESTAMP]"));
    }

    #[test]
    fn version_full() {
        let args = vec![String::new(), String::from("--version")].into_iter();
        let config = Config::new(args).unwrap();

        assert!(config.version);
    }

    #[test]
    fn version_short() {
        let args = vec![String::new(), String::from("-v")].into_iter();
        let config = Config::new(args).unwrap();

        assert!(config.version);
    }

    #[test]
    fn version_message_contains_binary_name_and_version() {
        let message = version_message();

        dbg!(&message);
        assert!(message.contains(env!("CARGO_BIN_NAME")));
        assert!(message.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn datetime() {
        let args = vec![String::new(), String::from("2024-10-13")].into_iter();
        let config = Config::new(args).unwrap();

        assert_eq!(config.datetime, Some(String::from("2024-10-13")));
    }

    #[test]
    fn timestamp() {
        let args = vec![String::new(), String::from("1715791943")].into_iter();
        let config = Config::new(args).unwrap();

        assert_eq!(config.datetime, Some(String::from("1715791943")));
    }

    #[test]
    fn timestamp_negative() {
        // Because it could be mistaken for an argument.
        let args = vec![String::new(), String::from("-1715791943")].into_iter();
        let config = Config::new(args).unwrap();

        assert_eq!(config.datetime, Some(String::from("-1715791943")));
    }

    #[test]
    fn error_invalid_argument_full() {
        // Because it could be mistaken for an argument.
        let args = vec![String::new(), String::from("--invalid")].into_iter();
        let config = Config::new(args);

        assert!(config.is_err());
        assert!(config.err().unwrap().contains("'--invalid'"));
    }

    #[test]
    fn error_invalid_argument_short() {
        // Because it could be mistaken for an argument.
        let args = vec![String::new(), String::from("-i")].into_iter();
        let config = Config::new(args);

        assert!(config.is_err());
        assert!(config.err().unwrap().contains("'-i'"));
    }

    // Main.

    #[test]
    fn try_parse_datetime_timestamp() {
        let dt = try_parse_datetime("966600000").unwrap();

        assert_eq!(dt, UTCDateTime::from((2000, 8, 18, 5, 12, 0, 0)));
    }

    #[test]
    fn try_parse_datetime_datetime() {
        let dt = try_parse_datetime("1964-12-20T04:35:00Z").unwrap();

        assert_eq!(dt, UTCDateTime::from((1964, 12, 20, 0, 4, 35, 0)));
    }

    #[test]
    fn try_parse_datetime_error() {
        let dt = try_parse_datetime("invalid");

        assert!(dt.is_none());
    }

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
