//! A Moon for Rust.
//!
//! John Walker's moontool.c calculation routines, ported to Rust.

// Calculations _rely_ on truncations (casts from floats to integers.)
#![allow(clippy::cast_possible_truncation)]
// Compatibility with C API.
#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::many_single_char_names
)]

mod datetime;

pub use datetime::{LocalDateTime, UTCDateTime};
use std::{fmt, fmt::Write};

//  Astronomical constants

const EPOCH: f64 = 2_444_238.5; // 1980 January 0.0

//  Constants defining the Sun's apparent orbit

const ELONGE: f64 = 278.833_540; // Ecliptic longitude of the Sun at epoch 1980.0
const ELONGP: f64 = 282.596_403; // Ecliptic longitude of the Sun at perigee
const ECCENT: f64 = 0.016_718; // Eccentricity of Earth's orbit
const SUNSMAX: f64 = 1.495_985e8; // Semi-major axis of Earth's orbit, km
const SUNANGSIZ: f64 = 0.533_128; // Sun's angular size, degrees, at semi-major axis distance

//  Elements of the Moon's orbit, epoch 1980.0

const MMLONG: f64 = 64.975_464; // Moon's mean longitude at the epoch
const MMLONGP: f64 = 349.383_063; // Mean longitude of the perigee at the epoch
const MLNODE: f64 = 151.950_429; // Mean longitude of the node at the epoch
const MINC: f64 = 5.145_396; // Inclination of the Moon's orbit
const MECC: f64 = 0.054_900; // Eccentricity of the Moon's orbit
const MANGSIZ: f64 = 0.5181; // Moon's angular size at distance a from Earth
const MSMAX: f64 = 384_401.0; // Semi-major axis of Moon's orbit in km
const MPARALLAX: f64 = 0.9507; // Parallax at distance a from Earth
const SYNMONTH: f64 = 29.530_588_68; // Synodic month (new Moon to new Moon)
const LUNATBASE: f64 = 2_423_436.0; // Base date for E. W. Brown's numbered series of lunations (1923 January 16)

//  Properties of the Earth

const EARTHRAD: f64 = 6378.16; // Radius of Earth in kilometres

//  Handy mathematical functions

/// Fix angle.
fn fixangle(a: f64) -> f64 {
    a - 360.0 * (a / 360.0).floor()
}

/// Sin from deg.
fn dsin(x: f64) -> f64 {
    x.to_radians().sin()
}

/// Cos from deg.
fn dcos(x: f64) -> f64 {
    x.to_radians().cos()
}

macro_rules! EPL {
    ($x:expr) => {
        if $x == 1 {
            ""
        } else {
            "s"
        }
    };
}

// TODO: We should probably refactor those, we're far from the C API
// now anyway.
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

const PHANAME: [&str; 8] = [
    "New Moon",
    "Waxing Crescent",
    "First Quarter",
    "Waxing Gibbous",
    "Full Moon",
    "Waning Gibbous",
    "Last Quarter",
    "Waning Crescent",
];

const MOONICN: [&str; 8] = [
    "\u{1f311}", // 🌑
    "\u{1f312}", // 🌒
    "\u{1f313}", // 🌓
    "\u{1f314}", // 🌔
    "\u{1f315}", // 🌕
    "\u{1f316}", // 🌖
    "\u{1f317}", // 🌗
    "\u{1f318}", // 🌘
];

/// Compute values for a given date and time.
pub trait ForDateTime: Sized {
    #[cfg(not(tarpaulin_include))]
    #[must_use]
    fn now() -> Self {
        let now = UTCDateTime::now();
        Self::for_datetime(&now)
    }

    #[must_use]
    fn for_datetime(datetime: &UTCDateTime) -> Self;

    /// Shortcut for date, with time set to midnight.
    #[must_use]
    fn for_ymd(year: i32, month: u32, day: u32) -> Self {
        Self::for_ymdhms(year, month, day, 0, 0, 0)
    }

    #[must_use]
    fn for_ymdhms(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> Self {
        let datetime = UTCDateTime::from_ymdhms(year, month, day, hour, minute, second);
        Self::for_datetime(&datetime)
    }

    /// # Errors
    ///
    /// If parsing of datetime string fails.
    fn for_iso_string(iso_string: &str) -> Result<Self, &'static str> {
        let datetime = iso_string.parse()?;
        #[cfg(not(tarpaulin_include))] // Erroneously marked uncovered.
        Ok(Self::for_datetime(&datetime))
    }

    /// # Errors
    ///
    /// If timestamp is out of bounds.
    fn for_timestamp(timestamp: i64) -> Result<Self, &'static str> {
        let datetime = UTCDateTime::from_timestamp(timestamp)?;
        #[cfg(not(tarpaulin_include))] // Erroneously marked uncovered.
        Ok(Self::for_datetime(&datetime))
    }

    #[must_use]
    fn for_julian_date(julian_date: f64) -> Self {
        let datetime = UTCDateTime::from_julian_date(julian_date);
        Self::for_datetime(&datetime)
    }
}

/// Compute values for a given year.
///
/// For some data it makes sense to only provide a year. For example,
/// [`SunCalendar`] computes the equinoxes and solstices of a year;
/// full date and time values are not needed, nor considered in the
/// calculations.
pub trait ForYear: ForDateTime {
    /// Shortcut for year, on January 1st at midnight.
    #[must_use]
    fn for_year(year: i32) -> Self {
        Self::for_ymd(year, 1, 1)
    }
}

/// Serialize values to JSON.
pub trait ToJSON {
    fn to_json(&self) -> String;
}

/// Marker trait to enforce common behaviour across API objects.
///
/// This is used exclusively as a compiler-driven development aid.
/// Implementing it ensures nothing is left aside.
#[allow(dead_code)]
trait MarkerBase: ForDateTime + fmt::Display + ToJSON {}

/// Helper to `write!()` to a string with auto-`unwrap()`.
macro_rules! write_to {
    ($target:ident, $string:literal $(, $value:expr)*) => {
        write!($target, $string $(, $value)*).unwrap_or(());
    };
}

// Custom API.

/// Serves as return value for [`phase()`].
///
/// Besides returning the phase of the Moon, [`phase()`] also returns
/// interesting properties of the Moon and of the Sun.
#[derive(Debug, PartialEq)]
struct PhaseInfo {
    phase: f64,
    fraction_illuminated: f64,
    age: f64,
    ecliptic_longitude: f64,
    ecliptic_latitude: f64,
    parallax: f64,
    distance: f64,
    angular_diameter: f64,
    sun_ecliptic_longitude: f64,
    sun_distance: f64,
    sun_angular_diameter: f64,
}

/// Information about the phase of the Moon, at given time.
///
/// # Examples
///
/// ```rust
/// use moontool::moon::{ForDateTime, MoonPhase};
///
/// let mphase = MoonPhase::for_ymdhms(2024, 5, 4, 10, 0, 0);
///
/// assert_eq!(mphase.phase_name, "Waning Crescent");
/// ```
///
/// # Errors
///
/// Errors may be caused by input values that are out of range. Also,
/// when formatting to string, if the system's timezone offset cannot be
/// retrieved then local time won't appear in the output.
#[derive(Clone, Debug, PartialEq)]
pub struct MoonPhase {
    pub julian_date: f64,
    pub timestamp: Option<i64>,
    pub utc_datetime: UTCDateTime,
    pub age: f64,
    pub fraction_of_lunation: f64,
    pub phase: usize,
    pub phase_name: String,
    pub phase_icon: String,
    pub fraction_illuminated: f64,
    /// Angular distance around the geocentric ecliptic (λ).
    ///
    /// The _ecliptic_ (or _ecliptic plane_) is the orbital plane of
    /// Earth around the Sun. Its direction (0°) is towards the March
    /// (vernal) equinox.
    ///
    /// > By definition, the times of New Moon, First Quarter, Full
    /// > Moon, and Last Quarter are the times at which the excess of
    /// > the apparent geocentric longitude of the Moon over the
    /// > apparent geocentric longitude of the Sun is 0°, 90°, 180°, and
    /// > 270° respectively.
    /// >
    /// > — Jean Meeus, Astronomical Algorithms, Chapter 49
    pub ecliptic_longitude: f64,
    /// Angular distance from the geocentric ecliptic towards the North
    /// (positive) or South (negative) ecliptic pole (β).
    ///
    /// Typically, between 5.145° and -5.145°.
    ///
    /// The _ecliptic_ (or _ecliptic plane_) is the orbital plane of
    /// Earth around the Sun. Its direction (0°) is towards the March
    /// (vernal) equinox.
    pub ecliptic_latitude: f64,
    pub parallax: f64,
    pub distance_to_earth_km: f64,
    pub distance_to_earth_earth_radii: f64,
    /// Angular diameter.
    pub subtends: f64,
    /// Sun's angular distance around the geocentric ecliptic (λ).
    ///
    /// The _ecliptic_ (or _ecliptic plane_) is the orbital plane of
    /// Earth around the Sun. Its direction (0°) is towards the March
    /// (vernal) equinox.
    ///
    /// > By definition, the times of the equinoxes and solstices are
    /// > the instants when the apparent geocentric longitude of the Sun
    /// > is an integer multiple of 90 degrees.
    /// >
    /// > - 0° for the March equinox,
    /// > - 90° for the June solstice,
    /// > - 180° for the September equinox,
    /// > - 270° for the December solstice.
    /// >
    /// > — Jean Meeus, Astronomical Algorithms, Chapter 27
    pub sun_ecliptic_longitude: f64,
    pub sun_distance_to_earth_km: f64,
    pub sun_distance_to_earth_astronomical_units: f64,
    /// Sun's angular diameter.
    pub sun_subtends: f64,
}

impl MarkerBase for MoonPhase {}

impl ForDateTime for MoonPhase {
    #[must_use]
    fn for_datetime(datetime: &UTCDateTime) -> Self {
        moonphase(datetime)
    }
}

impl fmt::Display for MoonPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let aom = &self.age;
        let aom_d = (aom.trunc() as i32).unsigned_abs();
        let aom_h = ((24.0 * (aom - aom.floor())).trunc() as i32).unsigned_abs();
        let aom_m = ((1440.0 * (aom - aom.floor()) % 60.0).trunc() as i32).unsigned_abs();

        let gm = &self.utc_datetime;

        writeln!(f, "Phase\n=====\n")?;
        writeln!(
            f,
            "Julian date:\t\t{:.5}   (0h variant: {:.5})",
            self.julian_date,
            self.julian_date + 0.5
        )?;
        writeln!(
            f,
            "Universal time:\t\t{:<9} {:>2}:{:0>2}:{:0>2} {:>2} {:<5} {}",
            DAYNAME[gm.weekday() as usize],
            gm.hour,
            gm.minute,
            gm.second,
            gm.day,
            MONAME[(gm.month - 1) as usize],
            gm.year,
        )?;

        if let Ok(tm) = LocalDateTime::try_from(gm) {
            writeln!(
                f,
                "Local time:\t\t{:<9} {:>2}:{:0>2}:{:0>2} {:>2} {:<5} {}\n",
                DAYNAME[tm.weekday() as usize],
                tm.hour,
                tm.minute,
                tm.second,
                tm.day,
                MONAME[(tm.month - 1) as usize],
                tm.year,
            )?;
        } else {
            writeln!(f)?;
        }

        writeln!(
            f,
            "Age of moon:\t\t{} day{}, {} hour{}, {} minute{}.",
            aom_d,
            EPL!(aom_d),
            aom_h,
            EPL!(aom_h),
            aom_m,
            EPL!(aom_m),
        )?;
        writeln!(
            f,
            "Lunation:\t\t{:.2}%   ({} {})",
            self.fraction_of_lunation * 100.0,
            self.phase_icon,
            self.phase_name
        )?;
        writeln!(
            f,
            "Moon phase:\t\t{:.2}%   (0% = New, 100% = Full)\n",
            self.fraction_illuminated * 100.0
        )?;

        writeln!(
            f,
            "Moon's distance:\t{:.0} kilometres, {:.1} Earth radii.",
            self.distance_to_earth_km, self.distance_to_earth_earth_radii
        )?;
        writeln!(f, "Moon subtends:\t\t{:.4} degrees.\n", self.subtends)?;

        writeln!(
            f,
            "Sun's distance:\t\t{:.0} kilometres, {:.3} astronomical units.",
            self.sun_distance_to_earth_km, self.sun_distance_to_earth_astronomical_units,
        )?;
        write!(f, "Sun subtends:\t\t{:.4} degrees.", self.sun_subtends)
    }
}

impl ToJSON for MoonPhase {
    fn to_json(&self) -> String {
        let mut json = String::new();
        write_to!(json, "{{");
        write_to!(json, r#""julian_date":{},"#, self.julian_date);
        write_to!(
            json,
            r#""timestamp":{},"#,
            self.timestamp
                .map_or_else(|| String::from("null"), |v| v.to_string())
        );
        write_to!(json, r#""utc_datetime":"{}","#, self.utc_datetime);
        write_to!(json, r#""age":{},"#, self.age);
        write_to!(
            json,
            r#""fraction_of_lunation":{},"#,
            self.fraction_of_lunation
        );
        write_to!(json, r#""phase":{},"#, self.phase);
        write_to!(json, r#""phase_name":"{}","#, self.phase_name);
        write_to!(json, r#""phase_icon":"{}","#, self.phase_icon);
        write_to!(
            json,
            r#""fraction_illuminated":{},"#,
            self.fraction_illuminated
        );
        write_to!(json, r#""ecliptic_longitude":{},"#, self.ecliptic_longitude);
        write_to!(json, r#""ecliptic_latitude":{},"#, self.ecliptic_latitude);
        write_to!(json, r#""parallax":{},"#, self.parallax);
        write_to!(
            json,
            r#""distance_to_earth_km":{},"#,
            self.distance_to_earth_km
        );
        write_to!(
            json,
            r#""distance_to_earth_earth_radii":{},"#,
            self.distance_to_earth_earth_radii
        );
        write_to!(json, r#""subtends":{},"#, self.subtends);
        write_to!(
            json,
            r#""sun_ecliptic_longitude":{},"#,
            self.sun_ecliptic_longitude
        );
        write_to!(
            json,
            r#""sun_distance_to_earth_km":{},"#,
            self.sun_distance_to_earth_km
        );
        write_to!(
            json,
            r#""sun_distance_to_earth_astronomical_units":{},"#,
            self.sun_distance_to_earth_astronomical_units
        );
        write_to!(json, r#""sun_subtends":{}"#, self.sun_subtends);
        write_to!(json, "}}");
        json
    }
}

/// Information about past and future Moons, around given time.
///
/// Note: [`last_new_moon`](MoonCalendar::last_new_moon),
/// [`first_quarter`](MoonCalendar::first_quarter),
/// [`full_moon`](MoonCalendar::full_moon),
/// [`last_quarter`](MoonCalendar::last_quarter), and
/// [`next_new_moon`](MoonCalendar::next_new_moon), are Julian Day
/// Numbers (JDN)[^jdn].
///
/// [^jdn]: <https://en.wikipedia.org/wiki/Julian_day>
///
/// # Examples
///
/// ```rust
/// use moontool::moon::{ForDateTime, MoonCalendar};
///
/// let mcal = MoonCalendar::for_ymdhms(2024, 5, 4, 10, 0, 0);
///
/// assert_eq!(mcal.lunation, 1253);
/// ```
///
/// # Errors
///
/// Errors may be caused by input values that are out of range. Also,
/// when formatting to string, if the system's timezone offset cannot be
/// retrieved then local time won't appear in the output.
#[derive(Clone, Debug, PartialEq)]
pub struct MoonCalendar {
    pub julian_date: f64,
    pub timestamp: Option<i64>,
    pub utc_datetime: UTCDateTime,
    /// Brown Lunation Number (BLN). Numbering begins at the first
    /// New Moon of 1923 (17 January 1923 at 2:41 UTC).
    pub lunation: i64,
    pub last_new_moon: f64,
    pub last_new_moon_utc: UTCDateTime,
    pub first_quarter: f64,
    pub first_quarter_utc: UTCDateTime,
    pub full_moon: f64,
    pub full_moon_utc: UTCDateTime,
    pub last_quarter: f64,
    pub last_quarter_utc: UTCDateTime,
    pub next_new_moon: f64,
    pub next_new_moon_utc: UTCDateTime,
}

impl MarkerBase for MoonCalendar {}

#[allow(clippy::missing_errors_doc)]
impl ForDateTime for MoonCalendar {
    #[must_use]
    fn for_datetime(datetime: &UTCDateTime) -> Self {
        mooncal(datetime)
    }
}

impl fmt::Display for MoonCalendar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Moon Calendar\n=============\n")?;
        writeln!(
            f,
            "Last new moon:\t\t{}\tLunation: {}",
            fmt_phase_time(&self.last_new_moon_utc),
            self.lunation
        )?;
        writeln!(
            f,
            "First quarter:\t\t{}",
            fmt_phase_time(&self.first_quarter_utc)
        )?;
        writeln!(f, "Full moon:\t\t{}", fmt_phase_time(&self.full_moon_utc))?;
        writeln!(
            f,
            "Last quarter:\t\t{}",
            fmt_phase_time(&self.last_quarter_utc)
        )?;
        write!(
            f,
            "Next new moon:\t\t{}\tLunation: {}",
            fmt_phase_time(&self.next_new_moon_utc),
            self.lunation + 1
        )
    }
}

impl ToJSON for MoonCalendar {
    fn to_json(&self) -> String {
        let mut json = String::new();
        write_to!(json, "{{");
        write_to!(json, r#""julian_date":{},"#, self.julian_date);
        write_to!(
            json,
            r#""timestamp":{},"#,
            self.timestamp
                .map_or_else(|| String::from("null"), |v| v.to_string())
        );
        write_to!(json, r#""utc_datetime":"{}","#, self.utc_datetime);
        write_to!(json, r#""lunation":{},"#, self.lunation);
        write_to!(json, r#""last_new_moon":{},"#, self.last_new_moon);
        write_to!(json, r#""last_new_moon_utc":"{}","#, self.last_new_moon_utc);
        write_to!(json, r#""first_quarter":{},"#, self.first_quarter);
        write_to!(json, r#""first_quarter_utc":"{}","#, self.first_quarter_utc);
        write_to!(json, r#""full_moon":{},"#, self.full_moon);
        write_to!(json, r#""full_moon_utc":"{}","#, self.full_moon_utc);
        write_to!(json, r#""last_quarter":{},"#, self.last_quarter);
        write_to!(json, r#""last_quarter_utc":"{}","#, self.last_quarter_utc);
        write_to!(json, r#""next_new_moon":{},"#, self.next_new_moon);
        write_to!(json, r#""next_new_moon_utc":"{}""#, self.next_new_moon_utc);
        write_to!(json, "}}");
        json
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewMoon {
    pub date: f64,
    pub date_utc: UTCDateTime,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FullMoon {
    pub date: f64,
    pub date_utc: UTCDateTime,
    pub name: String,
}

/// List of all New Moons and Full Moons, of a given year.
///
/// # Examples
///
/// ```rust
/// use moontool::moon::{ForYear, FullMoon, NewMoon, UTCDateTime, YearlyMoonCalendar};
///
/// let ymcal = YearlyMoonCalendar::for_year(2024);
///
/// println!("{ymcal:#?}");
///
/// assert_eq!(
///     ymcal.new_moons[0],
///     NewMoon {
///         date: 2460320.9986786423,
///         date_utc: UTCDateTime::from_ymdhms(2024, 1, 11, 11, 58, 6),
///     }
/// );
/// assert_eq!(
///     ymcal.full_moons[8],
///     FullMoon {
///         date: 2460571.6088363146,
///         date_utc: UTCDateTime::from_ymdhms(2024, 9, 18, 2, 36, 43),
///         name: String::from("Harvest Moon"),
///     },
/// );
/// ```
///
/// # Errors
///
/// Errors may be caused by input values that are out of range. Also,
/// when formatting to string, if the system's timezone offset cannot be
/// retrieved then local time won't appear in the output.
#[derive(Clone, Debug, PartialEq)]
pub struct YearlyMoonCalendar {
    pub julian_date: f64,
    pub timestamp: Option<i64>,
    pub utc_datetime: UTCDateTime,
    pub new_moons: Vec<NewMoon>,
    pub full_moons: Vec<FullMoon>,
}

impl MarkerBase for YearlyMoonCalendar {}

impl ForDateTime for YearlyMoonCalendar {
    fn for_datetime(datetime: &UTCDateTime) -> Self {
        yearly_mooncal(datetime)
    }
}

impl ForYear for YearlyMoonCalendar {}

impl fmt::Display for YearlyMoonCalendar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let new_moons: Vec<String> = self
            .new_moons
            .iter()
            .enumerate()
            .map(|(i, x)| {
                format!("{:>2}. {:<37}", i + 1, fmt_phase_time(&x.date_utc))
                    .trim_end()
                    .to_string()
            })
            .collect();

        let full_moons: Vec<String> = self
            .full_moons
            .iter()
            .enumerate()
            .map(|(i, x)| {
                format!(
                    "{:>2}. {:<37}   {}",
                    i + 1,
                    fmt_phase_time(&x.date_utc),
                    x.name
                )
                .trim_end()
                .to_string()
            })
            .collect();

        writeln!(f, "New Moons\n=========\n")?;
        writeln!(f, "{}", new_moons.join("\n"))?;
        writeln!(f, "\nFull Moons\n==========\n")?;
        write!(f, "{}", full_moons.join("\n"))
    }
}

impl ToJSON for YearlyMoonCalendar {
    fn to_json(&self) -> String {
        let mut json = String::new();
        write_to!(json, "{{");
        write_to!(json, r#""julian_date":{},"#, self.julian_date);
        write_to!(
            json,
            r#""timestamp":{},"#,
            self.timestamp
                .map_or_else(|| String::from("null"), |v| v.to_string())
        );
        write_to!(
            json,
            r#""new_moons":[{}],"#,
            self.new_moons
                .iter()
                .map(|new_moon| format!(
                    r#"{{"date":{},"date_utc":"{}"}}"#,
                    new_moon.date, new_moon.date_utc
                ))
                .collect::<Vec<String>>()
                .join(",")
        );
        write_to!(
            json,
            r#""full_moons":[{}]"#,
            self.full_moons
                .iter()
                .map(|full_moon| format!(
                    r#"{{"date":{},"date_utc":"{}","name":"{}"}}"#,
                    full_moon.date, full_moon.date_utc, full_moon.name
                ))
                .collect::<Vec<String>>()
                .join(",")
        );
        write_to!(json, "}}");
        json
    }
}

fn yearly_mooncal(gm: &UTCDateTime) -> YearlyMoonCalendar {
    let (new_moons, mut full_moons) = new_moons_for_year(gm.year);

    name_full_moons(&mut full_moons);

    YearlyMoonCalendar {
        julian_date: gm.to_julian_date(),
        timestamp: gm.to_timestamp().ok(),
        utc_datetime: gm.clone(),
        new_moons,
        full_moons,
    }
}

#[allow(clippy::comparison_chain)]
fn new_moons_for_year(year: i32) -> (Vec<NewMoon>, Vec<FullMoon>) {
    let mut new_moons = vec![];
    let mut full_moons = vec![];

    // Start on the last day of the year prior. This may catch Moons
    // that belong to the wrong year (they will be ignored), but on the
    // upside, we ensure that "next New Moon" is the first New Moon of
    // the target year, and dispel any concerns about rounding errors.
    let mut jd = UTCDateTime::from_ymdhms(year - 1, 12, 31, 0, 0, 0).to_julian_date();

    loop {
        // Get info for the current lunar month.
        let mcal = MoonCalendar::for_julian_date(jd);

        // If "Full Moon" is in the target year, take it.
        if mcal.full_moon_utc.year == year {
            full_moons.push(FullMoon {
                date: mcal.full_moon,
                date_utc: mcal.full_moon_utc,
                name: String::new(),
            });

        // But if "Full Moon" is next year, we're done. "next New Moon"
        // is necessarily next year as well.
        } else if mcal.full_moon_utc.year > year {
            break;
        }

        // If "next New Moon" is in the target year, take it (it may
        // belong to the year prior if it falls on Dec 31st).
        if mcal.next_new_moon_utc.year == year {
            new_moons.push(NewMoon {
                date: mcal.next_new_moon,
                date_utc: mcal.next_new_moon_utc,
            });
        }

        // Go to the day after "next New Moon". This ensures we're in
        // the subsequent lunar month for the next iteration.
        jd = mcal.next_new_moon + 1.0;
    }

    (new_moons, full_moons)
}

fn name_full_moons(full_moons: &mut [FullMoon]) {
    let mut last_month = 0;
    for full_moon in full_moons.iter_mut() {
        let name = match full_moon.date_utc.month {
            // The second Full Moon in a given month is a Blue Moon.
            _ if full_moon.date_utc.month == last_month => "Blue Moon",
            1 => "Wolf Moon",
            2 => "Snow Moon",
            3 => "Worm Moon",
            4 => "Pink Moon",
            5 => "Flower Moon",
            6 => "Strawberry Moon",
            7 => "Buck Moon",
            8 => "Sturgeon Moon",
            9 => "Corn Moon",
            10 => "Hunter's Moon",
            11 => "Beaver Moon",
            12 => "Cold Moon",
            #[cfg(not(tarpaulin_include))]
            _ => continue,
        };

        full_moon.name = String::from(name);
        last_month = full_moon.date_utc.month;
    }

    let i = find_index_of_harvest_moon(full_moons);

    if let Some(harvest_moon) = full_moons.get_mut(i) {
        harvest_moon.name = String::from("Harvest Moon");
    }
    // Traditionally, the Hunter's Moon follows the Harvest Moon.
    // If the Harvest Moon takes place in October, the Hunter's Moon
    // will be the first Full Moon of November (e.g., 2001).
    if let Some(hunters_moon) = full_moons.get_mut(i + 1) {
        hunters_moon.name = String::from("Hunter's Moon");
    }
}

/// Find the index of the Harvest Moon among the list of Full Moons.
///
/// The Harvest Moon is the full moon closest to September's equinox,
/// usually occurring in September but sometimes in October, about once
/// every three years.
///
/// Traditionally, this Moon provided extra light to farmers, aiding
/// them in harvesting crops. Many cultures and spiritual traditions
/// hold ceremonies or rituals during this time.
///
/// # Warning
///
/// This function assumes the list of Full Moons in exhaustive for the
/// given year, so that the Harvest Moon necessarily _is_ among the
/// given Moons.
fn find_index_of_harvest_moon(full_moons: &[FullMoon]) -> usize {
    let year = full_moons[0].date_utc.year;
    let september_equinox = solarevent(year, SolarEvent::SeptemberEquinox);

    let mut harvest_moon: Option<(f64, usize)> = None;

    for (i, full_moon) in full_moons.iter().enumerate() {
        let d = (september_equinox - full_moon.date).abs();
        if let Some((min_d, _)) = harvest_moon {
            if d < min_d {
                // Bias towards September if equal.
                harvest_moon = Some((d, i));
            }
        } else {
            harvest_moon = Some((d, i));
        }
    }

    let (_, i) = harvest_moon.expect("as long as there are Moons, there is a nearest.");
    i
}

/// Information about equinoxes and solstices, of a given year.
///
/// > By definition, the times of the equinoxes and solstices are the
/// > instants when the apparent geocentric longitude of the Sun (that
/// > is, calculated by including the effects of aberration and
/// > nutation) is an integer multiple of 90 degrees. (Because the
/// > latitude of the Sun is not exactly zero, the declination of the
/// > Sun is not exactly zero at the instant of an equinox.)
/// >
/// > — Jean Meeus, Astronomical Algorithms, Chapter 27, page 177
#[derive(Clone, Debug, PartialEq)]
pub struct SunCalendar {
    pub julian_date: f64,
    pub timestamp: Option<i64>,
    pub utc_datetime: UTCDateTime,
    /// March equinox.
    ///
    /// Beginning of astronomical spring.
    ///
    /// Around March 20, also called Vernal or Spring equinox in the
    /// Northern hemisphere.
    ///
    /// The day of the year when the Sun crosses the equator moving from
    /// the Southern hemisphere to the Northern hemisphere.
    ///
    /// Approximately equal length of day and night.
    pub march_equinox: f64,
    pub march_equinox_utc: UTCDateTime,
    /// June solstice.
    ///
    /// Beginning of astronomical summer.
    ///
    /// Around June 20–22, also called Estival or Summer solstice in the
    /// Northern hemisphere.
    ///
    /// The longest day of the year when the Sun is at its highest point
    /// in the sky at noon, marking the beginning of summer in the
    /// Northern hemisphere.
    ///
    /// Longest day and shortest night of the year.
    pub june_solstice: f64,
    pub june_solstice_utc: UTCDateTime,
    /// September equinox.
    ///
    /// Beginning of astronomical autumn.
    ///
    /// Around September 23, also called Autumnal or Autumn equinox in
    /// the Northern hemisphere.
    ///
    /// The day of the year when the Sun crosses the equator moving from
    /// the Northern hemisphere to the Southern hemisphere.
    ///
    /// Approximately equal length of day and night.
    pub september_equinox: f64,
    pub september_equinox_utc: UTCDateTime,
    /// December solstice.
    ///
    /// Beginning of astronomical winter.
    ///
    /// Around December 20-22, also called Hibernal or Winter solstice
    /// in the Northern hemisphere.
    ///
    /// The shortest day of the year when the Sun is at its lowest point
    /// in the sky at noon, marking the beginning of winter in the
    /// Northern hemisphere.
    ///
    /// Shortest day and longest night of the year.
    pub december_solstice: f64,
    pub december_solstice_utc: UTCDateTime,
}

impl MarkerBase for SunCalendar {}

impl ForDateTime for SunCalendar {
    fn for_datetime(datetime: &UTCDateTime) -> Self {
        suncal(datetime)
    }
}

impl ForYear for SunCalendar {}

impl fmt::Display for SunCalendar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Sun Calendar\n============\n")?;
        writeln!(
            f,
            "March equinox:\t\t{}",
            fmt_phase_time(&self.march_equinox_utc)
        )?;
        writeln!(
            f,
            "June solstice:\t\t{}",
            fmt_phase_time(&self.june_solstice_utc)
        )?;
        writeln!(
            f,
            "September equinox:\t{}",
            fmt_phase_time(&self.september_equinox_utc)
        )?;
        write!(
            f,
            "December solstice:\t{}",
            fmt_phase_time(&self.december_solstice_utc)
        )
    }
}

impl ToJSON for SunCalendar {
    fn to_json(&self) -> String {
        let mut json = String::new();
        write_to!(json, "{{");
        write_to!(json, r#""julian_date":{},"#, self.julian_date);
        write_to!(
            json,
            r#""timestamp":{},"#,
            self.timestamp
                .map_or_else(|| String::from("null"), |v| v.to_string())
        );
        write_to!(json, r#""utc_datetime":"{}","#, self.utc_datetime);
        write_to!(json, r#""march_equinox":{},"#, self.march_equinox);
        write_to!(json, r#""march_equinox_utc":"{}","#, self.march_equinox_utc);
        write_to!(json, r#""june_solstice":{},"#, self.june_solstice);
        write_to!(json, r#""june_solstice_utc":"{}","#, self.june_solstice_utc);
        write_to!(json, r#""september_equinox":{},"#, self.september_equinox);
        write_to!(
            json,
            r#""september_equinox_utc":"{}","#,
            self.september_equinox_utc
        );
        write_to!(json, r#""december_solstice":{},"#, self.december_solstice);
        write_to!(
            json,
            r#""december_solstice_utc":"{}""#,
            self.december_solstice_utc
        );
        write_to!(json, "}}");
        json
    }
}

fn suncal(gm: &UTCDateTime) -> SunCalendar {
    let march_equinox = solarevent(gm.year, SolarEvent::MarchEquinox);
    let june_solstice = solarevent(gm.year, SolarEvent::JuneSolstice);
    let september_equinox = solarevent(gm.year, SolarEvent::SeptemberEquinox);
    let december_solstice = solarevent(gm.year, SolarEvent::DecemberSolstice);

    let jd = gm.to_julian_date();

    SunCalendar {
        julian_date: jd,
        timestamp: gm.to_timestamp().ok(),
        utc_datetime: gm.clone(),
        march_equinox,
        march_equinox_utc: UTCDateTime::from_julian_date(march_equinox),
        june_solstice,
        june_solstice_utc: UTCDateTime::from_julian_date(june_solstice),
        september_equinox,
        september_equinox_utc: UTCDateTime::from_julian_date(september_equinox),
        december_solstice,
        december_solstice_utc: UTCDateTime::from_julian_date(december_solstice),
    }
}

#[derive(Copy, Clone)]
enum SolarEvent {
    MarchEquinox,
    JuneSolstice,
    SeptemberEquinox,
    DecemberSolstice,
}

/// Calculate equinoxes and solstices of a year as Julian dates.
///
/// Algorithm as given in Meeus, Astronomical Algorithms, Chapter 27,
/// page 177.
#[rustfmt::skip]
#[allow(non_snake_case)]
fn solarevent(year: i32, event: SolarEvent) -> f64 {
    // For years -1000 to +1000, but can be used for several centuries
    // before -1000 with only small errors.
    let jde0 = if year <= 1000 {
        let Y = f64::from(year) / 1000.0;
        let Y2 = Y * Y;
        let Y3 = Y2 * Y;
        let Y4 = Y3 * Y;

        match event {
            SolarEvent::MarchEquinox =>
                1_721_139.291_89 + 365_242.137_40 * Y + 0.061_34 * Y2 + 0.001_11 * Y3 - 0.000_71 * Y4,
            SolarEvent::JuneSolstice =>
                1_721_233.254_01 + 365_241.725_62 * Y - 0.053_23 * Y2 + 0.009_07 * Y3 + 0.000_25 * Y4,
            SolarEvent::SeptemberEquinox =>
                1_721_325.704_55 + 365_242.495_58 * Y - 0.116_77 * Y2 - 0.002_97 * Y3 + 0.000_74 * Y4,
            SolarEvent::DecemberSolstice =>
                1_721_414.399_87 + 365_242.882_57 * Y - 0.007_69 * Y2 - 0.009_33 * Y3 - 0.000_06 * Y4,
        }
    // For years +1000 to +3000, but can be used for several centuries
    // after +3000 with only small errors.
    } else {
        let Y = (f64::from(year) - 2000.0) / 1000.0;
        let Y2 = Y * Y;
        let Y3 = Y2 * Y;
        let Y4 = Y3 * Y;

        match event {
            SolarEvent::MarchEquinox =>
                2_451_623.809_84 + 365_242.374_04 * Y + 0.051_69 * Y2 - 0.004_11 * Y3 - 0.000_57 * Y4,
            SolarEvent::JuneSolstice =>
                2_451_716.567_67 + 365_241.626_03 * Y + 0.003_25 * Y2 + 0.008_88 * Y3 - 0.000_30 * Y4,
            SolarEvent::SeptemberEquinox =>
                2_451_810.217_15 + 365_242.017_67 * Y - 0.115_75 * Y2 + 0.003_37 * Y3 + 0.000_78 * Y4,
            SolarEvent::DecemberSolstice =>
                2_451_900.059_52 + 365_242.740_49 * Y - 0.062_23 * Y2 - 0.008_23 * Y3 + 0.000_32 * Y4,
        }
    };

    // Correction of approximate JDE0.

    let T = (jde0 - 2_451_545.0) / 36525.0;
    let W = 35_999.373 * T - 2.47;
    let dL = 1.0 + 0.033_4 * dcos(W) + 0.000_7 * dcos(2.0 * W);

    let S = 485.0 * dcos(324.96 + 1_934.136 * T)
        + 203.0 * dcos(337.23 + 32_964.467 * T)
        + 199.0 * dcos(342.08 + 20.186 * T)
        + 182.0 * dcos(27.85 + 445_267.112 * T)
        + 156.0 * dcos(73.14 + 45_036.886 * T)
        + 136.0 * dcos(171.52 + 22_518.443 * T)
        + 77.0 * dcos(222.54 + 65_928.934 * T)
        + 74.0 * dcos(296.72 + 3_034.906 * T)
        + 70.0 * dcos(243.58 + 9_037.513 * T)
        + 58.0 * dcos(119.81 + 33_718.147 * T)
        + 52.0 * dcos(297.17 + 150.678 * T)
        + 50.0 * dcos(21.02 + 2_281.226 * T)
        + 45.0 * dcos(247.54 + 29_929.562 * T)
        + 44.0 * dcos(325.15 + 31_555.956 * T)
        + 29.0 * dcos(60.93 + 4_443.417 * T)
        + 18.0 * dcos(155.12 + 67_555.328 * T)
        + 17.0 * dcos(288.79 + 4_562.452 * T)
        + 16.0 * dcos(198.04 + 62_894.029 * T)
        + 14.0 * dcos(199.76 + 31_436.921 * T)
        + 12.0 * dcos(95.39 + 14_577.848 * T)
        + 12.0 * dcos(287.11 + 31_931.756 * T)
        + 12.0 * dcos(320.81 + 34_777.259 * T)
        + 9.0 * dcos(227.73 + 1_222.114 * T)
        + 8.0 * dcos(15.45 + 16_859.074 * T);

    jde0 + (0.000_01 * S) / dL
}

fn fraction_of_lunation_to_phase(p: f64) -> usize {
    // Apart from Waxing and Waning, the other phases are very precise
    // points in time. For example, Full Moon occurs precisely at
    // `phase = 0.5`. This is too restrictive; for an observer, the Moon
    // appears Full over a larger timespan, rather than a single moment.
    // `day_frac` acts as padding around these lunar events, elongating
    // their duration artificially.
    let day_frac: f64 = (1.0 / SYNMONTH) * 0.75;

    if p < 0.00 + day_frac {
        0 // New Moon
    } else if p < 0.25 - day_frac {
        1 // Waxing Crescent
    } else if p < 0.25 + day_frac {
        2 // First Quarter
    } else if p < 0.50 - day_frac {
        3 // Waxing Gibbous
    } else if p < 0.50 + day_frac {
        4 // Full Moon
    } else if p < 0.75 - day_frac {
        5 // Waning Gibbous
    } else if p < 0.75 + day_frac {
        6 // Last Quarter
    } else if p < 1.00 - day_frac {
        7 // Waning Crescent
    } else {
        0 // New Moon
    }
}

/// Populate `MoonPhase` with info about the Moon at given time.
fn moonphase(gm: &UTCDateTime) -> MoonPhase {
    let jd = gm.to_julian_date();

    let phase_info = phase(jd);

    let phase_fraction = fraction_of_lunation_to_phase(phase_info.phase);

    MoonPhase {
        julian_date: jd,
        timestamp: gm.to_timestamp().ok(),
        utc_datetime: gm.clone(),
        age: phase_info.age,
        fraction_of_lunation: phase_info.phase,
        phase: phase_fraction,
        phase_name: String::from(PHANAME[phase_fraction]),
        phase_icon: String::from(MOONICN[phase_fraction]),
        fraction_illuminated: phase_info.fraction_illuminated,
        ecliptic_longitude: phase_info.ecliptic_longitude,
        ecliptic_latitude: phase_info.ecliptic_latitude,
        parallax: phase_info.parallax,
        distance_to_earth_km: phase_info.distance,
        distance_to_earth_earth_radii: phase_info.distance / EARTHRAD,
        subtends: phase_info.angular_diameter,
        sun_ecliptic_longitude: phase_info.sun_ecliptic_longitude,
        sun_distance_to_earth_km: phase_info.sun_distance,
        sun_distance_to_earth_astronomical_units: phase_info.sun_distance / SUNSMAX,
        sun_subtends: phase_info.sun_angular_diameter,
    }
}

/// Populate `MoonCalendar` with info about lunation at given time.
fn mooncal(gm: &UTCDateTime) -> MoonCalendar {
    let jd = jtime(gm);

    let phasar = phasehunt(jd + 0.5);
    let lunation = ((((phasar.0 + 7.0) - LUNATBASE) / SYNMONTH).floor().trunc() as i64) + 1;

    MoonCalendar {
        julian_date: jd,
        timestamp: gm.to_timestamp().ok(),
        utc_datetime: gm.clone(),
        lunation,
        last_new_moon: phasar.0,
        last_new_moon_utc: jtouct(phasar.0),
        first_quarter: phasar.1,
        first_quarter_utc: jtouct(phasar.1),
        full_moon: phasar.2,
        full_moon_utc: jtouct(phasar.2),
        last_quarter: phasar.3,
        last_quarter_utc: jtouct(phasar.3),
        next_new_moon: phasar.4,
        next_new_moon_utc: jtouct(phasar.4),
    }
}

/// Format the provided date and time in UTC format for screen display.
fn fmt_phase_time(gm: &UTCDateTime) -> String {
    format!(
        "{:<9} {:>2}:{:0>2} UTC {:>2} {:<5} {}",
        DAYNAME[gm.weekday() as usize], // TODO: Can weekday be 99 here? => Return Result and do something useful instead (just leave blank). Same elsewhere.
        gm.hour,
        gm.minute,
        gm.day,
        MONAME[(gm.month - 1) as usize],
        gm.year,
    )
}

/// Convert UTC date/time to astronomical Julian time.
///
/// (i.e. Julian date plus day fraction, expressed as a floating point).
fn jtime(t: &UTCDateTime) -> f64 {
    ucttoj(t.year, t.month - 1, t.day, t.hour, t.minute, t.second)
}

/// Convert GMT date and time to astronomical Julian time.
///
/// (i.e. Julian date plus day fraction, expressed as a floating point).
fn ucttoj(year: i32, month: u32, mday: u32, hour: u32, minute: u32, second: u32) -> f64 {
    // Algorithm as given in Meeus, Astronomical Algorithms, Chapter 7, page 61

    // RUSTFLAGS='--cfg PARANOID' cargo build
    #[cfg(PARANOID)]
    {
        assert!(month < 12);
        assert!(mday < 32);
        assert!(hour < 24);
        assert!(minute < 60);
        assert!(second < 60);
    }

    let mut m = month + 1;
    let mut y = year;

    if m <= 2 {
        y -= 1;
        m += 12;
    }

    // Determine whether date is in Julian or Gregorian calendar based on
    // canonical date of calendar reform.

    let b = if (year < 1582) || ((year == 1582) && ((month < 9) || (month == 9 && mday < 5))) {
        0
    } else {
        let a = y / 100;
        2 - a + (a / 4)
    };

    ((365.25 * f64::from(y + 4716)).trunc()
        + (30.6001 * f64::from(m + 1)).trunc()
        + f64::from(mday)
        + f64::from(b)
        - 1524.5)
        + (f64::from(second + 60 * (minute + 60 * hour)) / 86400.0)
}

/// Convert astronomical Julian time to GMT date and time.
fn jtouct(utime: f64) -> UTCDateTime {
    let (yy, mm, dd) = jyear(utime);
    let (hh, mmm, ss) = jhms(utime);

    UTCDateTime {
        year: yy,
        month: mm.unsigned_abs(),
        day: dd.unsigned_abs(),
        hour: hh.unsigned_abs(),
        minute: mmm.unsigned_abs(),
        second: ss.unsigned_abs(),
    }
}

/// Convert Julian date to year, month, day.
///
/// Year, month, day are returned via floating points.
fn jyear(mut td: f64) -> (i32, i32, i32) {
    td += 0.5;
    let z = td.floor();
    let f = td - z;

    let a = if z < 2_299_161.0 {
        z
    } else {
        let alpha = ((z - 1_867_216.25) / 36524.25).floor();
        z + 1.0 + alpha - (alpha / 4.0).floor()
    };

    let b = a + 1524.0;
    let c = ((b - 122.1) / 365.25).floor();
    let d = (365.25 * c).floor();
    let e = ((b - d) / 30.6001).floor();

    let dd = (b - d - (30.6001 * e).floor() + f).trunc() as i32;
    let mm = (if e < 14.0 { e - 1.0 } else { e - 13.0 }).trunc() as i32;
    let yy = (if mm > 2 { c - 4716.0 } else { c - 4715.0 }).trunc() as i32;

    (yy, mm, dd)
}

/// Convert Julian time to hour, minutes, and seconds.
fn jhms(mut j: f64) -> (i32, i32, i32) {
    j += 0.5; // Astronomical to civil
    let ij = (((j - j.floor()) * 86400.0) + 0.5).trunc() as i64; // Round to nearest second
    let h = (ij / 3600) as i32;
    let m = ((ij / 60) % 60) as i32;
    let s = (ij % 60) as i32;
    (h, m, s)
}

/// Determine day of the week for a given Julian day.
///
/// Not used, but kept for reference.
#[allow(dead_code)]
fn jwday(j: f64) -> i32 {
    (((j + 1.5).trunc() as i64) % 7).abs() as i32
}

/// Calculates time of the mean new Moon for a given base date.
///
/// This argument K to this function is the precomputed synodic month
/// index, given by:
///
/// K = (year - 1900) * 12.3685
///
/// where year is expressed as a year and fractional year.
fn meanphase(sdate: f64, k: f64) -> f64 {
    // Time in Julian centuries from 1900 January 0.5
    let t = (sdate - 2_415_020.0) / 36525.0;
    let t2 = t * t; // Square for frequent use
    let t3 = t2 * t; // Cube for frequent use

    2_415_020.759_33 + SYNMONTH * k + 0.000_117_8 * t2 - 0.000_000_155 * t3
        + 0.00033 * dsin(166.56 + 132.87 * t - 0.009_173 * t2)
}

/// True, corrected phase time.
///
/// Given a K value used to determine the mean phase of the new moon,
/// and a phase selector (0.0, 0.25, 0.5, 0.75), obtain the true,
/// corrected phase time.
///
/// # Panics
///
/// Panics if [`truephase()`] called with invalid phase selector. Phase
/// selector should be one of these values: 0.0, 0.25, 0.5, 0.75.
#[rustfmt::skip]
fn truephase(mut k: f64, phase: f64) -> f64 {
    let mut apcor = false;

    k += phase; // Add phase to new moon time
    let t = k / 1236.85; // Time in Julian centuries from 1900 January 0.5
    let t2 = t * t; // Square for frequent use
    let t3 = t2 * t; // Cube for frequent use

    // Mean time of phase
    let mut pt = 2_415_020.759_33 + SYNMONTH * k + 0.000_117_8 * t2 - 0.000_000_155 * t3
        + 0.00033 * dsin(166.56 + 132.87 * t - 0.009_173 * t2);
    // Sun's mean anomaly
    let m = 359.2242 + 29.105_356_08 * k - 0.000_033_3 * t2 - 0.000_003_47 * t3;
    // Moon's mean anomaly
    let mprime = 306.0253 + 385.816_918_06 * k + 0.010_730_6 * t2 + 0.000_012_36 * t3;
    // Moon's argument of latitude
    let f = 21.2964 + 390.670_506_46 * k - 0.001_652_8 * t2 - 0.000_002_39 * t3;

    if phase < 0.01 || (phase - 0.5).abs() < 0.01 {
        // Corrections for New and Full Moon
        pt += (0.1734 - 0.000_393 * t) * dsin(m)
            + 0.0021 * dsin(2.0 * m)
            - 0.4068 * dsin(mprime)
            + 0.0161 * dsin(2.0 * mprime)
            - 0.0004 * dsin(3.0 * mprime)
            + 0.0104 * dsin(2.0 * f)
            - 0.0051 * dsin(m + mprime)
            - 0.0074 * dsin(m - mprime)
            + 0.0004 * dsin(2.0 * f + m)
            - 0.0004 * dsin(2.0 * f - m)
            - 0.0006 * dsin(2.0 * f + mprime)
            + 0.0010 * dsin(2.0 * f - mprime)
            + 0.0005 * dsin(m + 2.0 * mprime);
        apcor = true;
    } else if (phase - 0.25).abs() < 0.01 || (phase - 0.75).abs() < 0.01 {
        pt += (0.1721 - 0.0004 * t) * dsin(m)
            + 0.0021 * dsin(2.0 * m)
            - 0.6280 * dsin(mprime)
            + 0.0089 * dsin(2.0 * mprime)
            - 0.0004 * dsin(3.0 * mprime)
            + 0.0079 * dsin(2.0 * f)
            - 0.0119 * dsin(m + mprime)
            - 0.0047 * dsin(m - mprime)
            + 0.0003 * dsin(2.0 * f + m)
            - 0.0004 * dsin(2.0 * f - m)
            - 0.0006 * dsin(2.0 * f + mprime)
            + 0.0021 * dsin(2.0 * f - mprime)
            + 0.0003 * dsin(m + 2.0 * mprime)
            + 0.0004 * dsin(m - 2.0 * mprime)
            - 0.0003 * dsin(2.0 * m + mprime);
        if phase < 0.5 {
            // First quarter correction
            pt += 0.0028 - 0.0004 * dcos(m) + 0.0003 * dcos(mprime);
        } else {
            // Last quarter correction
            pt += -0.0028 + 0.0004 * dcos(m) - 0.0003 * dcos(mprime);
        }
        apcor = true;
    }
    assert!(apcor, "TRUEPHASE called with invalid phase selector.");
    pt
}

/// Find time of phases of the moon which surround the current date.
///
/// Five phases are found, starting and ending with the new moons which
/// bound the current lunation.
fn phasehunt(sdate: f64) -> (f64, f64, f64, f64, f64) {
    let mut adate = sdate - 45.0;

    let ymd = jyear(adate);
    let yy = f64::from(ymd.0);
    let mm = f64::from(ymd.1);

    let mut k1 = ((yy + ((mm - 1.0) * (1.0 / 12.0)) - 1900.0) * 12.3685).floor();
    let mut k2;

    adate = meanphase(adate, k1);
    let mut nt1 = adate;
    let mut nt2;
    loop {
        adate += SYNMONTH;
        k2 = k1 + 1.0;
        nt2 = meanphase(adate, k2);
        if nt1 <= sdate && nt2 > sdate {
            break;
        }
        nt1 = nt2;
        k1 = k2;
    }

    (
        truephase(k1, 0.0),
        truephase(k1, 0.25),
        truephase(k1, 0.5),
        truephase(k1, 0.75),
        truephase(k2, 0.0),
    )
}

/// Solve the equation of Kepler.
fn kepler(mut m: f64, ecc: f64) -> f64 {
    // `f64::EPSILON` (machine epsilon) is too small, which caused
    // infinite loops here in some cases. Now we use the same value as
    // the C version, which is precise enough (tests still pass).
    const EPSILON: f64 = 1e-6;

    m = m.to_radians();
    let mut e = m;

    loop {
        let delta = e - ecc * e.sin() - m;
        e -= delta / (1.0 - ecc * e.cos());
        if delta.abs() <= EPSILON {
            break e;
        }
    }
}

/// Calculate phase of moon as a fraction.
///
/// The argument is the time for which the phase is requested, expressed
/// as a Julian date and fraction. Returns as a struct the terminator
/// phase angle as a percentage of a full circle (i.e., 0 to 1), the
/// illuminated fraction of the Moon's disc, the Moon's age in days and
/// fraction, the distance of the Moon from the centre of the Earth, and
/// the angular diameter subtended by the Moon as seen by an observer at
/// the centre of the Earth.
#[allow(non_snake_case)]
fn phase(pdate: f64) -> PhaseInfo {
    let Day: f64 = pdate - EPOCH; // Date within epoch
    let N: f64 = fixangle((360.0 / 365.2422) * Day); // Mean anomaly of the Sun
    let M: f64 = fixangle(N + ELONGE - ELONGP); // Convert from perigee co-ordinates to epoch 1980.0

    let mut Ec: f64 = kepler(M, ECCENT); // Solve equation of Kepler
    Ec = ((1.0 + ECCENT) / (1.0 - ECCENT)).sqrt() * (Ec / 2.0).tan();
    let Ec: f64 = 2.0 * Ec.atan().to_degrees(); // True anomaly
    let Lambdasun: f64 = fixangle(Ec + ELONGP); // Sun's geocentric ecliptic longitude

    // Orbital distance factor
    let F = (1.0 + ECCENT * Ec.to_radians().cos()) / (1.0 - ECCENT * ECCENT);
    let SunDist = SUNSMAX / F; // Distance to Sun in km
    let SunAng = F * SUNANGSIZ; // Sun's angular size in degrees

    // Calculation of the Moon's position

    // Moon's mean longitude
    let ml = fixangle(13.176_396_6 * Day + MMLONG);

    // Moon's mean anomaly
    let MM = fixangle(ml - 0.111_404_1 * Day - MMLONGP);

    // Moon's ascending node mean longitude
    let MN = fixangle(MLNODE - 0.052_953_9 * Day);

    // Evection
    let Ev = 1.2739 * (2.0 * (ml - Lambdasun) - MM).to_radians().sin();

    // Annual equation
    let Ae = 0.1858 * M.to_radians().sin();

    // Correction term
    let A3 = 0.37 * M.to_radians().sin();

    // Corrected anomaly
    let MmP = MM + Ev - Ae - A3;

    // Correction for the equation of the centre
    let mEc = 6.2886 * MmP.to_radians().sin();

    // Another correction term
    let A4 = 0.214 * (2.0 * MmP).to_radians().sin();

    // Corrected longitude
    let lP = ml + Ev + mEc - Ae + A4;

    // Variation
    let V = 0.6583 * (2.0 * (lP - Lambdasun)).to_radians().sin();

    // True longitude
    let lPP = lP + V;

    // Calculation of the Moon's inclination
    // (unused for phase calculation).

    // Corrected longitude of the node
    let NP = MN - 0.16 * M.to_radians().sin();

    // Y inclination coordinate
    let y = (lPP - NP).to_radians().sin() * MINC.to_radians().cos();

    // X inclination coordinate
    let x = (lPP - NP).to_radians().cos();

    // Ecliptic longitude
    let Lambdamoon = y.atan2(x).to_degrees() + NP;

    // Ecliptic latitude
    let BetaM = (lPP - NP).to_radians().sin().asin().to_degrees() * MINC.to_radians().sin();

    // Calculation of the phase of the Moon

    // Age of the Moon in degrees
    let MoonAge = lPP - Lambdasun;

    // Phase of the Moon
    let MoonPhase = (1.0 - MoonAge.to_radians().cos()) / 2.0;

    // Calculate distance of moon from the centre of the Earth

    let MoonDist = (MSMAX * (1.0 - MECC * MECC)) / (1.0 + MECC * (MmP + mEc).to_radians().cos());

    // Calculate Moon's angular diameter

    let MoonDFrac = MoonDist / MSMAX;
    let MoonAng = MANGSIZ / MoonDFrac;

    // Calculate Moon's parallax

    let MoonPar = MPARALLAX / MoonDFrac;

    let Phase = fixangle(MoonAge) / 360.0;

    PhaseInfo {
        phase: Phase,
        fraction_illuminated: MoonPhase,
        age: SYNMONTH * Phase,
        ecliptic_longitude: fixangle(Lambdamoon),
        ecliptic_latitude: BetaM,
        parallax: MoonPar,
        distance: MoonDist,
        angular_diameter: MoonAng,
        sun_ecliptic_longitude: Lambdasun,
        sun_distance: SunDist,
        sun_angular_diameter: SunAng,
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

    // Utils

    #[test]
    fn fixangle_all() {
        assert_almost_eq!(fixangle(-400.0), 320.0);
        assert_almost_eq!(fixangle(-350.0), 10.0);
        assert_almost_eq!(fixangle(-0.0), 0.0);
        assert_almost_eq!(fixangle(350.0), 350.0);
        assert_almost_eq!(fixangle(400.0), 40.0);
    }

    #[test]
    fn dsin_all() {
        assert_almost_eq!(dsin(-400.0), -0.642_787_609_686_539_3);
        assert_almost_eq!(dsin(-350.0), 0.173_648_177_666_930_4);
        assert_almost_eq!(dsin(-0.0), 0.0);
        assert_almost_eq!(dsin(350.0), -0.173_648_177_666_930_4);
        assert_almost_eq!(dsin(400.0), 0.642_787_609_686_539_3);
    }

    #[test]
    fn dcos_all() {
        assert_almost_eq!(dcos(-400.0), 0.766_044_443_118_978_1);
        assert_almost_eq!(dcos(-350.0), 0.984_807_753_012_208);
        assert_almost_eq!(dcos(-0.0), 1.0);
        assert_almost_eq!(dcos(350.0), 0.984_807_753_012_208);
        assert_almost_eq!(dcos(400.0), 0.766_044_443_118_978_1);
    }

    #[test]
    fn epl_all() {
        assert_eq!(EPL!(0), "s");
        assert_eq!(EPL!(1), "");
        assert_eq!(EPL!(2), "s");
    }

    // Custom API.

    #[test]
    fn every_way_of_creating_moonphase_gives_same_result() {
        let a = moonphase(&UTCDateTime::from_ymdhms(1968, 2, 27, 9, 10, 0));
        let b = MoonPhase::for_datetime(&UTCDateTime::from_ymdhms(1968, 2, 27, 9, 10, 0));
        let c = MoonPhase::for_ymdhms(1968, 2, 27, 9, 10, 0);
        let d = MoonPhase::for_iso_string("1968-02-27T10:10:00+01:00").unwrap();
        let e = MoonPhase::for_timestamp(-58_200_600).unwrap();
        let f = MoonPhase::for_julian_date(2_439_913.881_944_444_5);

        assert!([b, c, d, e, f].iter().all(|x| *x == a));
    }

    #[test]
    fn create_moonphase_for_date() {
        let mphase = MoonPhase::for_ymd(2024, 7, 15);

        assert_eq!(
            mphase,
            MoonPhase::for_datetime(&"2024-07-15T00:00:00Z".parse().unwrap()),
        );
    }

    #[test]
    fn moonphase_regular() {
        let mut mphase = moonphase(&UTCDateTime::from_ymdhms(1995, 3, 11, 1, 40, 0));

        // This value is slightly different across systems.
        // To simplify testing, we assert it is OK first, and then
        // normalize it.
        assert!(mphase.ecliptic_latitude.abs() - 5.389_251_414_139_025 <= 0.000_000_000_000_001);
        mphase.ecliptic_latitude = -5.389_251_414_139_025;

        assert_eq!(
            mphase,
            MoonPhase {
                julian_date: 2_449_787.569_444_444_5,
                timestamp: Some(794_886_000),
                utc_datetime: UTCDateTime::from_ymdhms(1995, 3, 11, 1, 40, 0),
                age: 8.861_826_144_635_483,
                fraction_of_lunation: 0.300_089_721_903_758_6,
                phase: 3,
                phase_name: String::from("Waxing Gibbous"),
                phase_icon: String::from("🌔"),
                fraction_illuminated: 0.654_776_546_611_648_4,
                ecliptic_longitude: 97.951_619_640_492_27,
                ecliptic_latitude: -5.389_251_414_139_025,
                parallax: 0.908_392_405_099_015_4,
                distance_to_earth_km: 402_304.145_927_074,
                distance_to_earth_earth_radii: 63.075_267_150_255_56,
                subtends: 0.495_043_762_576_837_96,
                sun_ecliptic_longitude: 350.019_412_506_235_65,
                sun_distance_to_earth_km: 148_602_888.215_602_64,
                sun_distance_to_earth_astronomical_units: 0.993_344_774_283_182_2,
                sun_subtends: 0.536_699_858_701_845_1,
            }
        );
    }

    #[test]
    fn moonphase_for_bad_timestamp() {
        let mphase = MoonPhase::for_timestamp(i64::MIN);

        assert!(mphase.is_err());
    }

    #[test]
    fn moonphase_display() {
        let mphase = moonphase(&UTCDateTime::from_ymdhms(1995, 3, 11, 1, 40, 0));

        // The testing environment is considered "unsound" by time-rs,
        // which then errors on anything local-time related. This is why
        // "Local time" does not appear in the output. This is exactly
        // what we want by the way. Otherwise, we would have to redact
        // the local time, as it varies according to the machine's
        // timezone. See: `time::util::local_offset::Soundness`.
        assert_eq!(
            mphase.to_string(),
            "\
Phase
=====

Julian date:\t\t2449787.56944   (0h variant: 2449788.06944)
Universal time:\t\tSaturday   1:40:00 11 March 1995

Age of moon:\t\t8 days, 20 hours, 41 minutes.
Lunation:\t\t30.01%   (🌔 Waxing Gibbous)
Moon phase:\t\t65.48%   (0% = New, 100% = Full)

Moon's distance:\t402304 kilometres, 63.1 Earth radii.
Moon subtends:\t\t0.4950 degrees.

Sun's distance:\t\t148602888 kilometres, 0.993 astronomical units.
Sun subtends:\t\t0.5367 degrees.\
"
        );
    }

    #[test]
    fn moonphase_to_json() {
        let mphase = moonphase(&UTCDateTime::from_ymdhms(1995, 3, 11, 1, 40, 0));

        let mut json = mphase.to_json();

        println!("{}", mphase.to_json());

        // This value is slightly different across systems.
        // To simplify testing, we normalize it.
        json = json.replace(
            r#""ecliptic_latitude":-5.389251414139024,"#,
            r#""ecliptic_latitude":-5.389251414139025,"#,
        );

        assert_eq!(
            json,
            r#"{"julian_date":2449787.5694444445,"timestamp":794886000,"utc_datetime":"1995-03-11T01:40:00Z","age":8.861826144635483,"fraction_of_lunation":0.3000897219037586,"phase":3,"phase_name":"Waxing Gibbous","phase_icon":"🌔","fraction_illuminated":0.6547765466116484,"ecliptic_longitude":97.95161964049227,"ecliptic_latitude":-5.389251414139025,"parallax":0.9083924050990154,"distance_to_earth_km":402304.145927074,"distance_to_earth_earth_radii":63.07526715025556,"subtends":0.49504376257683796,"sun_ecliptic_longitude":350.01941250623565,"sun_distance_to_earth_km":148602888.21560264,"sun_distance_to_earth_astronomical_units":0.9933447742831822,"sun_subtends":0.5366998587018451}"#,
        );
    }

    #[test]
    fn moonphase_to_json_timestamp_error() {
        let mut mphase = moonphase(&UTCDateTime::from_ymdhms(1995, 3, 11, 1, 40, 0));
        mphase.timestamp = None;

        assert!(mphase.to_json().contains(r#""timestamp":null,"#));
    }

    #[test]
    fn every_way_of_creating_mooncalendar_gives_same_result() {
        let a = mooncal(&UTCDateTime::from_ymdhms(1968, 2, 27, 9, 10, 0));
        let b = MoonCalendar::for_datetime(&UTCDateTime::from_ymdhms(1968, 2, 27, 9, 10, 0));
        let c = MoonCalendar::for_ymdhms(1968, 2, 27, 9, 10, 0);
        let d = MoonCalendar::for_iso_string("1968-02-27T10:10:00+01:00").unwrap();
        let e = MoonCalendar::for_timestamp(-58_200_600).unwrap();
        let f = MoonCalendar::for_julian_date(2_439_913.881_944_444_5);

        assert!([b, c, d, e, f].iter().all(|x| *x == a));
    }

    #[test]
    fn create_mooncalendar_for_date() {
        let mcal = MoonCalendar::for_ymd(2024, 7, 15);

        assert_eq!(
            mcal,
            MoonCalendar::for_datetime(&"2024-07-15T00:00:00Z".parse().unwrap()),
        );
    }

    #[test]
    fn mooncalendar_regular() {
        let mcal = mooncal(&UTCDateTime::from_ymdhms(1995, 3, 11, 1, 40, 0));

        assert_eq!(
            mcal,
            MoonCalendar {
                julian_date: 2_449_787.569_444_444_5,
                timestamp: Some(794_886_000),
                utc_datetime: UTCDateTime::from_ymdhms(1995, 3, 11, 1, 40, 0),
                lunation: 893,
                last_new_moon: 2_449_777.993_024_320_3,
                last_new_moon_utc: UTCDateTime::from_ymdhms(1995, 3, 1, 11, 49, 57),
                first_quarter: 2_449_785.925_942_567_6,
                first_quarter_utc: UTCDateTime::from_ymdhms(1995, 3, 9, 10, 13, 21),
                full_moon: 2_449_793.560_731_158_6,
                full_moon_utc: UTCDateTime::from_ymdhms(1995, 3, 17, 1, 27, 27),
                last_quarter: 2_449_800.341_072_181_2,
                last_quarter_utc: UTCDateTime::from_ymdhms(1995, 3, 23, 20, 11, 9),
                next_new_moon: 2_449_807.590_823_359_3,
                next_new_moon_utc: UTCDateTime::from_ymdhms(1995, 3, 31, 2, 10, 47),
            }
        );
    }

    #[test]
    fn mooncalendar_for_bad_timestamp() {
        let mcal = MoonCalendar::for_timestamp(i64::MIN);

        assert!(mcal.is_err());
    }

    #[test]
    fn mooncalendar_display() {
        let mcal = mooncal(&UTCDateTime::from_ymdhms(1995, 3, 11, 1, 40, 0));

        assert_eq!(
            mcal.to_string(),
            "\
Moon Calendar
=============

Last new moon:\t\tWednesday 11:49 UTC  1 March 1995\tLunation: 893
First quarter:\t\tThursday  10:13 UTC  9 March 1995
Full moon:\t\tFriday     1:27 UTC 17 March 1995
Last quarter:\t\tThursday  20:11 UTC 23 March 1995
Next new moon:\t\tFriday     2:10 UTC 31 March 1995\tLunation: 894\
"
        );
    }

    #[test]
    fn mooncalendar_to_json() {
        let mcal = mooncal(&UTCDateTime::from_ymdhms(1995, 3, 11, 1, 40, 0));

        println!("{}", mcal.to_json());
        assert_eq!(
            mcal.to_json(),
            r#"{"julian_date":2449787.5694444445,"timestamp":794886000,"utc_datetime":"1995-03-11T01:40:00Z","lunation":893,"last_new_moon":2449777.9930243203,"last_new_moon_utc":"1995-03-01T11:49:57Z","first_quarter":2449785.9259425676,"first_quarter_utc":"1995-03-09T10:13:21Z","full_moon":2449793.5607311586,"full_moon_utc":"1995-03-17T01:27:27Z","last_quarter":2449800.3410721812,"last_quarter_utc":"1995-03-23T20:11:09Z","next_new_moon":2449807.5908233593,"next_new_moon_utc":"1995-03-31T02:10:47Z"}"#,
        );
    }

    #[test]
    fn mooncalendar_to_json_timestamp_error() {
        let mut mcal = mooncal(&UTCDateTime::from_ymdhms(1995, 3, 11, 1, 40, 0));
        mcal.timestamp = None;

        assert!(mcal.to_json().contains(r#""timestamp":null,"#));
    }

    #[test]
    fn every_way_of_creating_yearly_mooncalendar_gives_same_result() {
        let a = yearly_mooncal(&UTCDateTime::from_ymdhms(1968, 2, 27, 9, 10, 0));
        let b = YearlyMoonCalendar::for_datetime(&UTCDateTime::from_ymdhms(1968, 2, 27, 9, 10, 0));
        let c = YearlyMoonCalendar::for_ymdhms(1968, 2, 27, 9, 10, 0);
        let d = YearlyMoonCalendar::for_iso_string("1968-02-27T10:10:00+01:00").unwrap();
        let e = YearlyMoonCalendar::for_timestamp(-58_200_600).unwrap();
        let f = YearlyMoonCalendar::for_julian_date(2_439_913.881_944_444_5);

        assert!([b, c, d, e, f].iter().all(|x| *x == a));
    }

    #[test]
    fn create_yearly_mooncalendar_for_date() {
        let ymcal = YearlyMoonCalendar::for_ymd(2024, 7, 15);

        assert_eq!(
            ymcal,
            YearlyMoonCalendar::for_datetime(&"2024-07-15T00:00:00Z".parse().unwrap()),
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn yearly_mooncalendar_regular() {
        let ymcal = yearly_mooncal(&UTCDateTime::from_ymdhms(1995, 3, 11, 1, 40, 0));

        assert_eq!(
            ymcal,
            YearlyMoonCalendar {
                julian_date: 2_449_787.569_444_444_5,
                timestamp: Some(794_886_000),
                utc_datetime: UTCDateTime::from_ymdhms(1995, 3, 11, 1, 40, 0),
                new_moons: vec![
                    NewMoon {
                        date: 2_449_718.956_136_873_5,
                        date_utc: UTCDateTime::from_ymdhms(1995, 1, 1, 10, 56, 50)
                    },
                    NewMoon {
                        date: 2_449_748.451_091_56,
                        date_utc: UTCDateTime::from_ymdhms(1995, 1, 30, 22, 49, 34)
                    },
                    NewMoon {
                        date: 2_449_777.993_024_320_3,
                        date_utc: UTCDateTime::from_ymdhms(1995, 3, 1, 11, 49, 57)
                    },
                    NewMoon {
                        date: 2_449_807.590_823_359_3,
                        date_utc: UTCDateTime::from_ymdhms(1995, 3, 31, 2, 10, 47)
                    },
                    NewMoon {
                        date: 2_449_837.234_842_154_7,
                        date_utc: UTCDateTime::from_ymdhms(1995, 4, 29, 17, 38, 10)
                    },
                    NewMoon {
                        date: 2_449_866.894_783_045,
                        date_utc: UTCDateTime::from_ymdhms(1995, 5, 29, 9, 28, 29)
                    },
                    NewMoon {
                        date: 2_449_896.535_279_648,
                        date_utc: UTCDateTime::from_ymdhms(1995, 6, 28, 0, 50, 48)
                    },
                    NewMoon {
                        date: 2_449_926.134_210_367,
                        date_utc: UTCDateTime::from_ymdhms(1995, 7, 27, 15, 13, 16)
                    },
                    NewMoon {
                        date: 2_449_955.688_148_399_3,
                        date_utc: UTCDateTime::from_ymdhms(1995, 8, 26, 4, 30, 56)
                    },
                    NewMoon {
                        date: 2_449_985.204_571_035,
                        date_utc: UTCDateTime::from_ymdhms(1995, 9, 24, 16, 54, 35)
                    },
                    NewMoon {
                        date: 2_450_014.691_681_338,
                        date_utc: UTCDateTime::from_ymdhms(1995, 10, 24, 4, 36, 1)
                    },
                    NewMoon {
                        date: 2_450_044.154_738_946,
                        date_utc: UTCDateTime::from_ymdhms(1995, 11, 22, 15, 42, 49)
                    },
                    NewMoon {
                        date: 2_450_073.599_341_999,
                        date_utc: UTCDateTime::from_ymdhms(1995, 12, 22, 2, 23, 3)
                    }
                ],
                full_moons: vec![
                    FullMoon {
                        date: 2_449_734.352_721_255_3,
                        date_utc: UTCDateTime::from_ymdhms(1995, 1, 16, 20, 27, 55),
                        name: String::from("Wolf Moon")
                    },
                    FullMoon {
                        date: 2_449_764.011_966_952_6,
                        date_utc: UTCDateTime::from_ymdhms(1995, 2, 15, 12, 17, 14),
                        name: String::from("Snow Moon")
                    },
                    FullMoon {
                        date: 2_449_793.560_731_158_6,
                        date_utc: UTCDateTime::from_ymdhms(1995, 3, 17, 1, 27, 27),
                        name: String::from("Worm Moon")
                    },
                    FullMoon {
                        date: 2_449_823.006_760_471,
                        date_utc: UTCDateTime::from_ymdhms(1995, 4, 15, 12, 9, 44),
                        name: String::from("Pink Moon")
                    },
                    FullMoon {
                        date: 2_449_852.367_306_99,
                        date_utc: UTCDateTime::from_ymdhms(1995, 5, 14, 20, 48, 55),
                        name: String::from("Flower Moon")
                    },
                    FullMoon {
                        date: 2_449_881.669_201_127,
                        date_utc: UTCDateTime::from_ymdhms(1995, 6, 13, 4, 3, 39),
                        name: String::from("Strawberry Moon")
                    },
                    FullMoon {
                        date: 2_449_910.950_985_403_7,
                        date_utc: UTCDateTime::from_ymdhms(1995, 7, 12, 10, 49, 25),
                        name: String::from("Buck Moon")
                    },
                    FullMoon {
                        date: 2_449_940.260_853_294_7,
                        date_utc: UTCDateTime::from_ymdhms(1995, 8, 10, 18, 15, 38),
                        name: String::from("Sturgeon Moon")
                    },
                    FullMoon {
                        date: 2_449_969.650_321_038_4,
                        date_utc: UTCDateTime::from_ymdhms(1995, 9, 9, 3, 36, 28),
                        name: String::from("Harvest Moon")
                    },
                    FullMoon {
                        date: 2_449_999.161_113_315_3,
                        date_utc: UTCDateTime::from_ymdhms(1995, 10, 8, 15, 52, 0),
                        name: String::from("Hunter's Moon")
                    },
                    FullMoon {
                        date: 2_450_028.806_614_596_4,
                        date_utc: UTCDateTime::from_ymdhms(1995, 11, 7, 7, 21, 32),
                        name: String::from("Beaver Moon")
                    },
                    FullMoon {
                        date: 2_450_058.561_306_783,
                        date_utc: UTCDateTime::from_ymdhms(1995, 12, 7, 1, 28, 17),
                        name: String::from("Cold Moon")
                    }
                ],
            }
        );
    }

    #[test]
    fn yearly_mooncalendar_year_with_12_new_moons() {
        let ymcal = yearly_mooncal(&UTCDateTime::from_ymdhms(2023, 1, 1, 0, 0, 0));

        assert_eq!(ymcal.new_moons.len(), 12);
    }

    #[test]
    fn yearly_mooncalendar_two_new_moons_in_same_month() {
        let ymcal = yearly_mooncal(&UTCDateTime::from_ymdhms(2019, 1, 1, 0, 0, 0));

        assert_eq!(
            ymcal.new_moons[7].date_utc.to_string(),
            "2019-08-01T03:12:55Z"
        );

        assert_eq!(
            ymcal.new_moons[8].date_utc.to_string(),
            "2019-08-30T10:38:21Z"
        );
    }

    #[test]
    fn yearly_mooncalendar_blue_moon() {
        let ymcal = yearly_mooncal(&UTCDateTime::from_ymdhms(1999, 1, 1, 0, 0, 0));

        assert_eq!(
            ymcal.full_moons[0].date_utc.to_string(),
            "1999-01-02T02:51:24Z"
        );
        assert_eq!(ymcal.full_moons[0].name, "Wolf Moon");

        assert_eq!(
            ymcal.full_moons[1].date_utc.to_string(),
            "1999-01-31T16:08:16Z"
        );
        assert_eq!(ymcal.full_moons[1].name, "Blue Moon");

        assert_eq!(
            ymcal.full_moons[2].date_utc.to_string(),
            "1999-03-02T06:59:52Z"
        );
        assert_eq!(ymcal.full_moons[2].name, "Worm Moon");

        assert_eq!(
            ymcal.full_moons[3].date_utc.to_string(),
            "1999-03-31T22:49:59Z"
        );
        assert_eq!(ymcal.full_moons[3].name, "Blue Moon");
    }

    #[test]
    fn yearly_mooncalendar_harvest_moon_in_october() {
        let ymcal = yearly_mooncal(&UTCDateTime::from_ymdhms(2001, 1, 1, 0, 0, 0));

        assert_eq!(
            ymcal.full_moons[8].date_utc.to_string(),
            "2001-09-02T21:44:53Z"
        );
        assert_eq!(ymcal.full_moons[8].name, "Corn Moon");

        assert_eq!(
            ymcal.full_moons[9].date_utc.to_string(),
            "2001-10-02T13:50:48Z"
        );
        assert_eq!(ymcal.full_moons[9].name, "Harvest Moon");

        assert_eq!(
            ymcal.full_moons[10].date_utc.to_string(),
            "2001-11-01T05:43:10Z"
        );
        assert_eq!(ymcal.full_moons[10].name, "Hunter's Moon");
    }

    #[test]
    fn yearly_mooncalendar_harvest_and_hunters_moons_override_blue_moon() {
        let ymcal = yearly_mooncal(&UTCDateTime::from_ymdhms(2020, 1, 1, 0, 0, 0));

        assert_eq!(
            ymcal.full_moons[9].date_utc.to_string(),
            "2020-10-01T21:06:55Z"
        );
        assert_eq!(ymcal.full_moons[9].name, "Harvest Moon");

        assert_eq!(
            ymcal.full_moons[10].date_utc.to_string(),
            "2020-10-31T14:51:30Z"
        );
        assert_eq!(ymcal.full_moons[10].name, "Hunter's Moon");
    }

    #[test]
    fn yearly_mooncalendar_year_with_13_full_moons() {
        let ymcal = yearly_mooncal(&UTCDateTime::from_ymdhms(2020, 1, 1, 0, 0, 0));

        assert_eq!(ymcal.full_moons.len(), 13);
    }

    #[test]
    fn yearly_mooncalendar_for_bad_timestamp() {
        let ymcal = YearlyMoonCalendar::for_timestamp(i64::MIN);

        assert!(ymcal.is_err());
    }

    #[test]
    fn yearly_mooncalendar_display() {
        let mcal = yearly_mooncal(&UTCDateTime::from_ymdhms(1995, 3, 11, 1, 40, 0));

        assert_eq!(
            mcal.to_string(),
            "\
New Moons
=========

 1. Sunday    10:56 UTC  1 January 1995
 2. Monday    22:49 UTC 30 January 1995
 3. Wednesday 11:49 UTC  1 March 1995
 4. Friday     2:10 UTC 31 March 1995
 5. Saturday  17:38 UTC 29 April 1995
 6. Monday     9:28 UTC 29 May   1995
 7. Wednesday  0:50 UTC 28 June  1995
 8. Thursday  15:13 UTC 27 July  1995
 9. Saturday   4:30 UTC 26 August 1995
10. Sunday    16:54 UTC 24 September 1995
11. Tuesday    4:36 UTC 24 October 1995
12. Wednesday 15:42 UTC 22 November 1995
13. Friday     2:23 UTC 22 December 1995

Full Moons
==========

 1. Monday    20:27 UTC 16 January 1995     Wolf Moon
 2. Wednesday 12:17 UTC 15 February 1995    Snow Moon
 3. Friday     1:27 UTC 17 March 1995       Worm Moon
 4. Saturday  12:09 UTC 15 April 1995       Pink Moon
 5. Sunday    20:48 UTC 14 May   1995       Flower Moon
 6. Tuesday    4:03 UTC 13 June  1995       Strawberry Moon
 7. Wednesday 10:49 UTC 12 July  1995       Buck Moon
 8. Thursday  18:15 UTC 10 August 1995      Sturgeon Moon
 9. Saturday   3:36 UTC  9 September 1995   Harvest Moon
10. Sunday    15:52 UTC  8 October 1995     Hunter's Moon
11. Tuesday    7:21 UTC  7 November 1995    Beaver Moon
12. Thursday   1:28 UTC  7 December 1995    Cold Moon\
"
        );
    }

    #[test]
    fn yearly_mooncalendar_to_json() {
        let ymcal = yearly_mooncal(&UTCDateTime::from_ymdhms(1995, 3, 11, 1, 40, 0));

        println!("{}", ymcal.to_json());
        assert_eq!(
            ymcal.to_json(),
            r#"{"julian_date":2449787.5694444445,"timestamp":794886000,"new_moons":[{"date":2449718.9561368735,"date_utc":"1995-01-01T10:56:50Z"},{"date":2449748.45109156,"date_utc":"1995-01-30T22:49:34Z"},{"date":2449777.9930243203,"date_utc":"1995-03-01T11:49:57Z"},{"date":2449807.5908233593,"date_utc":"1995-03-31T02:10:47Z"},{"date":2449837.2348421547,"date_utc":"1995-04-29T17:38:10Z"},{"date":2449866.894783045,"date_utc":"1995-05-29T09:28:29Z"},{"date":2449896.535279648,"date_utc":"1995-06-28T00:50:48Z"},{"date":2449926.134210367,"date_utc":"1995-07-27T15:13:16Z"},{"date":2449955.6881483993,"date_utc":"1995-08-26T04:30:56Z"},{"date":2449985.204571035,"date_utc":"1995-09-24T16:54:35Z"},{"date":2450014.691681338,"date_utc":"1995-10-24T04:36:01Z"},{"date":2450044.154738946,"date_utc":"1995-11-22T15:42:49Z"},{"date":2450073.599341999,"date_utc":"1995-12-22T02:23:03Z"}],"full_moons":[{"date":2449734.3527212553,"date_utc":"1995-01-16T20:27:55Z","name":"Wolf Moon"},{"date":2449764.0119669526,"date_utc":"1995-02-15T12:17:14Z","name":"Snow Moon"},{"date":2449793.5607311586,"date_utc":"1995-03-17T01:27:27Z","name":"Worm Moon"},{"date":2449823.006760471,"date_utc":"1995-04-15T12:09:44Z","name":"Pink Moon"},{"date":2449852.36730699,"date_utc":"1995-05-14T20:48:55Z","name":"Flower Moon"},{"date":2449881.669201127,"date_utc":"1995-06-13T04:03:39Z","name":"Strawberry Moon"},{"date":2449910.9509854037,"date_utc":"1995-07-12T10:49:25Z","name":"Buck Moon"},{"date":2449940.2608532947,"date_utc":"1995-08-10T18:15:38Z","name":"Sturgeon Moon"},{"date":2449969.6503210384,"date_utc":"1995-09-09T03:36:28Z","name":"Harvest Moon"},{"date":2449999.1611133153,"date_utc":"1995-10-08T15:52:00Z","name":"Hunter's Moon"},{"date":2450028.8066145964,"date_utc":"1995-11-07T07:21:32Z","name":"Beaver Moon"},{"date":2450058.561306783,"date_utc":"1995-12-07T01:28:17Z","name":"Cold Moon"}]}"#,
        );
    }

    #[test]
    fn yearly_mooncalendar_to_json_timestamp_error() {
        let mut ymcal = yearly_mooncal(&UTCDateTime::from_ymdhms(1995, 3, 11, 1, 40, 0));
        ymcal.timestamp = None;

        assert!(ymcal.to_json().contains(r#""timestamp":null,"#));
    }

    #[test]
    fn every_way_of_creating_suncalendar_gives_same_result() {
        let a = suncal(&UTCDateTime::from_ymdhms(1968, 2, 27, 9, 10, 0));
        let b = SunCalendar::for_datetime(&UTCDateTime::from_ymdhms(1968, 2, 27, 9, 10, 0));
        let c = SunCalendar::for_ymdhms(1968, 2, 27, 9, 10, 0);
        let d = SunCalendar::for_iso_string("1968-02-27T10:10:00+01:00").unwrap();
        let e = SunCalendar::for_timestamp(-58_200_600).unwrap();
        let f = SunCalendar::for_julian_date(2_439_913.881_944_444_5);

        assert!([b, c, d, e, f].iter().all(|x| *x == a));
    }

    #[test]
    fn create_suncalendar_for_date() {
        let scal = SunCalendar::for_ymd(2024, 7, 15);

        assert_eq!(
            scal,
            SunCalendar::for_datetime(&"2024-07-15T00:00:00Z".parse().unwrap()),
        );
    }

    #[test]
    fn create_suncalendar_for_year() {
        let scal = SunCalendar::for_year(2024);

        assert_eq!(
            scal,
            SunCalendar::for_datetime(&"2024-01-01T00:00:00Z".parse().unwrap()),
        );
    }

    #[test]
    fn suncalendar_regular() {
        let scal = suncal(&UTCDateTime::from_ymdhms(1995, 3, 11, 1, 40, 0));

        assert_eq!(
            scal,
            SunCalendar {
                julian_date: 2_449_787.569_444_444_5,
                timestamp: Some(794_886_000),
                utc_datetime: UTCDateTime::from_ymdhms(1995, 3, 11, 1, 40, 0),
                march_equinox: 2_449_797.594_275_648_7,
                march_equinox_utc: UTCDateTime::from_ymdhms(1995, 3, 21, 2, 15, 45),
                june_solstice: 2_449_890.357_965_532_7,
                june_solstice_utc: UTCDateTime::from_ymdhms(1995, 6, 21, 20, 35, 28),
                september_equinox: 2_449_984.009_840_158_3,
                september_equinox_utc: UTCDateTime::from_ymdhms(1995, 9, 23, 12, 14, 10),
                december_solstice: 2_450_073.845_976_675_4,
                december_solstice_utc: UTCDateTime::from_ymdhms(1995, 12, 22, 8, 18, 12),
            }
        );
    }

    #[test]
    fn suncalendar_before_1000_ad() {
        let scal = suncal(&UTCDateTime::from_ymdhms(420, 3, 6, 9, 42, 12));

        assert_eq!(
            scal,
            SunCalendar {
                julian_date: 1_874_527.904_305_555_6,
                timestamp: Some(-48_907_635_468),
                utc_datetime: UTCDateTime::from_ymdhms(420, 3, 6, 9, 42, 12),
                march_equinox: 1_874_541.007_477_060_2,
                march_equinox_utc: UTCDateTime::from_ymdhms(420, 3, 19, 12, 10, 46),
                june_solstice: 1_874_634.777_248_703_4,
                june_solstice_utc: UTCDateTime::from_ymdhms(420, 6, 21, 6, 39, 14),
                september_equinox: 1_874_727.537_210_142,
                september_equinox_utc: UTCDateTime::from_ymdhms(420, 9, 22, 0, 53, 35),
                december_solstice: 1_874_816.418_745_953_4,
                december_solstice_utc: UTCDateTime::from_ymdhms(420, 12, 19, 22, 3, 0),
            }
        );
    }

    #[test]
    fn suncalendar_for_bad_timestamp() {
        let scal = SunCalendar::for_timestamp(i64::MIN);

        assert!(scal.is_err());
    }

    #[test]
    fn suncalendar_display() {
        let scal = suncal(&UTCDateTime::from_ymdhms(1995, 3, 11, 1, 40, 0));

        assert_eq!(
            scal.to_string(),
            "\
Sun Calendar
============

March equinox:\t\tTuesday    2:15 UTC 21 March 1995
June solstice:\t\tWednesday 20:35 UTC 21 June  1995
September equinox:\tSaturday  12:14 UTC 23 September 1995
December solstice:\tFriday     8:18 UTC 22 December 1995\
"
        );
    }

    #[test]
    fn suncalendar_to_json() {
        let scal = suncal(&UTCDateTime::from_ymdhms(1995, 3, 11, 1, 40, 0));

        println!("{}", scal.to_json());
        assert_eq!(
            scal.to_json(),
            r#"{"julian_date":2449787.5694444445,"timestamp":794886000,"utc_datetime":"1995-03-11T01:40:00Z","march_equinox":2449797.5942756487,"march_equinox_utc":"1995-03-21T02:15:45Z","june_solstice":2449890.3579655327,"june_solstice_utc":"1995-06-21T20:35:28Z","september_equinox":2449984.0098401583,"september_equinox_utc":"1995-09-23T12:14:10Z","december_solstice":2450073.8459766754,"december_solstice_utc":"1995-12-22T08:18:12Z"}"#,
        );
    }

    #[test]
    fn suncalendar_to_json_timestamp_error() {
        let mut scal = suncal(&UTCDateTime::from_ymdhms(1995, 3, 11, 1, 40, 0));
        scal.timestamp = None;

        assert!(scal.to_json().contains(r#""timestamp":null,"#));
    }

    // Moon

    #[test]
    fn fraction_of_lunation_to_phase_number() {
        let new_moon_start = fraction_of_lunation_to_phase(0.0);
        assert_eq!(new_moon_start, 0);

        let waxing_crescent = fraction_of_lunation_to_phase(0.15);
        assert_eq!(waxing_crescent, 1);

        let first_quarter = fraction_of_lunation_to_phase(0.25);
        assert_eq!(first_quarter, 2);

        let waxing_gibbous = fraction_of_lunation_to_phase(0.35);
        assert_eq!(waxing_gibbous, 3);

        let full_moon = fraction_of_lunation_to_phase(0.5);
        assert_eq!(full_moon, 4);

        let waning_gibbous = fraction_of_lunation_to_phase(0.65);
        assert_eq!(waning_gibbous, 5);

        let last_quarter = fraction_of_lunation_to_phase(0.75);
        assert_eq!(last_quarter, 6);

        let waning_crescent = fraction_of_lunation_to_phase(0.85);
        assert_eq!(waning_crescent, 7);

        let new_moon_end = fraction_of_lunation_to_phase(1.0);
        assert_eq!(new_moon_end, 0);
    }

    #[test]
    fn fraction_of_lunation_to_phase_name() {
        let new_moon_start = PHANAME[fraction_of_lunation_to_phase(0.0)];
        assert_eq!(new_moon_start, "New Moon");

        let waxing_crescent = PHANAME[fraction_of_lunation_to_phase(0.15)];
        assert_eq!(waxing_crescent, "Waxing Crescent");

        let first_quarter = PHANAME[fraction_of_lunation_to_phase(0.25)];
        assert_eq!(first_quarter, "First Quarter");

        let waxing_gibbous = PHANAME[fraction_of_lunation_to_phase(0.35)];
        assert_eq!(waxing_gibbous, "Waxing Gibbous");

        let full_moon = PHANAME[fraction_of_lunation_to_phase(0.5)];
        assert_eq!(full_moon, "Full Moon");

        let waning_gibbous = PHANAME[fraction_of_lunation_to_phase(0.65)];
        assert_eq!(waning_gibbous, "Waning Gibbous");

        let last_quarter = PHANAME[fraction_of_lunation_to_phase(0.75)];
        assert_eq!(last_quarter, "Last Quarter");

        let waning_crescent = PHANAME[fraction_of_lunation_to_phase(0.85)];
        assert_eq!(waning_crescent, "Waning Crescent");

        let new_moon_end = PHANAME[fraction_of_lunation_to_phase(1.0)];
        assert_eq!(new_moon_end, "New Moon");
    }

    #[test]
    fn fraction_of_lunation_to_phase_icon() {
        let new_moon_start = MOONICN[fraction_of_lunation_to_phase(0.0)];
        assert_eq!(new_moon_start, "🌑");

        let waxing_crescent = MOONICN[fraction_of_lunation_to_phase(0.15)];
        assert_eq!(waxing_crescent, "🌒");

        let first_quarter = MOONICN[fraction_of_lunation_to_phase(0.25)];
        assert_eq!(first_quarter, "🌓");

        let waxing_gibbous = MOONICN[fraction_of_lunation_to_phase(0.35)];
        assert_eq!(waxing_gibbous, "🌔");

        let full_moon = MOONICN[fraction_of_lunation_to_phase(0.5)];
        assert_eq!(full_moon, "🌕");

        let waning_gibbous = MOONICN[fraction_of_lunation_to_phase(0.65)];
        assert_eq!(waning_gibbous, "🌖");

        let last_quarter = MOONICN[fraction_of_lunation_to_phase(0.75)];
        assert_eq!(last_quarter, "🌗");

        let waning_crescent = MOONICN[fraction_of_lunation_to_phase(0.85)];
        assert_eq!(waning_crescent, "🌘");

        let new_moon_end = MOONICN[fraction_of_lunation_to_phase(1.0)];
        assert_eq!(new_moon_end, "🌑");
    }

    #[test]
    fn fmt_phase_time_regular() {
        let gm = UTCDateTime::from_ymdhms(1995, 3, 12, 11, 16, 26);

        let res = fmt_phase_time(&gm);

        assert_eq!(res, "Sunday    11:16 UTC 12 March 1995");
    }

    #[test]
    fn fmt_phase_time_month_padding() {
        let mut gm = UTCDateTime::from_ymdhms(1995, 3, 12, 11, 16, 26);

        gm.month = 5; // May (shortest)
        assert_eq!(fmt_phase_time(&gm), "Friday    11:16 UTC 12 May   1995");

        gm.month = 9; // September (longest)
        assert_eq!(fmt_phase_time(&gm), "Tuesday   11:16 UTC 12 September 1995");

        gm.month = 7; // July (4 chars = 1 char padding)
        assert_eq!(fmt_phase_time(&gm), "Wednesday 11:16 UTC 12 July  1995");

        gm.month = 3; // March (5 chars = exact)
        assert_eq!(fmt_phase_time(&gm), "Sunday    11:16 UTC 12 March 1995");

        gm.month = 8; // August (6 chars = no padding)
        assert_eq!(fmt_phase_time(&gm), "Saturday  11:16 UTC 12 August 1995");
    }

    #[test]
    fn fmt_phase_time_at_boundaries() {
        let mut gm = UTCDateTime::from_ymdhms(1995, 3, 12, 11, 16, 26);

        gm.day = 12; // Sunday
        assert_eq!(fmt_phase_time(&gm), "Sunday    11:16 UTC 12 March 1995");

        gm.day = 13; // Monday
        assert_eq!(fmt_phase_time(&gm), "Monday    11:16 UTC 13 March 1995");

        gm.day = 11; // Saturday
        assert_eq!(fmt_phase_time(&gm), "Saturday  11:16 UTC 11 March 1995");

        gm.month = 1; // January
        assert_eq!(fmt_phase_time(&gm), "Wednesday 11:16 UTC 11 January 1995");

        gm.month = 12; // December
        assert_eq!(fmt_phase_time(&gm), "Monday    11:16 UTC 11 December 1995");
    }

    #[test]
    fn jtime_regular() {
        let jd = jtime(&UTCDateTime::from_ymdhms(1995, 3, 11, 1, 40, 0));

        assert_almost_eq!(jd, 2_449_787.569_444_444_5);
    }

    #[test]
    fn jtime_january() {
        let jd = jtime(&UTCDateTime::from_ymdhms(1995, 1, 1, 0, 0, 0));

        assert_almost_eq!(jd, 2_449_718.5);
    }

    #[test]
    fn jtime_zero() {
        let jd = jtime(&UTCDateTime::from_ymdhms(-4712, 1, 1, 12, 0, 0));

        assert_almost_eq!(jd, 0.0);
    }

    #[test]
    fn jtime_negative() {
        let jd = jtime(&UTCDateTime::from_ymdhms(-8000, 1, 1, 0, 0, 0));

        assert_almost_eq!(jd, -1_200_941.5);
    }

    #[test]
    fn ucttoj_regular() {
        let julian_date = ucttoj(1995, 2, 11, 0, 0, 0);

        assert_almost_eq!(julian_date, 2_449_787.5);
    }

    #[test]
    fn ucttoj_month_lte_2() {
        let julian_date = ucttoj(1900, 1, 1, 0, 0, 0);

        assert_almost_eq!(julian_date, 2_415_051.5);
    }

    #[test]
    fn ucttoj_year_1582() {
        let julian_date = ucttoj(1582, 9, 4, 0, 0, 0);

        assert_almost_eq!(julian_date, 2_299_159.5);
    }

    #[test]
    fn jtouct_regular() {
        let gm = jtouct(2_438_749.732_639);

        assert_eq!(gm, UTCDateTime::from_ymdhms(1964, 12, 20, 5, 35, 0));
    }

    #[test]
    fn jyear_regular() {
        let ymd = jyear(2_460_426.091_91);

        assert_eq!(ymd, (2024, 4, 25));
    }

    #[test]
    fn jyear_before_october_15_1582() {
        let ymd = jyear(2_299_160.0);

        assert_eq!(ymd, (1582, 10, 4));
    }

    #[test]
    fn jyear_on_october_15_1582() {
        let ymd = jyear(2_299_160.9);

        assert_eq!(ymd, (1582, 10, 15));
    }

    #[test]
    fn jhms_regular() {
        let hms = jhms(2_438_749.732_639); // P

        assert_eq!(hms, (5, 35, 0));
    }

    #[test]
    fn jhms_zero() {
        let hms = jhms(0.0);

        assert_eq!(hms, (12, 0, 0));
    }

    #[test]
    fn jhms_negative() {
        let hms = jhms(-1_200_941.5);

        assert_eq!(hms, (0, 0, 0));
    }

    #[test]
    fn jwday_regular() {
        let wday = jwday(2_439_913.881_944); // M

        assert_eq!(wday, 2);
    }

    #[test]
    fn jwday_positive_all_days() {
        assert_eq!(jwday(2_439_912.0), 0); // Sunday
        assert_eq!(jwday(2_439_913.0), 1);
        assert_eq!(jwday(2_439_914.0), 2);
        assert_eq!(jwday(2_439_915.0), 3);
        assert_eq!(jwday(2_439_916.0), 4);
        assert_eq!(jwday(2_439_917.0), 5);
        assert_eq!(jwday(2_439_918.0), 6);
        assert_eq!(jwday(2_439_919.0), 0);
    }

    #[test]
    fn meanphase_regular() {
        let meanph = meanphase(2_460_381.612_639, 1535.0);

        assert_almost_eq!(meanph, 2_460_350.212_978_046_4);
    }

    #[test]
    fn truephase_lt_0_01() {
        let trueph = truephase(1537.0, 0.0);

        assert_almost_eq!(trueph, 2_460_409.266_218_814);
    }

    #[test]
    fn truephase_abs_min_0_25_lt_0_01_and_lt_0_5() {
        let trueph = truephase(1537.0, 0.25);

        assert_almost_eq!(trueph, 2_460_416.301_725_250_7);
    }

    #[test]
    fn truephase_abs_min_0_75_lt_0_01_and_gte_0_5() {
        let trueph = truephase(1537.0, 0.75);

        assert_almost_eq!(trueph, 2_460_431.977_685_604_2);
    }

    #[test]
    #[should_panic(expected = "TRUEPHASE called with invalid phase selector.")]
    fn truephase_invalid_phase_selector() {
        let _ = truephase(1537.0, 1.0);
    }

    #[test]
    fn phasehunt_regular() {
        let phasar = phasehunt(2_449_818.3);

        assert_eq!(
            phasar,
            (
                2_449_807.590_823_359_3,
                2_449_815.732_797_042_5,
                2_449_823.006_760_471,
                2_449_829.638_518_093_6,
                2_449_837.234_842_154_7,
            ),
        );
    }

    #[test]
    fn kepler_regular() {
        let ec = kepler(111.615_376, 0.0167_18);

        assert_almost_eq!(ec, 1.963_501_188_099_530_1);
    }

    #[test]
    fn phase_regular() {
        let phase_info = phase(2_449_818.7);

        assert_eq!(
            phase_info,
            PhaseInfo {
                phase: 0.344_887_879_941_135_07,
                fraction_illuminated: 0.780_750_292_028_882_7,
                age: 10.184_742_123_258_882,
                ecliptic_longitude: 145.072_067_326_933_1,
                ecliptic_latitude: -6.394_104_031_972_319,
                parallax: 0.939_266_914_937_796_7,
                distance: 389_080.063_279_139_4,
                angular_diameter: 0.511_869_347_459_001_3,
                sun_ecliptic_longitude: 20.842_047_954_970_667,
                sun_distance: 149_916_135.218_393_74,
                sun_angular_diameter: 0.531_998_433_602_993_3,
            }
        );
    }

    #[test]
    fn tmp_suncal_june_solstice() {
        // Example values from Jean Meeus' Astronomical Algorithms book.
        let jd = solarevent(1962, SolarEvent::JuneSolstice);

        assert_almost_eq!(jd, 2_437_837.392_448_240_4);
        assert_eq!(
            UTCDateTime::from_julian_date(jd).to_string(),
            "1962-06-21T21:25:08Z"
        );
    }
}
