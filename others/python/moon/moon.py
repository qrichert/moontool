"""A Moon for Python.

John Walker's moontool.c calculation routines, ported to Python.
"""

import datetime as dt
import math
from dataclasses import dataclass
from typing import Self, cast

__all__ = [
    "MoonCalendar",
    "MoonPhase",
]

#  Astronomical constants

EPOCH: float = 2444238.5  # 1980 January 0.0

#  Constants defining the Sun's apparent orbit

ELONGE: float = 278.833540  # Ecliptic longitude of the Sun at epoch 1980.0
ELONGP: float = 282.596403  # Ecliptic longitude of the Sun at perigee
ECCENT: float = 0.016718  # Eccentricity of Earth's orbit
SUNSMAX: float = 1.495985e8  # Semi-major axis of Earth's orbit, km
SUNANGSIZ: float = 0.533128  # Sun's angular size, degrees, at semi-major axis distance

#  Elements of the Moon's orbit, epoch 1980.0

MMLONG: float = 64.975464  # Moon's mean longitude at the epoch
MMLONGP: float = 349.383063  # Mean longitude of the perigee at the epoch
# MLNODE: float = 151.950429  # Mean longitude of the node at the epoch
# MINC: float = 5.145396  # Inclination of the Moon's orbit
MECC: float = 0.054900  # Eccentricity of the Moon's orbit
MANGSIZ: float = 0.5181  # Moon's angular size at distance a from Earth
MSMAX: float = 384401.0  # Semi-major axis of Moon's orbit in km
# MPARALLAX: float = 0.9507  # Parallax at distance a from Earth
SYNMONTH: float = 29.53058868  # Synodic month (new Moon to new Moon)
LUNATBASE: float = 2423436.0  # Base date for E. W. Brown's numbered series of lunations (1923 January 16)

#  Properties of the Earth

EARTHRAD: float = 6378.16  # Radius of Earth in kilometres


#  Handy mathematical functions


def fixangle(a: float) -> float:
    """Fix angle."""
    return a - 360.0 * math.floor(a / 360.0)


def dsin(x: float) -> float:
    """Sin from deg."""
    return math.sin(math.radians(x))


def dcos(x: float) -> float:
    """Cos from deg."""
    return math.cos(math.radians(x))


def EPL(x: int) -> tuple[int, str]:
    return x, "" if x == 1 else "s"


MONAME: tuple[str, ...] = (
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
)

DAYNAME: tuple[str, ...] = (
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
)

PHANAME: tuple[str, ...] = (
    "New Moon",
    "Waxing Crescent",
    "First Quarter",
    "Waxing Gibbous",
    "Full Moon",
    "Waning Gibbous",
    "Last Quarter",
    "Waning Crescent",
)

MOONICN: tuple[str, ...] = (
    "\U0001f311",  # 🌑
    "\U0001f312",  # 🌒
    "\U0001f313",  # 🌓
    "\U0001f314",  # 🌔
    "\U0001f315",  # 🌕
    "\U0001f316",  # 🌖
    "\U0001f317",  # 🌗
    "\U0001f318",  # 🌘
)


# Custom API


@dataclass(slots=True, frozen=True)
class MoonPhase:
    """Information about the phase of the Moon at given time.

    Examples:
        >>> mphase = MoonPhase.for_datetime(dt.datetime(2024, 5, 4, 10, 0, 0))
        >>> mphase.phase_name
        'Waning Crescent'

    Attributes:
        subtends: Angular diameter.
        sun_subtends: Sun's angular diameter.

    Raises:
        OverflowError: In `MoonPhase.from_timestamp()` if the timestamp
            is out of the range of values supported by the platform C
            `localtime()` function.
        OSError: In `MoonPhase.from_timestamp()` on `localtime()`
            failure.
    """

    julian_date: float
    timestamp: int
    utc_datetime: dt.datetime
    age: float
    fraction_of_lunation: float
    phase: int
    phase_name: str
    phase_icon: str
    fraction_illuminated: float
    distance_to_earth_km: float
    distance_to_earth_earth_radii: float
    subtends: float
    sun_distance_to_earth_km: float
    sun_distance_to_earth_astronomical_units: float
    sun_subtends: float

    def __str__(self) -> str:
        aom: float = self.age
        aom_d: int = int(aom)
        aom_h: int = int((24 * (aom - math.floor(aom))))
        aom_m: int = int((1440 * (aom - math.floor(aom))) % 60)

        gm: dt.datetime = self.utc_datetime

        res: str = ""
        res += "Phase\n=====\n\n"
        res += "Julian date:\t\t{:.5f}   (0h variant: {:.5f})\n".format(
            self.julian_date, self.julian_date + 0.5
        )
        res += "Universal time:\t\t{:<9} {:>2}:{:0>2}:{:0>2} {:>2} {:<5} {}\n".format(
            DAYNAME[gm.isoweekday() % 7],
            gm.hour,
            gm.minute,
            gm.second,
            gm.day,
            MONAME[gm.month - 1],
            gm.year,
        )
        gm = gm.astimezone(tz=None)
        res += "Local time:\t\t{:<9} {:>2}:{:0>2}:{:0>2} {:>2} {:<5} {}\n\n".format(
            DAYNAME[gm.isoweekday() % 7],
            gm.hour,
            gm.minute,
            gm.second,
            gm.day,
            MONAME[gm.month - 1],
            gm.year,
        )

        res += "Age of moon:\t\t{} day{}, {} hour{}, {} minute{}.\n".format(
            *EPL(aom_d),
            *EPL(aom_h),
            *EPL(aom_m),
        )
        res += "Lunation:\t\t{:.2f}%   ({} {})\n".format(
            self.fraction_of_lunation * 100, self.phase_icon, self.phase_name
        )
        res += "Moon phase:\t\t{:.2f}%   (0% = New, 100% = Full)\n\n".format(
            self.fraction_illuminated * 100
        )

        res += "Moon's distance:\t{:.0f} kilometres, {:.1f} Earth radii.\n".format(
            self.distance_to_earth_km, self.distance_to_earth_earth_radii
        )
        res += "Moon subtends:\t\t{:.4f} degrees.\n\n".format(self.subtends)

        res += (
            "Sun's distance:\t\t{:.0f} kilometres, {:.3f} astronomical units.\n".format(
                self.sun_distance_to_earth_km,
                self.sun_distance_to_earth_astronomical_units,
            )
        )
        res += "Sun subtends:\t\t{:.4f} degrees.\n".format(self.sun_subtends)

        return res

    @classmethod
    def for_datetime(cls, datetime: dt.datetime) -> Self:
        return cast(Self, moonphase(datetime))

    @classmethod
    def for_timestamp(cls, timestamp: float) -> Self:
        return cls.for_datetime(dt.datetime.fromtimestamp(timestamp, tz=dt.UTC))

    @classmethod
    def now(cls) -> Self:  # pragma: no cover
        return cls.for_datetime(dt.datetime.now(tz=dt.UTC))


@dataclass(slots=True, frozen=True)
class MoonCalendar:
    """Information about past and future Moons, around given time.

    Note:
        `last_new_moon`, `first_quarter`, `full_moon`, `last_quarter`,
        and `next_new_moon`, are Julian Day Numbers (JDN)[^jdn].

        [^jdn]: https://en.wikipedia.org/wiki/Julian_day

    Examples:
        >>> mcal = MoonCalendar.for_datetime(dt.datetime(2024, 5, 4, 10, 0, 0))
        >>> mcal.lunation
        1253

    Attributes:
        lunation: Brown Lunation Number (BLN). Numbering begins at the
            first New Moon of 1923 (17 January 1923 at 2:41 UTC).

    Raises:
        OverflowError: In `MoonCalendar.from_timestamp()` if the
            timestamp is out of the range of values supported by the
            platform C `localtime()` function.
        OSError: In `MoonCalendar.from_timestamp()` on `localtime()`
            failure.
    """

    julian_date: float
    timestamp: int
    utc_datetime: dt.datetime
    lunation: int
    last_new_moon: float
    last_new_moon_utc: dt.datetime
    first_quarter: float
    first_quarter_utc: dt.datetime
    full_moon: float
    full_moon_utc: dt.datetime
    last_quarter: float
    last_quarter_utc: dt.datetime
    next_new_moon: float
    next_new_moon_utc: dt.datetime

    def __str__(self) -> str:
        res: str = ""

        res += "Moon Calendar\n=============\n\n"
        res += "Last new moon:\t\t{}\tLunation: {}\n".format(
            fmt_phase_time(self.last_new_moon_utc), self.lunation
        )
        res += "First quarter:\t\t{}\n".format(fmt_phase_time(self.first_quarter_utc))
        res += "Full moon:\t\t{}\n".format(fmt_phase_time(self.full_moon_utc))
        res += "Last quarter:\t\t{}\n".format(fmt_phase_time(self.last_quarter_utc))
        res += "Next new moon:\t\t{}\tLunation: {}\n".format(
            fmt_phase_time(self.next_new_moon_utc), self.lunation + 1
        )

        return res

    @classmethod
    def for_datetime(cls, datetime: dt.datetime) -> Self:
        return cast(Self, mooncal(datetime))

    @classmethod
    def for_timestamp(cls, timestamp: float) -> Self:
        return cls.for_datetime(dt.datetime.fromtimestamp(timestamp, tz=dt.UTC))

    @classmethod
    def now(cls) -> Self:  # pragma: no cover
        return cls.for_datetime(dt.datetime.now(tz=dt.UTC))


def fraction_of_lunation_to_phase(p: float) -> int:
    # Apart from Waxing and Waning, the other phases are very precise
    # points in time. For example, Full Moon occurs precisely at
    # `phase = 0.5`. This is too restrictive; for an observer, the Moon
    # appears Full over a larger timespan, rather than a single moment.
    # `day_frac` acts as padding around these lunar events, elongating
    # their duration artificially.
    day_frac: float = (1 / SYNMONTH) * 0.75

    if p < 0.00 + day_frac:
        return 0  # New Moon
    if p < 0.25 - day_frac:
        return 1  # Waxing Crescent
    if p < 0.25 + day_frac:
        return 2  # First Quarter
    if p < 0.50 - day_frac:
        return 3  # Waxing Gibbous
    if p < 0.50 + day_frac:
        return 4  # Full Moon
    if p < 0.75 - day_frac:
        return 5  # Waning Gibbous
    if p < 0.75 + day_frac:
        return 6  # Last Quarter
    if p < 1.00 - day_frac:
        return 7  # Waning Crescent
    return 0  # New Moon


def moonphase(gm: dt.datetime) -> MoonPhase:
    """Populate `MoonPhase` with info about the Moon at given time."""
    # Normalize and ensure UTC.
    gm = dt.datetime(
        gm.year, gm.month, gm.day, gm.hour, gm.minute, gm.second, tzinfo=dt.UTC
    )

    jd: float = jtime(gm)

    phase_info: dict = {}
    p: float = phase(jd, phase_info)

    phase_fraction: int = fraction_of_lunation_to_phase(p)
    mphase: MoonPhase = MoonPhase(
        julian_date=jd,
        timestamp=int(gm.timestamp()),
        utc_datetime=gm,
        age=phase_info["age"],
        fraction_of_lunation=p,
        phase=phase_fraction,
        phase_name=PHANAME[phase_fraction],
        phase_icon=MOONICN[phase_fraction],
        fraction_illuminated=phase_info["fraction_illuminated"],
        distance_to_earth_km=phase_info["distance"],
        distance_to_earth_earth_radii=phase_info["distance"] / EARTHRAD,
        subtends=phase_info["angular_diameter"],
        sun_distance_to_earth_km=phase_info["sun_distance"],
        sun_distance_to_earth_astronomical_units=phase_info["sun_distance"] / SUNSMAX,
        sun_subtends=phase_info["sun_angular_diameter"],
    )

    return mphase


def mooncal(gm: dt.datetime) -> MoonCalendar:
    """Populate `MoonCalendar` with info about lunation at given time."""
    jd: float = jtime(gm)

    phasar: tuple[float, float, float, float, float] = phasehunt(jd + 0.5)
    lunation: int = math.floor(((phasar[0] + 7) - LUNATBASE) / SYNMONTH) + 1
    mcal: MoonCalendar = MoonCalendar(
        julian_date=jd,
        timestamp=int(gm.timestamp()),
        utc_datetime=gm,
        lunation=lunation,
        last_new_moon=phasar[0],
        last_new_moon_utc=jtouct(phasar[0]),
        first_quarter=phasar[1],
        first_quarter_utc=jtouct(phasar[1]),
        full_moon=phasar[2],
        full_moon_utc=jtouct(phasar[2]),
        last_quarter=phasar[3],
        last_quarter_utc=jtouct(phasar[3]),
        next_new_moon=phasar[4],
        next_new_moon_utc=jtouct(phasar[4]),
    )

    return mcal


def fmt_phase_time(gm: dt.datetime) -> str:
    """Format the provided date and time in UTC format for screen display."""
    return "{:<9} {:>2}:{:0>2} UTC {:>2} {:<5} {}".format(
        DAYNAME[gm.isoweekday() % 7],
        gm.hour,
        gm.minute,
        gm.day,
        MONAME[gm.month - 1],
        gm.year,
    )


def jtime(t: dt.datetime) -> float:
    """Convert UTC date/time to astronomical Julian time.

    (i.e. Julian date plus day fraction, expressed as a floating point).
    """
    return ucttoj(t.year, t.month - 1, t.day, t.hour, t.minute, t.second)


def ucttoj(
    year: int, month: int, mday: int, hour: int, minute: int, second: int
) -> float:
    """Convert GMT date and time to astronomical Julian time.

    (i.e. Julian date plus day fraction, expressed as a floating point).
    """
    # Algorithm as given in Meeus, Astronomical Algorithms, Chapter 7, page 61

    # If PARANOID.
    # assert 0 <= month < 12
    # assert 0 < mday < 32
    # assert 0 <= hour < 24
    # assert 0 <= minute < 60
    # assert 0 <= second < 60

    m: int = month + 1
    y: int = year

    if m <= 2:
        y -= 1
        m += 12

    # Determine whether date is in Julian or Gregorian calendar based on
    # canonical date of calendar reform.

    if (year < 1582) or ((year == 1582) and ((month < 9) or (month == 9 and mday < 5))):
        b: int = 0
    else:
        a: int = int(y / 100)
        b: int = 2 - a + int(a / 4)

    return (int(365.25 * (y + 4716)) + int(30.6001 * (m + 1)) + mday + b - 1524.5) + (
        (second + 60 * int(minute + 60 * hour)) / 86400.0
    )


def jtouct(utime: float) -> dt.datetime:
    """Convert astronomical Julian time to GMT date and time."""
    yy, mm, dd = jyear(utime)
    hh, mmm, ss = jhms(utime)
    return dt.datetime(yy, mm, dd, hh, mmm, ss, tzinfo=dt.UTC)


def jyear(td: float) -> tuple[int, int, int]:
    """Convert Julian date to year, month, day.

    Year, month, day are returned via integers.
    """
    td += 0.5
    z: float = math.floor(td)
    f: float = td - z

    if z < 2299161.0:
        a: float = z
    else:
        alpha: float = math.floor((z - 1867216.25) / 36524.25)
        a: float = z + 1 + alpha - math.floor(alpha / 4)

    b: float = a + 1524
    c: float = math.floor((b - 122.1) / 365.25)
    d: float = math.floor(365.25 * c)
    e: float = math.floor((b - d) / 30.6001)

    dd: int = int(b - d - math.floor(30.6001 * e) + f)
    mm: int = int((e - 1) if (e < 14) else (e - 13))
    yy: int = int((c - 4716) if (mm > 2) else (c - 4715))

    return yy, mm, dd


def jhms(j: float) -> tuple[int, int, int]:
    """Convert Julian time to hour, minutes, and seconds."""
    j += 0.5  # Astronomical to civil
    ij: int = int(((j - math.floor(j)) * 86400.0) + 0.5)  # Round to nearest second
    h: int = int(ij / 3600)
    m: int = int((ij / 60) % 60)
    s: int = int(ij % 60)
    return h, m, s


def jwday(j: float) -> int:
    """Determine day of the week for a given Julian day."""
    return int(j + 1.5) % 7


def meanphase(sdate: float, k: float) -> float:
    """Calculates time of the mean new Moon for a given base date.

    This argument K to this function is the precomputed synodic month
    index, given by:

           K = (year - 1900) * 12.3685

    where year is expressed as a year and fractional year.
    """
    # Time in Julian centuries from 1900 January 0.5
    t: float = (sdate - 2415020.0) / 36525
    t2: float = t * t  # Square for frequent use
    t3: float = t2 * t  # Cube for frequent use

    nt1: float = (
        2415020.75933
        + SYNMONTH * k
        + 0.0001178 * t2
        - 0.000000155 * t3
        + 0.00033 * dsin(166.56 + 132.87 * t - 0.009173 * t2)
    )

    return nt1


def truephase(k: float, phase: float) -> float:
    """True, corrected phase time.

    Given a K value used to determine the mean phase of the new moon,
    and a phase selector (0.0, 0.25, 0.5, 0.75), obtain the true,
    corrected phase time.

    Raises:
        ValueError: If `truephase()` called with invalid phase selector.
    """
    apcor: bool = False

    k += phase  # Add phase to new moon time
    t: float = k / 1236.85  # Time in Julian centuries from 1900 January 0.5
    t2: float = t * t  # Square for frequent use
    t3: float = t2 * t  # Cube for frequent use

    # Mean time of phase
    pt: float = (
        2415020.75933
        + SYNMONTH * k
        + 0.0001178 * t2
        - 0.000000155 * t3
        + 0.00033 * dsin(166.56 + 132.87 * t - 0.009173 * t2)
    )
    # Sun's mean anomaly
    m: float = 359.2242 + 29.10535608 * k - 0.0000333 * t2 - 0.00000347 * t3
    # Moon's mean anomaly
    mprime: float = 306.0253 + 385.81691806 * k + 0.0107306 * t2 + 0.00001236 * t3
    # Moon's argument of latitude
    f: float = 21.2964 + 390.67050646 * k - 0.0016528 * t2 - 0.00000239 * t3

    if phase < 0.01 or abs(phase - 0.5) < 0.01:
        # Corrections for New and Full Moon
        pt += (
            (0.1734 - 0.000393 * t) * dsin(m)
            + 0.0021 * dsin(2 * m)
            - 0.4068 * dsin(mprime)
            + 0.0161 * dsin(2 * mprime)
            - 0.0004 * dsin(3 * mprime)
            + 0.0104 * dsin(2 * f)
            - 0.0051 * dsin(m + mprime)
            - 0.0074 * dsin(m - mprime)
            + 0.0004 * dsin(2 * f + m)
            - 0.0004 * dsin(2 * f - m)
            - 0.0006 * dsin(2 * f + mprime)
            + 0.0010 * dsin(2 * f - mprime)
            + 0.0005 * dsin(m + 2 * mprime)
        )
        apcor = True
    elif abs(phase - 0.25) < 0.01 or abs(phase - 0.75) < 0.01:
        pt += (
            (0.1721 - 0.0004 * t) * dsin(m)
            + 0.0021 * dsin(2 * m)
            - 0.6280 * dsin(mprime)
            + 0.0089 * dsin(2 * mprime)
            - 0.0004 * dsin(3 * mprime)
            + 0.0079 * dsin(2 * f)
            - 0.0119 * dsin(m + mprime)
            - 0.0047 * dsin(m - mprime)
            + 0.0003 * dsin(2 * f + m)
            - 0.0004 * dsin(2 * f - m)
            - 0.0006 * dsin(2 * f + mprime)
            + 0.0021 * dsin(2 * f - mprime)
            + 0.0003 * dsin(m + 2 * mprime)
            + 0.0004 * dsin(m - 2 * mprime)
            - 0.0003 * dsin(2 * m + mprime)
        )
        if phase < 0.5:
            # First quarter correction
            pt += 0.0028 - 0.0004 * dcos(m) + 0.0003 * dcos(mprime)
        else:
            # Last quarter correction
            pt += -0.0028 + 0.0004 * dcos(m) - 0.0003 * dcos(mprime)
        apcor = True

    if not apcor:
        raise ValueError("TRUEPHASE called with invalid phase selector.")

    return pt


def phasehunt(sdate: float) -> tuple[float, float, float, float, float]:
    """Find time of phases of the moon which surround the current date.

    Five phases are found, starting and ending with the new moons which
    bound the current lunation.
    """
    adate: float = sdate - 45

    yy: int
    mm: int
    yy, mm, _ = jyear(adate)
    k1: float = math.floor((yy + ((mm - 1) * (1.0 / 12.0)) - 1900) * 12.3685)
    k2: float

    adate = meanphase(adate, k1)
    nt1: float = adate
    nt2: float
    while True:
        adate += SYNMONTH
        k2 = k1 + 1
        nt2 = meanphase(adate, k2)
        if nt1 <= sdate < nt2:
            break
        nt1 = nt2
        k1 = k2

    # Return object as in moonphase()
    phases: tuple[float, float, float, float, float] = (
        truephase(k1, 0.0),
        truephase(k1, 0.25),
        truephase(k1, 0.5),
        truephase(k1, 0.75),
        truephase(k2, 0.0),
    )

    return phases


def kepler(m: float, ecc: float) -> float:
    """Solve the equation of Kepler."""
    # `sys.float_info.epsilon` (machine epsilon) is too small, which can
    # cause infinite loops here in some cases. Now we use the same value
    # as the C version, which is precise enough (tests still pass).
    EPSILON: float = 1e-6

    m = math.radians(m)
    e: float = m

    while True:
        delta: float = e - ecc * math.sin(e) - m
        e -= delta / (1 - ecc * math.cos(e))
        if abs(delta) <= EPSILON:
            break

    return e


def phase(pdate: float, phase_info: dict) -> float:
    """Calculate phase of moon as a fraction.

    The argument is the time for which the phase is requested, expressed
    as a Julian date and fraction. Returns the terminator phase angle as
    a percentage of a full circle (i.e., 0 to 1), and stores into a
    dictionary the illuminated fraction of the Moon's disc, the Moon's
    age in days and fraction, the distance of the Moon from the centre
    of the Earth, and the angular diameter subtended by the Moon as seen
    by an observer at the centre of the Earth.
    """
    Day: float = pdate - EPOCH  # Date within epoch
    N: float = fixangle((360 / 365.2422) * Day)  # Mean anomaly of the Sun
    M: float = fixangle(
        N + ELONGE - ELONGP
    )  # Convert from perigee co-ordinates to epoch 1980.0

    Ec: float = kepler(M, ECCENT)  # Solve equation of Kepler
    Ec = math.sqrt((1 + ECCENT) / (1 - ECCENT)) * math.tan(Ec / 2)
    Ec = 2 * math.degrees(math.atan(Ec))  # True anomaly
    Lambdasun: float = fixangle(Ec + ELONGP)  # Sun's geocentric ecliptic longitude

    # Orbital distance factor
    F: float = (1 + ECCENT * math.cos(math.radians(Ec))) / (1 - ECCENT * ECCENT)
    SunDist: float = SUNSMAX / F  # Distance to Sun in km
    SunAng: float = F * SUNANGSIZ  # Sun's angular size in degrees

    # Calculation of the Moon's position

    # Moon's mean longitude
    ml: float = fixangle(13.1763966 * Day + MMLONG)

    # Moon's mean anomaly
    MM: float = fixangle(ml - 0.1114041 * Day - MMLONGP)

    # Moon's ascending node mean longitude
    # MN: float = fixangle(MLNODE - 0.0529539 * Day)

    # Evection
    Ev: float = 1.2739 * math.sin(math.radians(2 * (ml - Lambdasun) - MM))

    # Annual equation
    Ae: float = 0.1858 * math.sin(math.radians(M))

    # Correction term
    A3: float = 0.37 * math.sin(math.radians(M))

    # Corrected anomaly
    MmP: float = MM + Ev - Ae - A3

    # Correction for the equation of the centre
    mEc: float = 6.2886 * math.sin(math.radians(MmP))

    # Another correction term
    A4: float = 0.214 * math.sin(math.radians(2 * MmP))

    # Corrected longitude
    lP: float = ml + Ev + mEc - Ae + A4

    # Variation
    V: float = 0.6583 * math.sin(math.radians(2 * (lP - Lambdasun)))

    # True longitude
    lPP: float = lP + V

    # Calculation of the Moon's inclination
    # (unused for phase calculation).

    # Corrected longitude of the node
    # NP: float = MN - 0.16 * math.sin(math.radians(M))

    # Y inclination coordinate
    # y: float = math.sin(math.radians(lPP - NP)) * math.cos(math.radians(MINC))

    # X inclination coordinate
    # x: float = math.cos(math.radians(lPP - NP))

    # Ecliptic longitude
    # Lambdamoon: float = math.degrees(math.atan2(y, x))
    # Lambdamoon += NP

    # Ecliptic latitude
    # BetaM: float = math.degrees(
    #     math.asin(math.sin(math.radians(lPP - NP)) * math.sin(math.radians(MINC)))
    # )

    # Calculation of the phase of the Moon

    # Age of the Moon in degrees
    MoonAge: float = lPP - Lambdasun

    # Phase of the Moon
    MoonPhase: float = (1 - math.cos(math.radians(MoonAge))) / 2

    # Calculate distance of moon from the centre of the Earth

    MoonDist: float = (MSMAX * (1 - MECC * MECC)) / (
        1 + MECC * math.cos(math.radians(MmP + mEc))
    )

    # Calculate Moon's angular diameter

    MoonDFrac: float = MoonDist / MSMAX
    MoonAng: float = MANGSIZ / MoonDFrac

    # Calculate Moon's parallax

    # MoonPar: float = MPARALLAX / MoonDFrac

    Phase: float = fixangle(MoonAge) / 360.0

    phase_info["fraction_illuminated"] = MoonPhase
    phase_info["age"] = SYNMONTH * Phase
    phase_info["distance"] = MoonDist
    phase_info["angular_diameter"] = MoonAng
    phase_info["sun_distance"] = SunDist
    phase_info["sun_angular_diameter"] = SunAng
    return Phase
