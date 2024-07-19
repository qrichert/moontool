import datetime as dt
import doctest
import re
import unittest
from textwrap import dedent

import moontool.moon
from moontool.moon import (
    EPL,
    MOONICN,
    PHANAME,
    MoonCalendar,
    MoonPhase,
    dcos,
    dsin,
    fixangle,
    fmt_phase_time,
    fraction_of_lunation_to_phase,
    jhms,
    jtime,
    jtouct,
    jwday,
    jyear,
    kepler,
    meanphase,
    mooncal,
    moonphase,
    phase,
    phasehunt,
    truephase,
    ucttoj,
)


def load_tests(
    loader: unittest.TestLoader, tests: unittest.TestSuite, ignore: str
) -> unittest.TestSuite:
    """Add module doctests."""
    tests.addTests(doctest.DocTestSuite(moontool.moon))
    return tests


def redact_local_time(phase: str) -> str:
    """Replace local time with dashes.

    Local time varies according to the machine's timezone. This is not
    suitable for tests.
    """

    def redact(match_: re.Match) -> str:
        return match_.group(1) + "-" * len(match_.group(2)) + "\n"

    return re.sub(r"(Local time:\s+)(.+)\n", redact, phase)


class TestUtils(unittest.TestCase):
    def test_fixangle_all(self) -> None:
        self.assertAlmostEqual(fixangle(-400.0), 320.0)
        self.assertAlmostEqual(fixangle(-350.0), 10.0)
        self.assertAlmostEqual(fixangle(-0.0), 0.0)
        self.assertAlmostEqual(fixangle(350.0), 350.0)
        self.assertAlmostEqual(fixangle(400.0), 40.0)

    def test_dsin_all(self) -> None:
        self.assertAlmostEqual(dsin(-400.0), -0.6427876096865393)
        self.assertAlmostEqual(dsin(-350.0), 0.1736481776669304)
        self.assertAlmostEqual(dsin(-0.0), 0.0)
        self.assertAlmostEqual(dsin(350.0), -0.1736481776669304)
        self.assertAlmostEqual(dsin(400.0), 0.6427876096865393)

    def test_dcos_all(self) -> None:
        self.assertAlmostEqual(dcos(-400.0), 0.7660444431189781)
        self.assertAlmostEqual(dcos(-350.0), 0.984807753012208)
        self.assertAlmostEqual(dcos(-0.0), 1.0)
        self.assertAlmostEqual(dcos(350.0), 0.984807753012208)
        self.assertAlmostEqual(dcos(400.0), 0.7660444431189781)

    def test_epl_all(self) -> None:
        self.assertEqual(EPL(0), (0, "s"))
        self.assertEqual(EPL(1), (1, ""))
        self.assertEqual(EPL(2), (2, "s"))


class TestCustomAPI(unittest.TestCase):
    def test_every_way_of_creating_moonphase_gives_same_result(self) -> None:
        a = moonphase(dt.datetime(1968, 2, 27, 9, 10, 0, tzinfo=dt.UTC))
        b = MoonPhase.for_datetime(dt.datetime(1968, 2, 27, 9, 10, 0, tzinfo=dt.UTC))
        c = MoonPhase.for_timestamp(-58200600)

        self.assertTrue(all([x == a for x in (b, c)]))

    def test_moonphase_regular(self) -> None:
        mphase = moonphase(dt.datetime(1995, 3, 11, 1, 40, 0, tzinfo=dt.UTC))

        self.assertEqual(
            mphase,
            MoonPhase(
                julian_date=2449787.5694444445,
                timestamp=794886000,
                utc_datetime=dt.datetime(1995, 3, 11, 1, 40, 0, tzinfo=dt.UTC),
                age=8.861826144635483,
                fraction_of_lunation=0.3000897219037586,
                phase=3,
                phase_name="Waxing Gibbous",
                phase_icon="🌔",
                fraction_illuminated=0.6547765466116484,
                distance_to_earth_km=402304.145927074,
                distance_to_earth_earth_radii=63.07526715025556,
                subtends=0.49504376257683796,
                sun_distance_to_earth_km=148602888.21560264,
                sun_distance_to_earth_astronomical_units=0.9933447742831822,
                sun_subtends=0.5366998587018451,
            ),
        )

    def test_moonphase_display(self) -> None:
        mphase = moonphase(dt.datetime(1995, 3, 11, 1, 40, 0, tzinfo=dt.UTC))

        self.assertEqual(
            redact_local_time(str(mphase)),
            dedent("""
            Phase
            =====

            Julian date:\t\t2449787.56944   (0h variant: 2449788.06944)
            Universal time:\t\tSaturday   1:40:00 11 March 1995
            Local time:\t\t--------------------------------

            Age of moon:\t\t8 days, 20 hours, 41 minutes.
            Lunation:\t\t30.01%   (🌔 Waxing Gibbous)
            Moon phase:\t\t65.48%   (0% = New, 100% = Full)

            Moon's distance:\t402304 kilometres, 63.1 Earth radii.
            Moon subtends:\t\t0.4950 degrees.

            Sun's distance:\t\t148602888 kilometres, 0.993 astronomical units.
            Sun subtends:\t\t0.5367 degrees.
            """).lstrip("\n"),
        )

    def test_every_way_of_creating_mooncalendar_gives_same_result(self) -> None:
        a = mooncal(dt.datetime(1968, 2, 27, 9, 10, 0, tzinfo=dt.UTC))
        b = MoonCalendar.for_datetime(dt.datetime(1968, 2, 27, 9, 10, 0, tzinfo=dt.UTC))
        c = MoonCalendar.for_timestamp(-58200600)

        self.assertTrue(all([x == a for x in (b, c)]))

    def test_mooncalendar_regular(self) -> None:
        mcal = mooncal(dt.datetime(1995, 3, 11, 1, 40, 0, tzinfo=dt.UTC))

        self.assertEqual(
            mcal,
            MoonCalendar(
                julian_date=2449787.5694444445,
                timestamp=794886000,
                utc_datetime=dt.datetime(1995, 3, 11, 1, 40, 0, tzinfo=dt.UTC),
                lunation=893,
                last_new_moon=2449777.9930243203,
                last_new_moon_utc=dt.datetime(1995, 3, 1, 11, 49, 57, tzinfo=dt.UTC),
                first_quarter=2449785.9259425676,
                first_quarter_utc=dt.datetime(1995, 3, 9, 10, 13, 21, tzinfo=dt.UTC),
                full_moon=2449793.5607311586,
                full_moon_utc=dt.datetime(1995, 3, 17, 1, 27, 27, tzinfo=dt.UTC),
                last_quarter=2449800.3410721812,
                last_quarter_utc=dt.datetime(1995, 3, 23, 20, 11, 9, tzinfo=dt.UTC),
                next_new_moon=2449807.5908233593,
                next_new_moon_utc=dt.datetime(1995, 3, 31, 2, 10, 47, tzinfo=dt.UTC),
            ),
        )

    def test_mooncalendar_display(self) -> None:
        mcal = mooncal(dt.datetime(1995, 3, 11, 1, 40, 0, tzinfo=dt.UTC))

        self.assertEqual(
            str(mcal),
            dedent("""
            Moon Calendar
            =============

            Last new moon:\t\tWednesday 11:49 UTC  1 March 1995\tLunation: 893
            First quarter:\t\tThursday  10:13 UTC  9 March 1995
            Full moon:\t\tFriday     1:27 UTC 17 March 1995
            Last quarter:\t\tThursday  20:11 UTC 23 March 1995
            Next new moon:\t\tFriday     2:10 UTC 31 March 1995\tLunation: 894
            """).lstrip("\n"),
        )


class TestMoon(unittest.TestCase):
    def test_fraction_of_lunation_to_phase_number(self) -> None:
        new_moon_start = fraction_of_lunation_to_phase(0)
        self.assertEqual(new_moon_start, 0)

        waxing_crescent = fraction_of_lunation_to_phase(0.15)
        self.assertEqual(waxing_crescent, 1)

        first_quarter = fraction_of_lunation_to_phase(0.25)
        self.assertEqual(first_quarter, 2)

        waxing_gibbous = fraction_of_lunation_to_phase(0.35)
        self.assertEqual(waxing_gibbous, 3)

        full_moon = fraction_of_lunation_to_phase(0.5)
        self.assertEqual(full_moon, 4)

        waning_gibbous = fraction_of_lunation_to_phase(0.65)
        self.assertEqual(waning_gibbous, 5)

        last_quarter = fraction_of_lunation_to_phase(0.75)
        self.assertEqual(last_quarter, 6)

        waning_crescent = fraction_of_lunation_to_phase(0.85)
        self.assertEqual(waning_crescent, 7)

        new_moon_end = fraction_of_lunation_to_phase(1)
        self.assertEqual(new_moon_end, 0)

    def test_fraction_of_lunation_to_phase_name(self) -> None:
        new_moon_start = PHANAME[fraction_of_lunation_to_phase(0)]
        self.assertEqual(new_moon_start, "New Moon")

        waxing_crescent = PHANAME[fraction_of_lunation_to_phase(0.15)]
        self.assertEqual(waxing_crescent, "Waxing Crescent")

        first_quarter = PHANAME[fraction_of_lunation_to_phase(0.25)]
        self.assertEqual(first_quarter, "First Quarter")

        waxing_gibbous = PHANAME[fraction_of_lunation_to_phase(0.35)]
        self.assertEqual(waxing_gibbous, "Waxing Gibbous")

        full_moon = PHANAME[fraction_of_lunation_to_phase(0.5)]
        self.assertEqual(full_moon, "Full Moon")

        waning_gibbous = PHANAME[fraction_of_lunation_to_phase(0.65)]
        self.assertEqual(waning_gibbous, "Waning Gibbous")

        last_quarter = PHANAME[fraction_of_lunation_to_phase(0.75)]
        self.assertEqual(last_quarter, "Last Quarter")

        waning_crescent = PHANAME[fraction_of_lunation_to_phase(0.85)]
        self.assertEqual(waning_crescent, "Waning Crescent")

        new_moon_end = PHANAME[fraction_of_lunation_to_phase(1)]
        self.assertEqual(new_moon_end, "New Moon")

    def test_fraction_of_lunation_to_phase_icon(self) -> None:
        new_moon_start = MOONICN[fraction_of_lunation_to_phase(0)]
        self.assertEqual(new_moon_start, "🌑")

        waxing_crescent = MOONICN[fraction_of_lunation_to_phase(0.15)]
        self.assertEqual(waxing_crescent, "🌒")

        first_quarter = MOONICN[fraction_of_lunation_to_phase(0.25)]
        self.assertEqual(first_quarter, "🌓")

        waxing_gibbous = MOONICN[fraction_of_lunation_to_phase(0.35)]
        self.assertEqual(waxing_gibbous, "🌔")

        full_moon = MOONICN[fraction_of_lunation_to_phase(0.5)]
        self.assertEqual(full_moon, "🌕")

        waning_gibbous = MOONICN[fraction_of_lunation_to_phase(0.65)]
        self.assertEqual(waning_gibbous, "🌖")

        last_quarter = MOONICN[fraction_of_lunation_to_phase(0.75)]
        self.assertEqual(last_quarter, "🌗")

        waning_crescent = MOONICN[fraction_of_lunation_to_phase(0.85)]
        self.assertEqual(waning_crescent, "🌘")

        new_moon_end = MOONICN[fraction_of_lunation_to_phase(1)]
        self.assertEqual(new_moon_end, "🌑")

    def test_fmt_phase_time_regular(self) -> None:
        gm = dt.datetime(1995, 3, 12, 11, 16, 26, tzinfo=dt.UTC)

        res = fmt_phase_time(gm)

        self.assertEqual(res, "Sunday    11:16 UTC 12 March 1995")

    def test_fmt_phase_time_month_padding(self) -> None:
        gm = dt.datetime(1995, 3, 12, 11, 16, 26)

        gm = gm.replace(month=5)  # May (shortest)
        self.assertEqual(fmt_phase_time(gm), "Friday    11:16 UTC 12 May   1995")

        gm = gm.replace(month=9)  # September (longest)
        self.assertEqual(fmt_phase_time(gm), "Tuesday   11:16 UTC 12 September 1995")

        gm = gm.replace(month=7)  # July (4 chars = 1 char padding)
        self.assertEqual(fmt_phase_time(gm), "Wednesday 11:16 UTC 12 July  1995")

        gm = gm.replace(month=3)  # March (5 chars = exact)
        self.assertEqual(fmt_phase_time(gm), "Sunday    11:16 UTC 12 March 1995")

        gm = gm.replace(month=8)  # August (6 chars = no padding)
        self.assertEqual(fmt_phase_time(gm), "Saturday  11:16 UTC 12 August 1995")

    def test_fmt_phase_time_at_boundaries(self) -> None:
        gm = dt.datetime(1995, 3, 12, 11, 16, 26)

        gm = gm.replace(day=12)  # Sunday
        self.assertEqual(fmt_phase_time(gm), "Sunday    11:16 UTC 12 March 1995")

        gm = gm.replace(day=13)  # Monday
        self.assertEqual(fmt_phase_time(gm), "Monday    11:16 UTC 13 March 1995")

        gm = gm.replace(day=11)  # Saturday
        self.assertEqual(fmt_phase_time(gm), "Saturday  11:16 UTC 11 March 1995")

        gm = gm.replace(month=1)  # January
        self.assertEqual(fmt_phase_time(gm), "Wednesday 11:16 UTC 11 January 1995")

        gm = gm.replace(month=12)  # December
        self.assertEqual(fmt_phase_time(gm), "Monday    11:16 UTC 11 December 1995")

    def test_jtime_regular(self) -> None:
        jd = jtime(dt.datetime(1995, 3, 11, 1, 40, 0, tzinfo=dt.UTC))

        self.assertAlmostEqual(jd, 2449787.5694444445)

    def test_jtime_january(self) -> None:
        jd = jtime(dt.datetime(1995, 1, 1, 0, 0, 0, tzinfo=dt.UTC))

        self.assertAlmostEqual(jd, 2449718.5)

    def test_ucttoj_regular(self) -> None:
        julian_date = ucttoj(1995, 2, 11, 0, 0, 0)

        self.assertAlmostEqual(julian_date, 2449787.5)

    def test_ucttoj_month_lte_2(self) -> None:
        julian_date = ucttoj(1900, 1, 1, 0, 0, 0)

        self.assertAlmostEqual(julian_date, 2415051.5)

    def test_ucttoj_year_1582(self) -> None:
        julian_date = ucttoj(1582, 9, 4, 0, 0, 0)

        self.assertAlmostEqual(julian_date, 2299159.5)

    def test_jtouct_regular(self) -> None:
        gm = jtouct(2438749.732639)

        self.assertEqual(
            gm,
            dt.datetime(1964, 12, 20, 5, 35, 0, tzinfo=dt.UTC),
        )

    def test_jyear_regular(self) -> None:
        ymd = jyear(2460426.09191)

        self.assertEqual(ymd, (2024, 4, 25))

    def test_jyear_before_october_15_1582(self) -> None:
        ymd = jyear(2299160.0)

        self.assertEqual(ymd, (1582, 10, 4))

    def test_jyear_on_october_15_1582(self) -> None:
        ymd = jyear(2299160.9)

        self.assertEqual(ymd, (1582, 10, 15))

    def test_jhms_regular(self) -> None:
        hms = jhms(2438749.732639)  # P

        self.assertEqual(hms, (5, 35, 0))

    def test_jhms_zero(self) -> None:
        hms = jhms(0.0)

        self.assertEqual(hms, (12, 0, 0))

    def test_jhms_negative(self) -> None:
        hms = jhms(-1200941.5)

        self.assertEqual(hms, (0, 0, 0))

    def test_jwday_regular(self) -> None:
        wday = jwday(2439913.881944)  # M

        self.assertEqual(wday, 2)

    def test_jwday_positive_all_days(self) -> None:
        self.assertEqual(jwday(2439912.0), 0)  # Sunday
        self.assertEqual(jwday(2439913.0), 1)
        self.assertEqual(jwday(2439914.0), 2)
        self.assertEqual(jwday(2439915.0), 3)
        self.assertEqual(jwday(2439916.0), 4)
        self.assertEqual(jwday(2439917.0), 5)
        self.assertEqual(jwday(2439918.0), 6)
        self.assertEqual(jwday(2439919.0), 0)

    def test_meanphase_regular(self):
        meanph = meanphase(2460381.612639, 1535.0)

        self.assertAlmostEqual(meanph, 2460350.2129780464)

    def test_truephase_lt_0_01(self) -> None:
        trueph = truephase(1537.0, 0)

        self.assertAlmostEqual(trueph, 2460409.266218814)

    def test_truephase_abs_min_0_25_lt_0_01_and_lt_0_5(self) -> None:
        trueph = truephase(1537.0, 0.25)

        self.assertAlmostEqual(trueph, 2460416.3017252507)

    def test_truephase_abs_min_0_75_lt_0_01_and_gte_0_5(self) -> None:
        trueph = truephase(1537.0, 0.75)

        self.assertAlmostEqual(trueph, 2460431.9776856042)

    def test_truephase_invalid_phase_selector(self) -> None:
        with self.assertRaises(ValueError):
            truephase(1537.0, 1)

    def test_phasehunt_regular(self) -> None:
        phasar = phasehunt(2449818.3)

        self.assertEqual(
            phasar,
            (
                2449807.5908233593,
                2449815.7327970425,
                2449823.006760471,
                2449829.6385180936,
                2449837.2348421547,
            ),
        )

    def test_kepler_regular(self) -> None:
        ec = kepler(111.615376, 0.016718)

        self.assertAlmostEqual(ec, 1.9635011880995301)

    def test_phase_regular(self) -> None:
        phase_info = {}
        p = phase(2449818.7, phase_info)

        self.assertEqual(p, 0.34488787994113507)
        self.assertEqual(
            phase_info,
            {
                "fraction_illuminated": 0.7807502920288827,
                "age": 10.184742123258882,
                "distance": 389080.0632791394,
                "angular_diameter": 0.5118693474590013,
                "sun_distance": 149916135.21839374,
                "sun_angular_diameter": 0.5319984336029933,
            },
        )
