//! Command Line Interface for moon.rs.

use moontool::moon::{
    ForDateTime, LocalDateTime, MoonCalendar, MoonPhase, SunCalendar, ToJSON, UTCDateTime,
    YearlyMoonCalendar,
};
use std::fmt::Write;
use std::{env, process};
use textcanvas::{charts::Plot, Color, TextCanvas};

mod moon_icon;

const GRAPH_WIDTH: i32 = 80;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Eq, PartialEq)]
struct Config {
    datetime: Option<String>,
    help: bool,
    version: bool,
    verbose: bool,
    moon: bool,
    graph: bool,
    json: bool,
}

// Prefer explicit default config.
#[allow(clippy::derivable_impls)]
impl Default for Config {
    fn default() -> Self {
        Self {
            datetime: None,
            help: false,
            version: false,
            verbose: false,
            moon: false,
            graph: false,
            json: false,
        }
    }
}

impl Config {
    fn new(args: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut config = Self::default();

        for arg in args.skip(1) {
            if arg == "-h" || arg == "--help" {
                config.help = true;
                break;
            }

            if arg == "-v" || arg == "--version" {
                config.version = true;
                break;
            }

            if arg == "-vv" || arg == "--verbose" {
                config.verbose = true;
                continue;
            }

            if arg == "--moon" {
                config.moon = true;
                continue;
            }

            if arg == "--graph" {
                config.graph = true;
                continue;
            }

            if arg == "--json" {
                config.json = true;
                continue;
            }

            // `-` can be the start of a negative timestamp.
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

    for_datetime(&datetime, &config);
}

fn help_message() -> String {
    format!(
        "\
usage: {bin} [-h] [] [DATETIME] [±TIMESTAMP]

optional arguments:
  -h, --help            show this help message and exit
  -v, --version         show the version and exit
  -vv, --verbose        verbose output
  --moon                show render of Moon
  --graph               graph of lunation
  --json                output as json
  []                    without arguments, defaults to now
  [DATETIME]            local datetime (e.g., 1994-12-22T14:53:34+01:00)
  [±TIMESTAMP]          Unix timestamp (e.g., 788104414)
  [JULIAN DATE]         Julian date (e.g., 2449709.07887)",
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
    } else if let Some(datetime) = try_from_julian_date(datetime) {
        Some(datetime)
    } else {
        try_from_iso_string(datetime)
    }
}

fn try_from_timestamp(timestamp: &str) -> Option<UTCDateTime> {
    let Ok(timestamp) = timestamp.parse::<i64>() else {
        return None;
    };
    UTCDateTime::from_timestamp(timestamp).ok()
}

fn try_from_julian_date(julian_date: &str) -> Option<UTCDateTime> {
    let Ok(jd) = julian_date.parse::<f64>() else {
        return None;
    };
    Some(UTCDateTime::from_julian_date(jd))
}

fn try_from_iso_string(datetime: &str) -> Option<UTCDateTime> {
    UTCDateTime::from_iso_string(datetime).ok()
}

#[cfg(not(tarpaulin_include))]
fn for_datetime(datetime: &UTCDateTime, config: &Config) {
    let mphase = MoonPhase::for_datetime(datetime);

    if config.moon {
        draw_moon(mphase.fraction_of_lunation, &mphase.utc_datetime);
        return;
    }

    let mcal = MoonCalendar::for_datetime(datetime);

    if config.graph {
        graph_moon_data(&mcal, config.verbose);
        return;
    }

    let ymcal = if config.verbose {
        Some(YearlyMoonCalendar::for_datetime(datetime))
    } else {
        None
    };

    let scal = if config.verbose {
        Some(SunCalendar::for_datetime(datetime))
    } else {
        None
    };

    if config.json {
        print_json(&mphase, &mcal, &ymcal, &scal);
        return;
    }

    print_pretty(&mphase, &mcal, &ymcal, &scal);
}

#[cfg(not(tarpaulin_include))]
fn draw_moon(ph: f64, date: &UTCDateTime) {
    print!("{}", render_moon(ph, date));
}

/// Construct icon for moon, given phase of moon.
///
/// Adapted from `moontool/moontool.c`'s `drawmoon()` function.
///
/// This uses a scanning technique, where for each `Y` coordinate, it
/// calculates the bounds of the visible portion of the Moon: `LX` and
/// `RX` (`left-X`, and `right-X`). It starts at the center of the Moon
/// (`i = 0`) and goes down to the bottom portion (`i = RADIUS`). This
/// is then reflected vertically to get the top portion, thus covering
/// the entire Moon (bottom: `center + i`, top: `center + i`). Then for
/// each `(Y, LX, RX)`, it samples the corresponding bounded line on the
/// source pixmap (Full Moon image), and blits it onto the destination
/// pixmap (render). The portions outside `[LX;RX]` are not blitted, and
/// this is what creates the shadow.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::manual_range_contains
)]
fn render_moon(ph: f64, date: &UTCDateTime) -> String {
    let mut canvas = TextCanvas::new_auto().unwrap_or_default();
    let offset_x = canvas.ucx() - (moon_icon::WIDTH / 2);
    let offset_y = canvas.ucy() - (moon_icon::HEIGHT / 2);

    // Allow the moon to be completely dark for a few hours when new.
    if ph < 0.01 || ph > 0.99 {
        return canvas.to_string();
    }

    // Fractional width of the visible portion.
    //
    // |-------|------|------|------|------|------|
    // | phase |   0  | 0.25 |  0.5 | 0.75 |   1  |
    // | rad   |   0  |  π/2 |   π  | 1.5π |  2π  |
    // | cos() |   1  |   0  |  -1  |   0  |   1  |
    //
    // This value scales the radius (`cp`).
    //
    // - Phase < 0.5 (Moon growing from the right).
    // - Phase >= 0.5 (Moon shrinking to the left).
    //
    // | Phase | lx                         | rx                         |
    // |-------|----------------------------|----------------------------|
    // |  <0.5 | center + ([-1;1] * radius) | center + radius            |
    // | >=0.5 | center - radius            | center - ([-1;1] * radius) |
    //
    // At the start, `lx` = _center + (1 * radius)_, which is equal to
    // `rx` = _center + radius_:
    //
    // ```txt
    // Phase = 0, xscale = 1
    //
    // -radius       center      +radius
    //     |            o           ||
    //                            lx rx
    // ```
    //
    // When phase reaches 0.25, `lx` = _center + (0 * radius)_, so
    // _`lx` = center_:
    //
    //
    // ```txt
    // Phase = 0.25, xscale = 0
    //
    // -radius       center      +radius
    //     |            o------------|
    //                  lx           rx
    // ```
    //
    // With phase >= 0.5 `lx` and `rx` are inverted.
    // `lx` = _center - radius_, and `rx` = _center - (-1 * radius)_
    // <=> _center + radius_:
    //
    // ```txt
    // Phase = 0.5, xscale = -1
    //
    // -radius       center      +radius
    //     |------------o------------|
    //    lx                        rx
    // ```
    //
    // This time, with `xscale` = _0_, it's `rx` that's in the center:
    //
    // ```txt
    // Phase = 0.75, xscale = 0
    //
    // -radius       center      +radius
    //     |------------o            |
    //    lx           rx
    // ```
    //
    // Finally, and once again, _`lx` = `rx`_, but on the other side:
    //
    // ```txt
    // Phase = 1, xscale = 1
    //
    // -radius       center      +radius
    //     ||           o            |
    //    lx rx
    // ```
    let xscale: f64 = (2.0 * std::f64::consts::PI * ph).cos();

    for i in 0..moon_icon::IRADIUS {
        // Radius, but tapered towards the extremities.
        //
        // 100% width in the middle, 0% width at the bottom or top.
        //
        // _r * cos(arcsin(x/r))_ describes the upper-right quarter of
        // a circle, where _f(0) = r_ and _f(r) = 0_.
        //
        // Since we're solving for the bottom portion here, we have
        // maximum width at the center (_i = 0_, so _f(0) = radius)_,
        // and minimum width once we reach the bottom (_i = radius_, so
        // _f(radius) = 0_).
        let cp: f64 = moon_icon::RADIUS * (i as f64 / moon_icon::RADIUS).asin().cos();

        let rx: usize;
        let lx: usize;
        if ph < 0.5 {
            // /!\ `f64`, because `usize` can't handle negative values.
            rx = (moon_icon::CENTER as f64 + cp.trunc()) as usize;
            lx = (moon_icon::CENTER as f64 + (xscale * cp).trunc()) as usize;
        } else {
            lx = (moon_icon::CENTER as f64 - cp.trunc()) as usize;
            rx = (moon_icon::CENTER as f64 - (xscale * cp).trunc()) as usize;
        }

        // We now know the left and right endpoints of the scan line for
        // this y coordinate. We blit the corresponding scanlines from
        // the source pixrect to the destination pixrect, offsetting to
        // properly place it in the pixrect and reflecting vertically.

        // Bottom portion.
        blit_line(
            &moon_icon::MOON,
            &mut canvas,
            lx,
            moon_icon::OFFSET + i,
            (rx - lx) + 1,
            offset_x,
            offset_y,
        );
        // Top portion (but don't do center line twice).
        if i != 0 {
            blit_line(
                &moon_icon::MOON,
                &mut canvas,
                lx,
                moon_icon::OFFSET - i,
                (rx - lx) + 1,
                offset_x,
                offset_y,
            );
        }
    }

    // If it's July 20th (in local time if we're running in real time,
    // otherwise based on UTC), display the Apollo 11 Commemorative
    // Red Dot at Tranquility Base. Otherwise, just show the regular
    // mare floor.
    let (month, day) = LocalDateTime::try_from(date).map_or_else(
        |_| (date.month, date.day), // Fall back to UTC.
        |local| (local.month, local.day),
    );
    if month == 7 && day == 20 {
        draw_apollo_11_commemorative_dot(&mut canvas, offset_x, offset_y);
    }

    canvas.to_string()
}

#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
fn blit_line(
    source: &[u8; 4096],
    dest: &mut TextCanvas,
    x: usize,
    y: usize,
    width: usize,
    doffset_x: usize,
    doffset_y: usize,
) {
    // Source (X, Y, Width) = Destination (X, Y, Width)
    for x in x..x + width {
        let color = source[y * moon_icon::WIDTH + x];
        if color <= moon_icon::MONOCHROME_THRESHOLD {
            continue;
        }
        dest.set_pixel((x + doffset_x) as i32, (y + doffset_y) as i32, true);
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
fn draw_apollo_11_commemorative_dot(canvas: &mut TextCanvas, offset_x: usize, offset_y: usize) {
    canvas.set_color(Color::new().bright_red());

    let x = (moon_icon::APOLLO_11.0 + offset_x) as i32;
    let y = (moon_icon::APOLLO_11.1 + offset_y) as i32;

    // Clean up neighboring pixels to make dot look clean: ⢻ -> ⠛
    canvas.set_pixel(x, y + 1, false);
    canvas.set_pixel(x, y + 2, false);

    canvas.set_pixel(x - 1, y - 1, true);
    canvas.set_pixel(x, y - 1, true);
    canvas.set_pixel(x - 1, y, true);
    canvas.set_pixel(x, y, true);
}

#[cfg(not(tarpaulin_include))]
fn graph_moon_data(mcal: &MoonCalendar, verbose: bool) {
    print!("{}", render_moon_graphs(mcal, verbose));
}

fn render_moon_graphs(mcal: &MoonCalendar, verbose: bool) -> String {
    let date = &mcal.utc_datetime;

    let mut output = String::new();

    writeln!(output, "\n{}", graph_lunation_for_month(mcal)).unwrap();

    let (x, y_phase) = pre_compute_yearly_graph_data(date);

    macro_rules! graph_data_for_year {
        ($label:literal, $field:ident) => {
            let y: Vec<f64> = y_phase.iter().map(|phase| phase.$field).collect();
            let data = graph_data_for_year(&x, &y, date);
            writeln!(output, "{} {}\n{data}", $label, date.year).unwrap();
        };
    }

    graph_data_for_year!("Moon phases", fraction_illuminated);

    if verbose {
        // Moon.
        graph_data_for_year!("Moon distance to Earth", distance_to_earth_km);
        graph_data_for_year!("Moon subtends", subtends);
        graph_data_for_year!("Lunations", fraction_of_lunation);
        graph_data_for_year!("Moon ecliptic longitude", ecliptic_longitude);
        graph_data_for_year!("Moon ecliptic latitude", ecliptic_latitude);
        graph_data_for_year!("Moon parallax", parallax);

        // Sun.
        graph_data_for_year!("Sun distance to Earth", sun_distance_to_earth_km);
        graph_data_for_year!("Sun subtends", sun_subtends);
        graph_data_for_year!("Sun ecliptic longitude", sun_ecliptic_longitude);
    }

    output
}

fn graph_lunation_for_month(mcal: &MoonCalendar) -> String {
    let f = |jd: f64| {
        let phase = MoonPhase::for_julian_date(jd);
        phase.fraction_illuminated
    };

    let mut canvas = TextCanvas::new(GRAPH_WIDTH, 13);

    let start = mcal.last_new_moon;
    let end = mcal.next_new_moon;
    let (x, y) = Plot::compute_function(start, end, canvas.screen.fwidth(), &f);

    Plot::line(&mut canvas, &x, &y);

    let date = mcal.utc_datetime.to_julian_date();
    canvas.set_color(Color::new().bright_red());
    Plot::stroke_line_at_x(&mut canvas, date, &x);

    format!("{canvas}🌑      🌒         🌓         🌔         🌕        🌖       🌗        🌘      🌑\n")
}

/// Pre-compute all yearly values at once.
///
/// This avoids lots of overhead, because otherwise we would need to
/// compute the exact same phase values twice per graph (once for the
/// curve, and once for the vertical current-date line).
///
/// On a typical 80 chars output, we would compute 3200 phases in
/// verbose mode (10 graphs * 2 times per graph * 160 pixels).
///
/// With this method, we only compute 160 phases (once for each of the
/// 160 pixels, as there are two horizontal pixels per output char).
fn pre_compute_yearly_graph_data(date: &UTCDateTime) -> (Vec<f64>, Vec<MoonPhase>) {
    let f = |jd: f64| MoonPhase::for_julian_date(jd);

    let start = UTCDateTime::from_ymdhms(date.year, 1, 1, 0, 0, 0).to_julian_date();
    let end = UTCDateTime::from_ymdhms(date.year, 12, 31, 23, 59, 59).to_julian_date();

    Plot::compute_function(start, end, f64::from(GRAPH_WIDTH * 2), &f)
}

fn graph_data_for_year(x: &[f64], y: &[f64], date: &UTCDateTime) -> String {
    let mut canvas = TextCanvas::new(GRAPH_WIDTH, 4);

    Plot::line(&mut canvas, x, y);

    canvas.set_color(Color::new().bright_red());
    Plot::stroke_line_at_x(&mut canvas, date.to_julian_date(), x);

    format!("{canvas}")
}

#[cfg(not(tarpaulin_include))]
fn print_json(
    mphase: &MoonPhase,
    mcal: &MoonCalendar,
    ymcal: &Option<YearlyMoonCalendar>,
    scal: &Option<SunCalendar>,
) {
    let mphase = mphase.to_json();
    let mcal = mcal.to_json();

    print!(r#"{{"phase":{mphase},"calendar":{mcal}"#);

    if let Some(ymcal) = ymcal {
        let ymcal = ymcal.to_json();
        print!(r#","yearly_calendar":{ymcal}"#);
    }

    if let Some(scal) = scal {
        let scal = scal.to_json();
        print!(r#","sun_calendar":{scal}"#);
    }

    println!(r#"}}"#);
}

#[cfg(not(tarpaulin_include))]
fn print_pretty(
    mphase: &MoonPhase,
    mcal: &MoonCalendar,
    ymcal: &Option<YearlyMoonCalendar>,
    scal: &Option<SunCalendar>,
) {
    println!("\n{mphase}\n");
    println!("{mcal}\n");

    if let Some(scal) = scal {
        println!("{scal}\n");
    }

    if let Some(ymcal) = ymcal {
        println!("{ymcal}\n");
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
                verbose: false,
                moon: false,
                graph: false,
                json: false,
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
                verbose: false,
                moon: false,
                graph: false,
                json: false,
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
        assert!(message.contains("-vv, --verbose"));
        assert!(message.contains("--moon"));
        assert!(message.contains("--graph"));
        assert!(message.contains("--json"));
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
    fn verbose_full() {
        let args = vec![String::new(), String::from("--verbose")].into_iter();
        let config = Config::new(args).unwrap();

        assert!(config.verbose);
    }

    #[test]
    fn verbose_short() {
        let args = vec![String::new(), String::from("-vv")].into_iter();
        let config = Config::new(args).unwrap();

        assert!(config.verbose);
    }

    #[test]
    fn moon() {
        let args = vec![String::new(), String::from("--moon")].into_iter();
        let config = Config::new(args).unwrap();

        assert!(config.moon);
    }

    #[test]
    fn moon_waxing() {
        let mphase = MoonPhase::for_ymdhms(2024, 5, 17, 17, 48, 19);

        assert_eq!(
            render_moon(mphase.fraction_of_lunation, &mphase.utc_datetime),
            "\
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠠⠀⠀⠤⢀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠠⠴⢭⠭⡖⢔⠦⣬⣝⣦⣄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⢿⣁⣷⣿⠙⢾⡽⣻⢦⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡀⠀⠀⠀⢘⡗⠁⠈⠘⠙⠋⠀⠯⣳⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠰⣀⠂⠀⣀⣠⢉⠄⠀⠀⠀⠑⠀⢳⣤⢵⡻⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡆⢘⣕⠂⠂⠀⠈⠈⡶⠢⠀⠀⠀⠸⠎⠉⠗⣷⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⢛⡏⠥⡀⠘⠲⢤⣼⡕⠀⠀⢀⣄⠄⠀⠩⠔⣹⡆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠀⠻⢠⣼⠷⡎⢩⠉⠔⢠⠀⠾⠿⡅⠀⠶⠃⢺⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠁⠀⠉⣴⠡⡩⡩⢡⣿⡒⣄⣀⣼⣷⣴⢄⢥⣿⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡂⠤⡼⢁⠽⣉⣴⣟⢻⣮⣾⡂⡷⢖⣭⣿⣿⠏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠀⠃⠁⢖⣡⣶⡿⢯⠽⣖⣖⡄⡮⣓⣿⠏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡀⠀⢒⢶⣶⣿⡿⣿⣿⢿⠍⣈⡿⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠐⠲⠸⢞⠶⡤⣿⢿⣋⣟⣧⠞⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠠⠌⠁⠩⠬⠙⠋⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
",
        );
    }

    #[test]
    fn moon_waning() {
        let mphase = MoonPhase::for_ymdhms(2024, 5, 29, 17, 48, 19);

        assert_eq!(
            render_moon(mphase.fraction_of_lunation, &mphase.utc_datetime),
            "\
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⡠⠤⠀⠠⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⠶⡥⠤⠖⠢⠴⢭⠭⡖⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠔⠓⢱⣎⣵⠚⠀⠀⠀⠀⠉⢿⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⠊⠁⠀⣀⠈⠁⠀⠀⡀⠀⠀⠀⢘⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⡋⠀⡄⠁⠀⡢⣀⠰⣀⠂⠀⣀⣠⢉⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣮⣥⠀⠀⢰⣁⣱⢈⡆⢘⣕⠂⠂⠀⠈⠈⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣇⢹⣇⢀⠀⠀⢉⢤⢛⡏⠥⡀⠘⠲⢤⣼⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⣿⢿⣳⣮⣀⡀⠳⠌⠀⠻⢠⣼⠷⡎⢩⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣟⡿⠟⣏⠀⠉⢐⠥⠁⠀⠉⣴⠡⡩⡩⢡⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠸⣏⢘⢸⣆⡀⠂⡨⡂⠤⡼⢁⠽⣉⣴⣟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠹⡮⡍⣙⠧⣑⡀⠊⠀⠃⠁⢖⣡⣶⡿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⢮⡛⠭⣽⣥⠶⠀⡀⠀⢒⢶⣶⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠻⣍⡂⡴⡷⠲⠸⢞⠶⡤⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠒⠳⠤⠤⠌⠁⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
",
        );
    }

    #[test]
    fn moon_new() {
        let mphase = MoonPhase::for_ymdhms(2024, 6, 6, 17, 5, 0);

        let render = render_moon(mphase.fraction_of_lunation, &mphase.utc_datetime);

        assert!(render.trim_matches(&['\n', '⠀']).is_empty());
    }

    #[test]
    fn moon_apollo_11() {
        let mphase = MoonPhase::for_ymdhms(1969, 7, 20, 20, 17, 40);

        let render = render_moon(mphase.fraction_of_lunation, &mphase.utc_datetime);

        assert!(render.contains("\x1b[0;91m⠛\x1b[0m"));
    }

    #[test]
    fn graph() {
        let args = vec![String::new(), String::from("--graph")].into_iter();
        let config = Config::new(args).unwrap();

        assert!(config.graph);
    }

    #[test]
    fn graph_with_datetime_after() {
        let args = vec![
            String::new(),
            String::from("--graph"),
            String::from("2024-05-15"),
        ]
        .into_iter();
        let config = Config::new(args).unwrap();

        assert!(config.graph);
        assert!(config.datetime.is_some());
    }

    #[test]
    fn graph_with_datetime_before() {
        let args = vec![
            String::new(),
            String::from("2024-05-15"),
            String::from("--graph"),
        ]
        .into_iter();
        let config = Config::new(args).unwrap();

        assert!(config.graph);
        assert!(config.datetime.is_some());
    }

    #[test]
    fn graph_regular() {
        let mcal = MoonCalendar::for_julian_date(2_460_472.289_13);

        let render = render_moon_graphs(&mcal, false);

        assert!(render.contains("\x1b[0;91m"));
        assert!(render.contains("\x1b[0m"));

        let render = render.replace("\x1b[0;91m", "").replace("\x1b[0m", "");

        assert_eq!(
            render,
            "
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡠⠔⠒⠉⠉⠉⠉⠉⠉⠉⠒⠢⢄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⠔⠊⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠑⢄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠢⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡠⠒⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠢⢄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠤⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠣⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠢⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⡠⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠢⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⡠⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠢⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⢀⠔⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⢆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⡠⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠢⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⢺⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠢⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⣀⠔⠊⠁⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠑⠢⣀⠀⠀⠀⠀⠀⠀⠀
⠤⠤⠤⠒⠊⠉⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠒⠢⠤⠤⠤⣀
🌑      🌒         🌓         🌔         🌕        🌖       🌗        🌘      🌑

Moon phases 2024
⡀⠀⠀⠀⡜⠑⡄⠀⠀⠀⢠⠋⢢⠀⠀⠀⢀⠎⠑⡄⠀⠀⠀⡰⠉⢆⠀⠀⠀⢀⠎⢱⠀⠀⠀⡇⡰⠙⡄⠀⠀⠀⢠⠋⢆⠀⠀⠀⠀⡜⠱⡀⠀⠀⠀⢠⠋⡆⠀⠀⠀⢀⠎⢣⠀⠀⠀⠀⡰⠑⡄⠀⠀⠀⢠⠋⢆⠀⠀⠀
⡇⠀⠀⢸⠀⠀⢱⠀⠀⢀⠇⠀⠈⡆⠀⠀⡜⠀⠀⢱⠀⠀⢠⠃⠀⠘⡄⠀⠀⡜⠀⠀⢇⠀⠀⣷⠁⠀⢱⠀⠀⠀⡎⠀⠘⡄⠀⠀⢰⠁⠀⢇⠀⠀⠀⡇⠀⢸⠀⠀⠀⡸⠀⠈⡆⠀⠀⢠⠃⠀⢱⠀⠀⠀⡎⠀⠘⡄⠀⠀
⠸⡀⢀⠇⠀⠀⠀⢇⠀⡸⠀⠀⠀⢸⠀⢠⠃⠀⠀⠈⡆⠀⡎⠀⠀⠀⢣⠀⢰⠁⠀⠀⠸⡀⠀⡏⠀⠀⠀⡇⠀⡸⠀⠀⠀⢣⠀⢀⠇⠀⠀⠸⡀⠀⡸⠀⠀⠀⢇⠀⢠⠃⠀⠀⠸⡀⠀⡜⠀⠀⠈⡆⠀⢰⠁⠀⠀⢱⠀⠀
⠀⠱⠜⠀⠀⠀⠀⠈⠦⠃⠀⠀⠀⠀⠣⠎⠀⠀⠀⠀⠱⠜⠀⠀⠀⠀⠈⠦⠃⠀⠀⠀⠀⠱⠜⡇⠀⠀⠀⠘⠤⠃⠀⠀⠀⠈⢆⠜⠀⠀⠀⠀⠱⠤⠃⠀⠀⠀⠈⠦⠎⠀⠀⠀⠀⠱⠔⠁⠀⠀⠀⠘⠤⠃⠀⠀⠀⠀⠣⠤

"
        );
    }

    #[test]
    fn graph_verbose() {
        let mcal = MoonCalendar::for_julian_date(2_460_472.289_13);

        let render = render_moon_graphs(&mcal, true);

        assert!(render.contains("\x1b[0;91m"));
        assert!(render.contains("\x1b[0m"));

        let render = render.replace("\x1b[0;91m", "").replace("\x1b[0m", "");

        println!("{render}");
        assert_eq!(
            render,
            "
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡠⠔⠒⠉⠉⠉⠉⠉⠉⠉⠒⠢⢄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⠔⠊⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠑⢄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠢⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡠⠒⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠢⢄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠤⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠣⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠢⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⡠⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠢⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⡠⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠢⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⢀⠔⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⢆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⡠⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠢⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⢺⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠢⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⣀⠔⠊⠁⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠑⠢⣀⠀⠀⠀⠀⠀⠀⠀
⠤⠤⠤⠒⠊⠉⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠒⠢⠤⠤⠤⣀
🌑      🌒         🌓         🌔         🌕        🌖       🌗        🌘      🌑

Moon phases 2024
⡀⠀⠀⠀⡜⠑⡄⠀⠀⠀⢠⠋⢢⠀⠀⠀⢀⠎⠑⡄⠀⠀⠀⡰⠉⢆⠀⠀⠀⢀⠎⢱⠀⠀⠀⡇⡰⠙⡄⠀⠀⠀⢠⠋⢆⠀⠀⠀⠀⡜⠱⡀⠀⠀⠀⢠⠋⡆⠀⠀⠀⢀⠎⢣⠀⠀⠀⠀⡰⠑⡄⠀⠀⠀⢠⠋⢆⠀⠀⠀
⡇⠀⠀⢸⠀⠀⢱⠀⠀⢀⠇⠀⠈⡆⠀⠀⡜⠀⠀⢱⠀⠀⢠⠃⠀⠘⡄⠀⠀⡜⠀⠀⢇⠀⠀⣷⠁⠀⢱⠀⠀⠀⡎⠀⠘⡄⠀⠀⢰⠁⠀⢇⠀⠀⠀⡇⠀⢸⠀⠀⠀⡸⠀⠈⡆⠀⠀⢠⠃⠀⢱⠀⠀⠀⡎⠀⠘⡄⠀⠀
⠸⡀⢀⠇⠀⠀⠀⢇⠀⡸⠀⠀⠀⢸⠀⢠⠃⠀⠀⠈⡆⠀⡎⠀⠀⠀⢣⠀⢰⠁⠀⠀⠸⡀⠀⡏⠀⠀⠀⡇⠀⡸⠀⠀⠀⢣⠀⢀⠇⠀⠀⠸⡀⠀⡸⠀⠀⠀⢇⠀⢠⠃⠀⠀⠸⡀⠀⡜⠀⠀⠈⡆⠀⢰⠁⠀⠀⢱⠀⠀
⠀⠱⠜⠀⠀⠀⠀⠈⠦⠃⠀⠀⠀⠀⠣⠎⠀⠀⠀⠀⠱⠜⠀⠀⠀⠀⠈⠦⠃⠀⠀⠀⠀⠱⠜⡇⠀⠀⠀⠘⠤⠃⠀⠀⠀⠈⢆⠜⠀⠀⠀⠀⠱⠤⠃⠀⠀⠀⠈⠦⠎⠀⠀⠀⠀⠱⠔⠁⠀⠀⠀⠘⠤⠃⠀⠀⠀⠀⠣⠤

Moon distance to Earth 2024
⢣⠀⠀⠀⢀⠎⢱⠀⠀⠀⢀⠎⢣⠀⠀⠀⢠⠋⢣⠀⠀⠀⢠⠋⢣⠀⠀⠀⢠⠋⢣⠀⠀⠀⢠⡏⢢⠀⠀⠀⢠⠋⢢⠀⠀⠀⢠⠋⢢⠀⠀⠀⢠⠋⢢⠀⠀⠀⢠⠋⢢⠀⠀⠀⢠⠋⢆⠀⠀⠀⢠⠋⢆⠀⠀⠀⢠⠋⢆⠀
⠈⡆⠀⠀⡜⠀⠀⡇⠀⠀⡜⠀⠈⡆⠀⠀⡜⠀⠈⡆⠀⠀⡜⠀⠈⡆⠀⠀⡜⠀⠈⡆⠀⠀⡜⡇⠈⡆⠀⠀⡎⠀⠈⡆⠀⠀⡎⠀⠈⡆⠀⠀⡎⠀⠈⡆⠀⠀⡎⠀⠈⡆⠀⠀⡎⠀⠘⡄⠀⠀⡎⠀⠘⡄⠀⠀⡎⠀⠘⡄
⠀⢸⠀⢠⠃⠀⠀⢸⠀⢠⠃⠀⠀⢸⠀⢠⠃⠀⠀⢸⠀⢠⠃⠀⠀⢱⠀⢠⠃⠀⠀⢱⠀⢰⠁⡇⠀⢱⠀⢰⠁⠀⠀⢱⠀⢰⠁⠀⠀⢱⠀⢰⠁⠀⠀⢱⠀⢰⠁⠀⠀⢱⠀⢰⠁⠀⠀⢱⠀⢰⠁⠀⠀⢣⠀⢰⠁⠀⠀⠉
⠀⠀⠣⠎⠀⠀⠀⠀⠣⠎⠀⠀⠀⠀⠣⠎⠀⠀⠀⠀⠣⠎⠀⠀⠀⠀⠣⠎⠀⠀⠀⠀⢣⠇⠀⡇⠀⠈⠦⠃⠀⠀⠀⠈⠦⠃⠀⠀⠀⠈⠦⠃⠀⠀⠀⠈⠦⠃⠀⠀⠀⠈⠦⠃⠀⠀⠀⠈⠦⠃⠀⠀⠀⠈⠦⠃⠀⠀⠀⠀

Moon subtends 2024
⠀⢀⠎⢆⠀⠀⠀⢀⠎⢆⠀⠀⠀⢀⠎⢆⠀⠀⠀⢀⠎⢆⠀⠀⠀⢀⠎⢆⠀⠀⠀⢀⠎⢆⠀⡇⠀⢀⠎⢆⠀⠀⠀⢠⠋⢆⠀⠀⠀⢠⠋⢆⠀⠀⠀⢠⠋⢆⠀⠀⠀⢠⠋⢆⠀⠀⠀⢠⠋⢆⠀⠀⠀⢠⠋⢆⠀⠀⠀⠀
⠀⡸⠀⠘⡄⠀⠀⡸⠀⠘⡄⠀⠀⡸⠀⠘⡄⠀⠀⡸⠀⠘⡄⠀⠀⡸⠀⠘⡄⠀⠀⡸⠀⠘⡄⡇⠀⡜⠀⠘⡄⠀⠀⡜⠀⠘⡄⠀⠀⡜⠀⠸⡀⠀⠀⡜⠀⠸⡀⠀⠀⡜⠀⠸⡀⠀⠀⡜⠀⠸⡀⠀⠀⡜⠀⠸⡀⠀⠀⣀
⢀⠇⠀⠀⢱⠀⢀⠇⠀⠀⢣⠀⢀⠇⠀⠀⢣⠀⢀⠇⠀⠀⢣⠀⢠⠃⠀⠀⢣⠀⢠⠃⠀⠀⢣⡇⢠⠃⠀⠀⢣⠀⢠⠃⠀⠀⢣⠀⢠⠃⠀⠀⢣⠀⢠⠃⠀⠀⢇⠀⢠⠃⠀⠀⢇⠀⢠⠃⠀⠀⢇⠀⢰⠁⠀⠀⢇⠀⢰⠁
⠜⠀⠀⠀⠀⠣⠜⠀⠀⠀⠈⠦⠜⠀⠀⠀⠈⠦⠜⠀⠀⠀⠈⢆⠎⠀⠀⠀⠈⠦⠎⠀⠀⠀⠈⡧⠎⠀⠀⠀⠈⠦⠎⠀⠀⠀⠈⠦⠎⠀⠀⠀⠈⠦⠊⠀⠀⠀⠈⠦⠊⠀⠀⠀⠈⠦⠊⠀⠀⠀⠈⠦⠃⠀⠀⠀⠈⠦⠃⠀

Lunations 2024
⢀⢾⠀⠀⠀⠀⠀⡔⡇⠀⠀⠀⠀⡠⢺⠀⠀⠀⠀⢀⢴⠀⠀⠀⠀⠀⡠⡇⠀⠀⠀⠀⢀⢾⠀⡇⠀⠀⢀⠔⡇⠀⠀⠀⠀⡠⢺⠀⠀⠀⠀⢀⢼⠀⠀⠀⠀⠀⡔⡇⠀⠀⠀⠀⡠⢺⠀⠀⠀⠀⢀⠔⡇⠀⠀⠀⠀⡠⢺⠀
⠃⠸⡀⠀⠀⡠⠊⠀⢇⠀⠀⢀⠔⠁⠸⡀⠀⠀⡠⠃⢸⠀⠀⠀⢠⠊⠀⢇⠀⠀⢀⠔⠁⠸⡀⡇⠀⡠⠊⠀⢇⠀⠀⢀⠜⠀⠸⡀⠀⠀⡰⠁⢸⠀⠀⠀⢠⠊⠀⢇⠀⠀⢀⠔⠁⠸⡀⠀⠀⡠⠃⠀⢇⠀⠀⢀⠜⠀⠸⡀
⠀⠀⡇⡠⠊⠀⠀⠀⢸⢀⠔⠁⠀⠀⠀⡇⡠⠊⠀⠀⠀⡇⢀⠔⠁⠀⠀⢸⠀⡠⠊⠀⠀⠀⡇⣇⠜⠀⠀⠀⢸⠀⡔⠁⠀⠀⠀⡇⡠⠊⠀⠀⠀⡇⢀⠔⠁⠀⠀⢸⠀⡠⠊⠀⠀⠀⡇⢀⠜⠀⠀⠀⢸⠀⡰⠁⠀⠀⠀⡇
⠀⠀⠗⠁⠀⠀⠀⠀⠸⠃⠀⠀⠀⠀⠀⠋⠀⠀⠀⠀⠀⠗⠁⠀⠀⠀⠀⠸⠊⠀⠀⠀⠀⠀⠗⡇⠀⠀⠀⠀⠸⠊⠀⠀⠀⠀⠀⠟⠀⠀⠀⠀⠀⡷⠁⠀⠀⠀⠀⠸⠊⠀⠀⠀⠀⠀⠗⠁⠀⠀⠀⠀⠸⠊⠀⠀⠀⠀⠀⠧

Moon ecliptic longitude 2024
⠀⢀⢼⠀⠀⠀⠀⢀⢼⠀⠀⠀⠀⢀⢾⠀⠀⠀⠀⢀⢾⠀⠀⠀⠀⢠⢺⠀⠀⠀⠀⢠⢺⠀⠀⡇⠀⡠⡆⠀⠀⠀⠀⡠⡇⠀⠀⠀⠀⡠⡇⠀⠀⠀⠀⡠⡇⠀⠀⠀⠀⡰⡇⠀⠀⠀⠀⡰⡇⠀⠀⠀⠀⡔⡇⠀⠀⠀⢀⠤
⡠⠃⠸⡀⠀⠀⡠⠃⠸⡀⠀⠀⡰⠁⠸⡀⠀⢀⠔⠁⠸⡀⠀⢀⠔⠁⠸⡀⠀⢀⠔⠁⠸⡀⠀⣇⠔⠁⡇⠀⠀⢀⠜⠀⢇⠀⠀⢀⠜⠀⢇⠀⠀⢀⠎⠀⢇⠀⠀⡠⠊⠀⢇⠀⠀⡠⠊⠀⢇⠀⠀⡠⠊⠀⢇⠀⠀⡠⠊⠀
⠁⠀⠀⡇⡠⠊⠀⠀⠀⡇⡠⠊⠀⠀⠀⡇⡠⠊⠀⠀⠀⡇⡠⠃⠀⠀⠀⡇⡰⠁⠀⠀⠀⡇⡰⡇⠀⠀⢸⠀⡔⠁⠀⠀⢸⠀⡔⠁⠀⠀⢸⢀⠔⠁⠀⠀⢸⢀⠔⠁⠀⠀⢸⢀⠜⠀⠀⠀⢸⢀⠜⠀⠀⠀⢸⢀⠎⠀⠀⠀
⠀⠀⠀⠗⠁⠀⠀⠀⠀⠗⠁⠀⠀⠀⠀⠟⠀⠀⠀⠀⠀⠟⠀⠀⠀⠀⠀⠟⠀⠀⠀⠀⠀⠋⠀⡇⠀⠀⢸⠎⠀⠀⠀⠀⠸⠊⠀⠀⠀⠀⠸⠊⠀⠀⠀⠀⠸⠃⠀⠀⠀⠀⠸⠃⠀⠀⠀⠀⠸⠃⠀⠀⠀⠀⠸⠃⠀⠀⠀⠀

Moon ecliptic latitude 2024
⡀⠀⠀⠀⡜⢆⠀⠀⠀⠀⡜⢆⠀⠀⠀⢀⠎⡆⠀⠀⠀⢀⠞⡄⠀⠀⠀⢀⠖⡄⠀⠀⠀⢠⠳⡇⠀⠀⠀⢠⠳⡀⠀⠀⠀⢠⢳⠀⠀⠀⠀⡰⢣⠀⠀⠀⠀⡰⢢⠀⠀⠀⠀⡜⢆⠀⠀⠀⠀⡜⢆⠀⠀⠀⠀⡜⡆⠀⠀⠀
⢇⠀⠀⡸⠀⠈⢆⠀⠀⡜⠀⠘⡄⠀⠀⡜⠀⠘⡄⠀⠀⡜⠀⠘⡄⠀⠀⡎⠀⠱⡀⠀⠀⡎⠀⡷⡀⠀⢀⠇⠀⢱⠀⠀⢠⠃⠀⢣⠀⠀⢠⠃⠀⢣⠀⠀⢰⠁⠈⢆⠀⠀⢰⠁⠈⢆⠀⠀⢰⠁⠈⡆⠀⠀⡸⠀⠘⡄⠀⠀
⠈⡆⢠⠃⠀⠀⠘⡄⢰⠁⠀⠀⠘⡄⢰⠁⠀⠀⠸⡀⡸⠀⠀⠀⠱⡀⡸⠀⠀⠀⢱⠀⡜⠀⠀⡇⢣⠀⡜⠀⠀⠀⢣⠀⡎⠀⠀⠀⢇⠀⡎⠀⠀⠈⢆⢀⠇⠀⠀⠈⡆⢀⠇⠀⠀⠘⡄⢀⠇⠀⠀⠘⡄⢠⠃⠀⠀⠸⡀⠀
⠀⠘⠃⠀⠀⠀⠀⠘⠃⠀⠀⠀⠀⠱⠃⠀⠀⠀⠀⠱⠃⠀⠀⠀⠀⠱⠃⠀⠀⠀⠀⠳⠁⠀⠀⡇⠀⠓⠁⠀⠀⠀⠈⠞⠀⠀⠀⠀⠈⠞⠀⠀⠀⠀⠘⡜⠀⠀⠀⠀⠘⠜⠀⠀⠀⠀⠘⠎⠀⠀⠀⠀⠱⠊⠀⠀⠀⠀⠱⠒

Moon parallax 2024
⠀⢀⠎⢆⠀⠀⠀⢀⠎⢆⠀⠀⠀⢀⠎⢆⠀⠀⠀⢀⠎⢆⠀⠀⠀⢀⠎⢆⠀⠀⠀⢀⠎⢆⠀⡇⠀⢀⠎⢆⠀⠀⠀⢠⠋⢆⠀⠀⠀⢠⠋⢆⠀⠀⠀⢠⠋⢆⠀⠀⠀⢠⠋⢆⠀⠀⠀⢠⠋⢆⠀⠀⠀⢠⠋⢆⠀⠀⠀⠀
⠀⡸⠀⠘⡄⠀⠀⡸⠀⠘⡄⠀⠀⡸⠀⠘⡄⠀⠀⡸⠀⠘⡄⠀⠀⡸⠀⠘⡄⠀⠀⡸⠀⠘⡄⡇⠀⡜⠀⠘⡄⠀⠀⡜⠀⠘⡄⠀⠀⡜⠀⠸⡀⠀⠀⡜⠀⠸⡀⠀⠀⡜⠀⠸⡀⠀⠀⡜⠀⠸⡀⠀⠀⡜⠀⠸⡀⠀⠀⣀
⢀⠇⠀⠀⢱⠀⢀⠇⠀⠀⢣⠀⢀⠇⠀⠀⢣⠀⢀⠇⠀⠀⢣⠀⢠⠃⠀⠀⢣⠀⢠⠃⠀⠀⢣⡇⢠⠃⠀⠀⢣⠀⢠⠃⠀⠀⢣⠀⢠⠃⠀⠀⢣⠀⢠⠃⠀⠀⢇⠀⢠⠃⠀⠀⢇⠀⢠⠃⠀⠀⢇⠀⢰⠁⠀⠀⢇⠀⢰⠁
⠜⠀⠀⠀⠀⠣⠜⠀⠀⠀⠈⠦⠜⠀⠀⠀⠈⠦⠜⠀⠀⠀⠈⢆⠎⠀⠀⠀⠈⠦⠎⠀⠀⠀⠈⡧⠎⠀⠀⠀⠈⠦⠎⠀⠀⠀⠈⠦⠎⠀⠀⠀⠈⠦⠊⠀⠀⠀⠈⠦⠊⠀⠀⠀⠈⠦⠊⠀⠀⠀⠈⠦⠃⠀⠀⠀⠈⠦⠃⠀

Sun distance to Earth 2024
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣀⠤⠤⠔⠒⠒⠊⠉⡏⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠑⠒⠒⠢⠤⠤⣀⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣀⠤⠔⠒⠉⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠑⠒⠢⠤⣀⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣀⠤⠔⠒⠊⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠑⠒⠢⠤⣀⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⡤⠤⠤⠤⠤⠤⠤⠒⠒⠊⠉⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠉⠑⠒⠒⠤⠤⠤⠤⠤⠤⠤

Sun subtends 2024
⠉⠉⠉⠉⠉⠉⠑⠒⠒⠢⠤⢄⣀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⠤⠤⠔⠒⠒⠉⠉⠉⠉⠉⠉⠉
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠉⠑⠒⠤⠤⣀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⠤⠔⠒⠊⠉⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠉⠒⠒⠤⠤⣀⣀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣀⠤⠤⠒⠊⠉⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠉⠒⠒⠒⠤⠤⡧⠤⠤⠤⢄⠤⠤⠤⠤⠤⠤⠤⠒⠒⠊⠉⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀

Sun ecliptic longitude 2024
⣀⡠⠤⠤⠤⠤⠔⠒⠒⠒⠒⠊⠉⠉⠉⠉⢹⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⣀⣀⣀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣀⣀⣀⣀⠤⠤⠤⠤⠤⠔⠒⠒⠒⠒⠊⠉⠉⠉⠉⠁⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣀⣇⣀⣀⡠⠤⠤⠤⠤⠤⠒⠒⠒⠒⠒⠊⠉⠉⠉⠉⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡧⠤⠤⠤⠤⠔⠒⠒⠒⠒⠒⠉⠉⠉⠉⠉⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀

"
        );
    }

    #[test]
    fn json() {
        let args = vec![String::new(), String::from("--json")].into_iter();
        let config = Config::new(args).unwrap();

        assert!(config.json);
    }

    #[test]
    fn json_with_datetime_after() {
        let args = vec![
            String::new(),
            String::from("--json"),
            String::from("2024-05-15"),
        ]
        .into_iter();
        let config = Config::new(args).unwrap();

        assert!(config.json);
        assert!(config.datetime.is_some());
    }

    #[test]
    fn json_with_datetime_before() {
        let args = vec![
            String::new(),
            String::from("2024-05-15"),
            String::from("--json"),
        ]
        .into_iter();
        let config = Config::new(args).unwrap();

        assert!(config.json);
        assert!(config.datetime.is_some());
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
        let args = vec![String::new(), String::from("--invalid")].into_iter();
        let config = Config::new(args);

        assert!(config.is_err());
        assert!(config.err().unwrap().contains("'--invalid'"));
    }

    #[test]
    fn error_invalid_argument_short() {
        let args = vec![String::new(), String::from("-i")].into_iter();
        let config = Config::new(args);

        assert!(config.is_err());
        assert!(config.err().unwrap().contains("'-i'"));
    }

    // Main.

    #[test]
    fn try_parse_datetime_timestamp() {
        let dt = try_parse_datetime("966600000").unwrap();

        assert_eq!(dt, UTCDateTime::from_ymdhms(2000, 8, 18, 12, 0, 0));
    }

    #[test]
    fn try_parse_datetime_julian_date() {
        let dt = try_parse_datetime("2460473.19655").unwrap();

        assert_eq!(dt, UTCDateTime::from_ymdhms(2024, 6, 11, 16, 43, 2));
    }

    #[test]
    fn try_parse_datetime_datetime() {
        let dt = try_parse_datetime("1964-12-20T04:35:00Z").unwrap();

        assert_eq!(dt, UTCDateTime::from_ymdhms(1964, 12, 20, 4, 35, 0));
    }

    #[test]
    fn try_parse_datetime_error() {
        let dt = try_parse_datetime("invalid");

        assert!(dt.is_none());
    }

    #[test]
    fn try_from_timestamp_positive() {
        let dt = try_from_timestamp("966600000").unwrap();

        assert_eq!(dt, UTCDateTime::from_ymdhms(2000, 8, 18, 12, 0, 0));
    }

    #[test]
    fn try_from_timestamp_zero() {
        let dt = try_from_timestamp("0").unwrap();

        assert_eq!(dt, UTCDateTime::from_ymdhms(1970, 1, 1, 0, 0, 0));
    }

    #[test]
    fn try_from_timestamp_negative() {
        let dt = try_from_timestamp("-58200600").unwrap();

        assert_eq!(dt, UTCDateTime::from_ymdhms(1968, 2, 27, 9, 10, 0));
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
    fn try_from_julian_date_regular() {
        let dt = try_from_julian_date("2460473.19655").unwrap();

        assert_eq!(dt, UTCDateTime::from_ymdhms(2024, 6, 11, 16, 43, 2));
    }

    #[test]
    fn try_from_julian_date_zero() {
        let dt = try_from_julian_date("0.0").unwrap();

        assert_eq!(dt, UTCDateTime::from_ymdhms(-4712, 1, 1, 12, 0, 0));
    }

    #[test]
    fn try_from_datetime_regular() {
        let dt = try_from_iso_string("1964-12-20T04:35:00Z").unwrap();

        assert_eq!(dt, UTCDateTime::from_ymdhms(1964, 12, 20, 4, 35, 0));
    }

    #[test]
    fn try_from_datetime_implicit_utc() {
        let dt = try_from_iso_string("1964-12-20T04:35:00").unwrap();

        assert_eq!(dt, UTCDateTime::from_ymdhms(1964, 12, 20, 4, 35, 0));
    }

    #[test]
    fn try_from_datetime_offset() {
        let dt = try_from_iso_string("1964-12-20T05:35:00+01:00").unwrap();

        assert_eq!(dt, UTCDateTime::from_ymdhms(1964, 12, 20, 4, 35, 0));
    }

    #[test]
    fn try_from_datetime_error_invalid_string() {
        let dt = try_from_iso_string("1964-12-20T05-35-00");

        assert!(dt.is_none());
    }

    #[test]
    fn from_date() {
        let d = try_from_iso_string("1938-07-15").unwrap();

        assert_eq!(d, UTCDateTime::from_ymdhms(1938, 7, 15, 0, 0, 0));
    }

    #[test]
    fn from_date_error_invalid_string() {
        let d = try_from_iso_string("1938:07:15");

        assert!(d.is_none());
    }
}
