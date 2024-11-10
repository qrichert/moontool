//! Date and time handling functions.

use std::error::Error;
use std::fmt;
use std::str::FromStr;

use crate::moon;

const MONAME: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

const DAYNAME: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

#[derive(Debug)]
pub struct DateTimeError(pub &'static str);

impl fmt::Display for DateTimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for DateTimeError {}

fn monthname(month: u32) -> Result<&'static str, DateTimeError> {
    let month = month as usize;
    if month < 1 || month > MONAME.len() {
        return Err(DateTimeError("invalid month"));
    }
    Ok(MONAME[month - 1])
}

fn dayname(day: u32) -> Result<&'static str, DateTimeError> {
    let day = day as usize;
    if day >= DAYNAME.len() {
        return Err(DateTimeError("invalid day"));
    }
    Ok(DAYNAME[day])
}

// Everything `time-rs` is confined in here.

#[cfg(not(tarpaulin_include))]
fn utcdatetime_now() -> UTCDateTime {
    let now = time::OffsetDateTime::now_utc();
    offsetdatetime_to_utcdatetime(&now)
}

fn utcdatetime_to_timestamp(datetime: &UTCDateTime) -> Result<i64, DateTimeError> {
    let datetime = utcdatetime_to_offsetdatetime(datetime)?;
    Ok(datetime.unix_timestamp())
}

fn timestamp_to_utcdatetime(timestamp: i64) -> Result<UTCDateTime, DateTimeError> {
    let Ok(datetime) = time::OffsetDateTime::from_unix_timestamp(timestamp) else {
        return Err(DateTimeError("timestamp is out of range"));
    };
    Ok(offsetdatetime_to_utcdatetime(&datetime))
}

fn iso_datetime_string_to_utcdatetime(iso_datetime: &str) -> Result<UTCDateTime, DateTimeError> {
    let datetime = if iso_datetime.contains('T') || iso_datetime.contains('t') {
        parse_datetime(iso_datetime)
    } else {
        parse_date(iso_datetime)
    };

    let Ok(datetime) = datetime else {
        return Err(DateTimeError("invalid datetime string"));
    };

    let datetime = datetime.to_offset(time::UtcOffset::UTC);

    Ok(UTCDateTime::from(datetime))
}

fn parse_datetime(datetime: &str) -> Result<time::OffsetDateTime, DateTimeError> {
    let mut datetime = datetime.to_owned();
    // Implicit UTC if no offset provided.
    if !datetime.ends_with('Z') && !datetime.ends_with('z') && !datetime.contains('+') {
        datetime.push('Z');
    }
    let format = time::format_description::well_known::Rfc3339;
    time::OffsetDateTime::parse(&datetime, &format)
        .map_or(Err(DateTimeError("error parsing datetime")), Ok)
}

fn parse_date(date: &str) -> Result<time::OffsetDateTime, DateTimeError> {
    static DATE_FORMAT: std::sync::OnceLock<
        Vec<time::format_description::BorrowedFormatItem<'static>>,
    > = std::sync::OnceLock::new();

    let format = DATE_FORMAT.get_or_init(|| {
        time::format_description::parse_borrowed::<1>("[year]-[month]-[day]")
            .expect("format is valid")
    });

    let Ok(date) = time::Date::parse(date, &format) else {
        return Err(DateTimeError("error parsing date"));
    };
    Ok(time::OffsetDateTime::new_utc(date, time::Time::MIDNIGHT))
}

fn weekday_for_utcdatetime(datetime: &UTCDateTime) -> Result<u32, DateTimeError> {
    let datetime = utcdatetime_to_offsetdatetime(datetime)?;
    Ok(u32::from(datetime.weekday().number_days_from_sunday()))
}

#[cfg(not(tarpaulin_include))]
fn weekday_for_localdatetime(datetime: &LocalDateTime) -> Result<u32, DateTimeError> {
    let datetime = localdatetime_to_offsetdatetime(datetime)?;
    Ok(u32::from(datetime.weekday().number_days_from_sunday()))
}

#[cfg(not(tarpaulin_include))]
fn utcdatetime_to_localdatetime(datetime: &UTCDateTime) -> Result<LocalDateTime, DateTimeError> {
    let utc = utcdatetime_to_offsetdatetime(datetime)?;
    let Ok(local_offset) = time::UtcOffset::local_offset_at(utc) else {
        return Err(DateTimeError("error obtaining local offset"));
    };

    let local = utc.to_offset(local_offset);

    Ok(LocalDateTime {
        year: local.year(),
        month: u32::from(local.month() as u8),
        day: u32::from(local.day()),
        hour: u32::from(local.hour()),
        minute: u32::from(local.minute()),
        second: u32::from(local.second()),
    })
}

// If it gets truncated, values are wrong anyway.
#[allow(clippy::cast_possible_truncation)]
fn utcdatetime_to_offsetdatetime(
    datetime: &UTCDateTime,
) -> Result<time::OffsetDateTime, DateTimeError> {
    let Ok(month) = time::Month::try_from(datetime.month as u8) else {
        return Err(DateTimeError("invalid month"));
    };
    let Ok(date) = time::Date::from_calendar_date(datetime.year, month, datetime.day as u8) else {
        return Err(DateTimeError("invalid date"));
    };
    let Ok(time) = time::Time::from_hms(
        datetime.hour as u8,
        datetime.minute as u8,
        datetime.second as u8,
    ) else {
        return Err(DateTimeError("invalid time"));
    };

    Ok(time::OffsetDateTime::new_utc(date, time))
}

#[cfg(not(tarpaulin_include))]
fn localdatetime_to_utcdatetime(datetime: &LocalDateTime) -> Result<UTCDateTime, DateTimeError> {
    let local = localdatetime_to_offsetdatetime(datetime)?;
    let utc = local.to_offset(time::UtcOffset::UTC);
    Ok(offsetdatetime_to_utcdatetime(&utc))
}

/// # Warning
///
/// The returned [`OffsetDateTime`](time::OffsetDateTime) will be in
/// _local_ offset.
///
/// Keep in mind that passing it like this to
/// `offsetdatetime_to_utcdatetime()` will panic. It must be converted
/// to UTC offset first.
#[cfg(not(tarpaulin_include))]
fn localdatetime_to_offsetdatetime(
    datetime: &LocalDateTime,
) -> Result<time::OffsetDateTime, DateTimeError> {
    // Treat local datetime as UTC. This is INVALID for now, but it lets
    // us create an `OffsetDateTime` with the correct date and time
    // components (but offset is WRONG).
    let local = utcdatetime_to_offsetdatetime(&UTCDateTime::from_ymdhms(
        datetime.year,
        datetime.month,
        datetime.day,
        datetime.hour,
        datetime.minute,
        datetime.second,
    ))?;

    let Ok(local_offset) = time::UtcOffset::current_local_offset() else {
        return Err(DateTimeError("error obtaining local offset"));
    };

    // Replace offset with local offset, WITHOUT changing the date and
    // time components. This is now CORRECT.
    let local = local.replace_offset(local_offset);

    Ok(local)
}

#[cfg(not(tarpaulin_include))]
fn local_offset_as_string() -> Result<String, DateTimeError> {
    let Ok(local_offset) = time::UtcOffset::current_local_offset() else {
        return Err(DateTimeError("error obtaining local offset"));
    };
    Ok(local_offset.to_string())
}

/// # Panics
///
/// Panics if provided [`OffsetDateTime`](time::OffsetDateTime) is not
/// in UTC offset. Otherwise, resulting [`UTCDateTime`]s would have
/// wrong values.
///
/// Dealing with non-UTC datetimes is the exception by a large margin.
/// Panicking instead of performing the conversion keeps the API clean
/// and reduces confusion.
///
/// The idea is you _can_ manipulate non-UTC datetimes, but you have to
/// explicitly convert them to UTC first if you want to interact with
/// the regular API. Because the regular API explicitly always deals
/// with UTC datetimes.
fn offsetdatetime_to_utcdatetime(datetime: &time::OffsetDateTime) -> UTCDateTime {
    debug_assert_eq!(datetime.offset(), time::UtcOffset::UTC);

    UTCDateTime {
        year: datetime.year(),
        month: u32::from(datetime.month() as u8),
        day: u32::from(datetime.day()),
        hour: u32::from(datetime.hour()),
        minute: u32::from(datetime.minute()),
        second: u32::from(datetime.second()),
    }
}

/// Internal date and time representation.
///
/// This is not really meant to be created by the user. Its main purpose
/// is to be part of return values from library functions—though there
/// are scenarios where it _can_ be useful to create a `UTCDateTime`
/// manually.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UTCDateTime {
    pub year: i32,
    /// `[1;12]`
    pub month: u32,
    /// `[1;31]`
    pub day: u32,
    /// `[0;23]`
    pub hour: u32,
    /// `[0;59]`
    pub minute: u32,
    /// `[0;59]`
    pub second: u32,
}

impl UTCDateTime {
    /// Name of month in English.
    ///
    /// # Errors
    ///
    /// Errors if month is not a number between 1 and 12.
    pub fn monthname(&self) -> Result<&'static str, DateTimeError> {
        monthname(self.month)
    }

    /// Day of week as offset from Sunday.
    ///
    /// - 0 = Sunday
    /// - 1 = Monday
    /// - ...
    /// - 6 = Saturday
    ///
    /// # Errors
    ///
    /// Errors if date or time is invalid.
    pub fn weekday(&self) -> Result<u32, DateTimeError> {
        weekday_for_utcdatetime(self)
    }

    /// Name of day in English.
    ///
    /// # Errors
    ///
    /// Errors if date or time is invalid.
    pub fn dayname(&self) -> Result<&'static str, DateTimeError> {
        dayname(self.weekday()?)
    }
}

impl UTCDateTime {
    #[cfg(not(tarpaulin_include))]
    #[must_use]
    pub fn now() -> Self {
        utcdatetime_now()
    }

    /// From raw Year, Month, Day, Hour, Minute, Second values.
    #[must_use]
    pub fn from_ymdhms(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }

    /// From ISO 8601 date or datetime string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use moontool::moon::UTCDateTime;
    /// _ = UTCDateTime::from_iso_string("2024-06-14").unwrap();
    /// _ = UTCDateTime::from_iso_string("2024-06-14T21:21:00").unwrap();
    /// _ = UTCDateTime::from_iso_string("2024-06-14T21:21:00Z").unwrap();
    /// _ = UTCDateTime::from_iso_string("2024-06-14T19:21:00+02:00").unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Errors if input string in invalid format.
    pub fn from_iso_string(iso_string: &str) -> Result<Self, DateTimeError> {
        Self::try_from(iso_string)
    }

    /// Convert Unix timestamp to `UTCDateTime`.
    ///
    /// # Errors
    ///
    /// Errors if result date or time is invalid (e.g., `2024-01-42`).
    pub fn from_timestamp(timestamp: i64) -> Result<Self, DateTimeError> {
        let dt = timestamp_to_utcdatetime(timestamp)?;
        Ok(dt)
    }

    /// Convert `UTCDateTime` to Unix timestamp.
    ///
    /// # Errors
    ///
    /// Errors if date or time is invalid (e.g., `2024-01-42`).
    pub fn to_timestamp(&self) -> Result<i64, DateTimeError> {
        let timestamp = utcdatetime_to_timestamp(self)?;
        Ok(timestamp)
    }

    /// Convert astronomical Julian date to `UTCDateTime`.
    #[must_use]
    pub fn from_julian_date(julian_date: f64) -> Self {
        moon::jtouct(julian_date)
    }

    /// Convert `UTCDateTime` to astronomical Julian date.
    ///
    /// Working with Julian dates makes it very convenient to do maths
    /// with dates. You can add, subtract and iterate of days very
    /// simply, and then convert the date back to a normal date when
    /// done.
    ///
    /// > The Julian day is the continuous count of days since the
    /// > beginning of the Julian period, and is used primarily by
    /// > astronomers, and in software for easily calculating elapsed
    /// > days between two events.
    /// >
    /// > The Julian day number (JDN) is the integer assigned to a whole
    /// > solar day in the Julian day count starting from noon Universal
    /// > Time, with Julian day number 0 assigned to the day starting at
    /// > noon on Monday, January 1, 4713 BC (-4712), proleptic Julian
    /// > calendar.
    /// >
    /// > The Julian date (JD) of any instant is the Julian day number
    /// > plus the fraction of a day since the preceding noon in
    /// > Universal Time.
    /// >
    /// > — Wikipedia
    ///
    /// # Examples
    ///
    /// ```
    /// # use moontool::moon::UTCDateTime;
    /// #
    /// let day1: UTCDateTime = "2024-01-01T00:00:00".parse().unwrap();
    /// let day1 = day1.to_julian_date();
    ///
    /// let day2: UTCDateTime = "2024-06-11T00:00:00".parse().unwrap();
    /// let day2 = day2.to_julian_date();
    ///
    /// assert_eq!(day1, 2460310.5);
    /// assert_eq!(day2, 2460472.5);
    /// assert_eq!(day2 - day1, 162.0); // Days elapsed.
    /// ```
    #[must_use]
    pub fn to_julian_date(&self) -> f64 {
        moon::jtime(self)
    }

    /// Convert `UTCDateTime` to civil Julian date.
    ///
    /// Conventional Julian date starts at noon (12h). This function
    /// returns the civil, or 0h-variant, of the Julian date which
    /// starts at midnight.
    ///
    /// The relation between the two is the following:
    ///
    /// ```text
    /// Civil Julian day = Julian day + 0.5
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// # use moontool::moon::UTCDateTime;
    /// #
    /// let day1: UTCDateTime = "2024-01-01T00:00:00".parse().unwrap();
    /// let day1 = day1.to_civil_julian_date();
    ///
    /// let day2: UTCDateTime = "2024-06-11T00:00:00".parse().unwrap();
    /// let day2 = day2.to_civil_julian_date();
    ///
    /// assert_eq!(day1, 2460311.0);
    /// assert_eq!(day2, 2460473.0);
    /// assert_eq!(day2 - day1, 162.0); // Days elapsed.
    /// ```
    #[must_use]
    pub fn to_civil_julian_date(&self) -> f64 {
        self.to_julian_date() + 0.5
    }
}

impl FromStr for UTCDateTime {
    type Err = DateTimeError;

    fn from_str(datetime: &str) -> Result<Self, Self::Err> {
        let dt = iso_datetime_string_to_utcdatetime(datetime)?;
        Ok(dt)
    }
}

impl TryFrom<&str> for UTCDateTime {
    type Error = DateTimeError;

    fn try_from(datetime: &str) -> Result<Self, Self::Error> {
        datetime.parse()
    }
}

impl TryFrom<&LocalDateTime> for UTCDateTime {
    type Error = DateTimeError;

    #[cfg(not(tarpaulin_include))]
    fn try_from(datetime: &LocalDateTime) -> Result<Self, Self::Error> {
        localdatetime_to_utcdatetime(datetime)
    }
}

impl From<time::OffsetDateTime> for UTCDateTime {
    fn from(dt: time::OffsetDateTime) -> Self {
        offsetdatetime_to_utcdatetime(&dt)
    }
}

impl TryFrom<&UTCDateTime> for time::OffsetDateTime {
    type Error = DateTimeError;

    fn try_from(dt: &UTCDateTime) -> Result<Self, Self::Error> {
        utcdatetime_to_offsetdatetime(dt)
    }
}

impl fmt::Display for UTCDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:0>4}-{:0>2}-{:0>2}T{:0>2}:{:0>2}:{:0>2}Z",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

/// Internal local date and time representation.
///
/// This is mostly meant to be used for display, or condition behaviour
/// according to the user's local time. It's just a helper.
///
/// # Warning
///
/// [`LocalDateTime`] is NOT offset aware. It is always assumed to be
/// "current local offset". Thus, if the local offset changes (because
/// of DST or because you've sent the values to someone in a different
/// timezone), the values will be WRONG.
///
/// Always store or send [`UTCDateTime`] instead. `LocalDateTime` is
/// meant for solving problems "right now, on this machine". Nothing
/// more.
///
/// # Examples
///
/// ```rust
/// # use moontool::moon::{LocalDateTime, UTCDateTime};
/// let landing = UTCDateTime::try_from("1969-07-20T20:17:40Z").unwrap();
///
/// let (month, day) = LocalDateTime::try_from(&landing).map_or_else(
///     |_| (landing.month, landing.day), // Fall back to UTC.
///     |local| (local.month, local.day),
/// );
///
/// if month == 7 && day == 20 {
///     // Apollo 11 anniversary.
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalDateTime {
    pub year: i32,
    /// `[1;12]`
    pub month: u32,
    /// `[1;31]`
    pub day: u32,
    /// `[0;23]`
    pub hour: u32,
    /// `[0;59]`
    pub minute: u32,
    /// `[0;59]`
    pub second: u32,
}

impl LocalDateTime {
    /// Name of month in English.
    ///
    /// # Errors
    ///
    /// Errors if month is not a number between 1 and 12.
    #[cfg(not(tarpaulin_include))]
    pub fn monthname(&self) -> Result<&'static str, DateTimeError> {
        monthname(self.month)
    }

    /// Day of week as offset from Sunday.
    ///
    /// - 0 = Sunday
    /// - 1 = Monday
    /// - ...
    /// - 6 = Saturday
    ///
    /// # Errors
    ///
    /// Errors if date or time is invalid.
    #[cfg(not(tarpaulin_include))]
    pub fn weekday(&self) -> Result<u32, DateTimeError> {
        weekday_for_localdatetime(self)
    }

    /// Name of day in English.
    ///
    /// # Errors
    ///
    /// Errors if date or time is invalid.
    #[cfg(not(tarpaulin_include))]
    pub fn dayname(&self) -> Result<&'static str, DateTimeError> {
        dayname(self.weekday()?)
    }
}

impl TryFrom<&UTCDateTime> for LocalDateTime {
    type Error = DateTimeError;

    #[cfg(not(tarpaulin_include))]
    fn try_from(datetime: &UTCDateTime) -> Result<Self, Self::Error> {
        utcdatetime_to_localdatetime(datetime)
    }
}

impl fmt::Display for LocalDateTime {
    #[cfg(not(tarpaulin_include))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:0>4}-{:0>2}-{:0>2}T{:0>2}:{:0>2}:{:0>2}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )?;
        if let Ok(local_offset) = local_offset_as_string() {
            write!(f, "{local_offset}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_almost_eq {
        ($a:expr, $b:expr) => {
            assert!(($a - $b).abs() < f64::EPSILON, "{} != {}", $a, $b);
        };
    }

    // Date/time error.

    #[test]
    fn datetimeerror_format() {
        let error = DateTimeError("an error has occurred");

        assert_eq!(error.to_string(), "an error has occurred");
    }

    // Date/time utils.

    #[test]
    fn month_number_to_name() {
        assert!(monthname(0).is_err());
        assert_eq!(monthname(1).unwrap(), "January");
        assert_eq!(monthname(2).unwrap(), "February");
        assert_eq!(monthname(3).unwrap(), "March");
        assert_eq!(monthname(4).unwrap(), "April");
        assert_eq!(monthname(5).unwrap(), "May");
        assert_eq!(monthname(6).unwrap(), "June");
        assert_eq!(monthname(7).unwrap(), "July");
        assert_eq!(monthname(8).unwrap(), "August");
        assert_eq!(monthname(9).unwrap(), "September");
        assert_eq!(monthname(10).unwrap(), "October");
        assert_eq!(monthname(11).unwrap(), "November");
        assert_eq!(monthname(12).unwrap(), "December");
        assert!(monthname(13).is_err());
    }

    #[test]
    fn day_number_to_name() {
        assert_eq!(dayname(0).unwrap(), "Sunday");
        assert_eq!(dayname(1).unwrap(), "Monday");
        assert_eq!(dayname(2).unwrap(), "Tuesday");
        assert_eq!(dayname(3).unwrap(), "Wednesday");
        assert_eq!(dayname(4).unwrap(), "Thursday");
        assert_eq!(dayname(5).unwrap(), "Friday");
        assert_eq!(dayname(6).unwrap(), "Saturday");
        assert!(dayname(7).is_err());
    }

    #[test]
    fn utcdatetime_to_timestamp_regular() {
        let t =
            utcdatetime_to_timestamp(&UTCDateTime::from_ymdhms(2024, 4, 30, 18, 21, 42)).unwrap();

        assert_eq!(t, 1_714_501_302);
    }

    #[test]
    fn utcdatetime_to_timestamp_zero() {
        let t = utcdatetime_to_timestamp(&UTCDateTime::from_ymdhms(1970, 1, 1, 0, 0, 0)).unwrap();

        assert_eq!(t, 0);
    }

    #[test]
    fn utcdatetime_to_timestamp_negative() {
        let t = utcdatetime_to_timestamp(&UTCDateTime::from_ymdhms(1940, 10, 13, 0, 0, 0)).unwrap();

        assert_eq!(t, -922_060_800);
    }

    #[test]
    fn timestamp_to_utcdatetime_regular() {
        let dt = timestamp_to_utcdatetime(1_714_501_302).unwrap();

        assert_eq!(dt, UTCDateTime::from_ymdhms(2024, 4, 30, 18, 21, 42));
    }

    #[test]
    fn timestamp_to_utcdatetime_zero() {
        let dt = timestamp_to_utcdatetime(0).unwrap();

        assert_eq!(dt, UTCDateTime::from_ymdhms(1970, 1, 1, 0, 0, 0));
    }

    #[test]
    fn timestamp_to_utcdatetime_negative() {
        let dt = timestamp_to_utcdatetime(-922_060_800).unwrap();

        assert_eq!(dt, UTCDateTime::from_ymdhms(1940, 10, 13, 0, 0, 0));
    }

    #[test]
    fn timestamp_to_utcdatetime_bad_timestamp() {
        let dt = timestamp_to_utcdatetime(i64::MAX);

        assert!(dt.is_err());
    }

    #[test]
    fn iso_datetime_string_to_utcdatetime_from_datetime_utc() {
        let dt = iso_datetime_string_to_utcdatetime("1964-12-20T04:35:00Z").unwrap();

        assert_eq!(dt, UTCDateTime::from_ymdhms(1964, 12, 20, 4, 35, 0));
    }

    #[test]
    fn iso_datetime_string_to_utcdatetime_from_datetime_utc_lowercase() {
        let dt = iso_datetime_string_to_utcdatetime("1964-12-20t04:35:00z").unwrap();

        assert_eq!(dt, UTCDateTime::from_ymdhms(1964, 12, 20, 4, 35, 0));
    }

    #[test]
    fn iso_datetime_string_to_utcdatetime_from_datetime_implicit_utc() {
        let dt = iso_datetime_string_to_utcdatetime("1964-12-20T04:35:00").unwrap();

        assert_eq!(dt, UTCDateTime::from_ymdhms(1964, 12, 20, 4, 35, 0));
    }

    #[test]
    fn iso_datetime_string_to_utcdatetime_from_datetime_offset() {
        let dt = iso_datetime_string_to_utcdatetime("1964-12-20T05:35:00+01:00").unwrap();

        assert_eq!(dt, UTCDateTime::from_ymdhms(1964, 12, 20, 4, 35, 0));
    }

    #[test]
    fn iso_datetime_string_to_utcdatetime_from_datetime_error_invalid_string() {
        let dt = iso_datetime_string_to_utcdatetime("1964-12-20T05-35-00");

        assert!(dt.is_err());
    }

    #[test]
    fn iso_datetime_string_to_utcdatetime_from_date() {
        let d = iso_datetime_string_to_utcdatetime("1938-07-15").unwrap();

        assert_eq!(d, UTCDateTime::from_ymdhms(1938, 7, 15, 0, 0, 0));
    }

    #[test]
    fn iso_datetime_string_to_utcdatetime_from_date_error_invalid_string() {
        let d = iso_datetime_string_to_utcdatetime("1938:07:15");

        assert!(d.is_err());
    }

    #[test]
    fn weekday_for_datetime_regular() {
        assert_eq!(
            weekday_for_utcdatetime(&UTCDateTime::from_ymdhms(2024, 5, 13, 20, 47, 23)).unwrap(),
            1
        ); // Monday
        assert_eq!(
            weekday_for_utcdatetime(&UTCDateTime::from_ymdhms(2024, 5, 14, 20, 47, 23)).unwrap(),
            2
        ); // Tuesday
        assert_eq!(
            weekday_for_utcdatetime(&UTCDateTime::from_ymdhms(2024, 5, 15, 20, 47, 23)).unwrap(),
            3
        ); // Wednesday
        assert_eq!(
            weekday_for_utcdatetime(&UTCDateTime::from_ymdhms(2024, 5, 16, 20, 47, 23)).unwrap(),
            4
        ); // Thursday
        assert_eq!(
            weekday_for_utcdatetime(&UTCDateTime::from_ymdhms(2024, 5, 17, 20, 47, 23)).unwrap(),
            5
        ); // Friday
        assert_eq!(
            weekday_for_utcdatetime(&UTCDateTime::from_ymdhms(2024, 5, 18, 20, 47, 23)).unwrap(),
            6
        ); // Saturday
        assert_eq!(
            weekday_for_utcdatetime(&UTCDateTime::from_ymdhms(2024, 5, 19, 20, 47, 23)).unwrap(),
            0
        ); // Sunday
    }

    #[test]
    fn weekday_for_datetime_error() {
        let weekday = weekday_for_utcdatetime(&UTCDateTime::from_ymdhms(2024, 5, 99, 20, 47, 23));

        assert!(weekday.is_err());
    }

    #[test]
    fn utcdatetime_to_offsetdatetime_regular() {
        let odt =
            utcdatetime_to_offsetdatetime(&UTCDateTime::from_ymdhms(1938, 7, 15, 0, 0, 0)).unwrap();

        assert_eq!(
            odt,
            time::OffsetDateTime::new_utc(
                time::Date::from_calendar_date(1938, time::Month::July, 15).unwrap(),
                time::Time::from_hms(0, 0, 0).unwrap()
            )
        );
    }

    #[test]
    fn utcdatetime_to_offsetdatetime_bad_month() {
        let odt = utcdatetime_to_offsetdatetime(&UTCDateTime::from_ymdhms(1938, 9999, 15, 0, 0, 0));

        assert!(odt.is_err());
    }

    #[test]
    fn utcdatetime_to_offsetdatetime_bad_date() {
        let odt = utcdatetime_to_offsetdatetime(&UTCDateTime::from_ymdhms(1938, 7, 255, 0, 0, 0));

        assert!(odt.is_err());
    }

    #[test]
    fn utcdatetime_to_offsetdatetime_bad_time() {
        let odt = utcdatetime_to_offsetdatetime(&UTCDateTime::from_ymdhms(1938, 7, 15, 255, 0, 0));

        assert!(odt.is_err());
    }

    #[test]
    fn offsetdatetime_to_utcdatetime_regular() {
        let dt = offsetdatetime_to_utcdatetime(&time::OffsetDateTime::new_utc(
            time::Date::from_calendar_date(1938, time::Month::July, 15).unwrap(),
            time::Time::from_hms(0, 0, 0).unwrap(),
        ));

        assert_eq!(dt, UTCDateTime::from_ymdhms(1938, 7, 15, 0, 0, 0));
    }

    // UTCDateTime.

    #[test]
    #[allow(clippy::many_single_char_names)]
    fn every_way_of_creating_utcdatetime_gives_same_result() {
        let a = UTCDateTime {
            year: 1968,
            month: 2,
            day: 27,
            hour: 9,
            minute: 10,
            second: 0,
        };
        let b = UTCDateTime::from_ymdhms(1968, 2, 27, 9, 10, 0);
        let c = "1968-02-27T09:10:00Z".parse::<UTCDateTime>().unwrap();
        let d = UTCDateTime::from_iso_string("1968-02-27T09:10:00Z").unwrap();
        let e = UTCDateTime::try_from("1968-02-27T09:10:00Z").unwrap();
        let f = UTCDateTime::from(time::OffsetDateTime::new_utc(
            time::Date::from_calendar_date(1968, time::Month::February, 27).unwrap(),
            time::Time::from_hms(9, 10, 0).unwrap(),
        ));
        let g = UTCDateTime::from_timestamp(-58_200_600).unwrap();
        let h = UTCDateTime::from_julian_date(2_439_913.881_944_444_5);

        assert!([b, c, d, e, f, g, h].iter().all(|x| *x == a));
    }

    #[test]
    fn utcdatetime_from_iso_string_date() {
        let a = UTCDateTime {
            year: 2024,
            month: 6,
            day: 14,
            hour: 0,
            minute: 0,
            second: 0,
        };
        let b = "2024-06-14".parse::<UTCDateTime>().unwrap();

        assert!(std::iter::once(&b).all(|x| *x == a));
    }

    #[test]
    fn utcdatetime_from_iso_string_datetime() {
        let a = UTCDateTime {
            year: 2024,
            month: 6,
            day: 14,
            hour: 21,
            minute: 21,
            second: 0,
        };
        let b = "2024-06-14T21:21:00".parse::<UTCDateTime>().unwrap();
        let c = "2024-06-14T21:21:00Z".parse::<UTCDateTime>().unwrap();
        let d = "2024-06-14T23:21:00+02:00".parse::<UTCDateTime>().unwrap();

        assert!([b, c, d].iter().all(|x| *x == a));
    }

    #[test]
    fn utcdatetime_try_from_timestamp_positive() {
        let dt = UTCDateTime::from_timestamp(966_600_000).unwrap();

        assert_eq!(dt, UTCDateTime::from_ymdhms(2000, 8, 18, 12, 0, 0));
    }

    #[test]
    fn utcdatetime_try_from_timestamp_zero() {
        let dt = UTCDateTime::from_timestamp(0).unwrap();

        assert_eq!(dt, UTCDateTime::from_ymdhms(1970, 1, 1, 0, 0, 0));
    }

    #[test]
    fn utcdatetime_try_from_timestamp_negative() {
        let dt = UTCDateTime::from_timestamp(-58_200_600).unwrap();

        assert_eq!(dt, UTCDateTime::from_ymdhms(1968, 2, 27, 9, 10, 0));
    }

    #[test]
    fn utcdatetime_to_timestamp_positive() {
        let dt = UTCDateTime::from_ymdhms(2000, 8, 18, 12, 0, 0);

        assert_eq!(dt.to_timestamp().unwrap(), 966_600_000);
    }

    #[test]
    fn utcdatetime_to_timestamp_zero_() {
        let dt = UTCDateTime::from_ymdhms(1970, 1, 1, 0, 0, 0);

        assert_eq!(dt.to_timestamp().unwrap(), 0);
    }

    #[test]
    fn utcdatetime_to_timestamp_negative_() {
        let dt = UTCDateTime::from_ymdhms(1968, 2, 27, 9, 10, 0);

        assert_eq!(dt.to_timestamp().unwrap(), -58_200_600);
    }

    #[test]
    fn utcdatetime_from_julian_date_regular() {
        let dt = UTCDateTime::from_julian_date(2_460_473.196_55);

        assert_eq!(dt, UTCDateTime::from_ymdhms(2024, 6, 11, 16, 43, 2));
    }

    #[test]
    fn utcdatetime_from_julian_date_zero() {
        let dt = UTCDateTime::from_julian_date(0.0);

        assert_eq!(dt, UTCDateTime::from_ymdhms(-4712, 1, 1, 12, 0, 0));
    }

    #[test]
    fn utcdatetime_to_julian_date_regular() {
        let dt = UTCDateTime::from_ymdhms(2024, 6, 11, 16, 43, 2);

        assert_almost_eq!(dt.to_julian_date(), 2_460_473.196_550_925_7);
    }

    #[test]
    fn utcdatetime_to_julian_date_zero() {
        let dt = UTCDateTime::from_ymdhms(-4712, 1, 1, 12, 0, 0);

        assert_almost_eq!(dt.to_julian_date(), 0.0);
    }

    #[test]
    fn utcdatetime_to_civil_julian_date_regular() {
        let dt = UTCDateTime::from_ymdhms(2024, 6, 11, 16, 43, 2);

        assert_almost_eq!(dt.to_civil_julian_date(), 2_460_473.696_550_925_7);
    }

    #[test]
    fn utcdatetime_to_civil_julian_date_zero() {
        let dt = UTCDateTime::from_ymdhms(-4712, 1, 1, 0, 0, 0);

        assert_almost_eq!(dt.to_civil_julian_date(), 0.0);
    }

    #[test]
    fn utcdatetime_parse_invalid_string() {
        let dt = "Sat. 11 May 2024".parse::<UTCDateTime>();

        assert!(dt.is_err());
    }

    #[test]
    fn utcdatetime_from_invalid_string() {
        let dt = UTCDateTime::try_from("Sat. 11 May 2024");

        assert!(dt.is_err());
    }

    #[test]
    fn utcdatetime_display() {
        let dt = UTCDateTime::from_ymdhms(1968, 2, 27, 9, 10, 0);

        assert_eq!(dt.to_string(), "1968-02-27T09:10:00Z");
    }

    #[test]
    fn utcdatetime_to_offsetdatetime_() {
        let odt = time::OffsetDateTime::try_from(&UTCDateTime::from_ymdhms(1938, 7, 15, 0, 0, 0))
            .unwrap();

        assert_eq!(
            odt,
            time::OffsetDateTime::new_utc(
                time::Date::from_calendar_date(1938, time::Month::July, 15).unwrap(),
                time::Time::from_hms(0, 0, 0).unwrap()
            )
        );
    }

    #[test]
    fn utcdatetime_monthname() {
        let dt = UTCDateTime::from_ymdhms(1968, 2, 27, 9, 10, 0);

        assert_eq!(dt.monthname().unwrap(), "February");
    }

    #[test]
    fn utcdatetime_weekday() {
        let dt = UTCDateTime::from_ymdhms(1968, 2, 27, 9, 10, 0);

        assert_eq!(dt.weekday().unwrap(), 2);
    }

    #[test]
    fn utcdatetime_dayname() {
        let dt = UTCDateTime::from_ymdhms(1968, 2, 27, 9, 10, 0);

        assert_eq!(dt.dayname().unwrap(), "Tuesday");
    }

    #[test]
    fn localdatetime_display() {
        let dt = LocalDateTime {
            year: 1968,
            month: 2,
            day: 27,
            hour: 9,
            minute: 10,
            second: 0,
        };

        assert_eq!(dt.to_string(), "1968-02-27T09:10:00");
    }
}
