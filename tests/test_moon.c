#include "../moon/moon.c"

#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <time.h>

void assert_almost_equal(double a, double b) {
    if (abs(a - b) > 1e-7) {
        fprintf(stderr, "%f != %f\n", a, b);
        assert(FALSE);
    }
}

void redact_local_time(char* phase) {
    char* prefix = "Local time:\t\t";
    int prefix_len = strlen(prefix);

    while (*phase) {
        if (strncmp(prefix, phase, prefix_len) == 0) {
            // Found. Redact until newline, then leave.
            phase += prefix_len;
            while (*phase && *phase != '\n') {
                *phase = '-';
                ++phase;
            }
            return;
        }
        ++phase;
    }
}

/**
 * Helper function to print differing portion of two strings.
 *
 * This is meant to be used in case of a failing string-comparing test.
 */
int posdiff(char* p1, char* p2) {
    int pos = 0;
    while (*p1 && (*p1 == *p2)) {
        ++pos;
        ++p1;
        ++p2;
    }
    printf("%s", p1 - 1);
    return pos;
}

// Utils

void test_abs_all(void) {
    assert_almost_equal(abs(-1.12), 1.12);
    assert_almost_equal(abs(0.0), 0.0);
    assert_almost_equal(abs(1.12), 1.12);
}

void test_fixangle_all(void) {
    assert_almost_equal(fixangle(-400.0), 320.0);
    assert_almost_equal(fixangle(-350.0), 10.0);
    assert_almost_equal(fixangle(-0.0), 0.0);
    assert_almost_equal(fixangle(350.0), 350.0);
    assert_almost_equal(fixangle(400.0), 40.0);
}

void test_torad_all(void) {
    assert_almost_equal(torad(-400.0), -6.981317007977318);
    assert_almost_equal(torad(-350.0), -6.1086523819801535);
    assert_almost_equal(torad(-0.0), 0.0);
    assert_almost_equal(torad(350.0), 6.1086523819801535);
    assert_almost_equal(torad(400.0), 6.981317007977318);
}

void test_todeg_all(void) {
    assert_almost_equal(todeg(-6.981317007977318), -400.0);
    assert_almost_equal(todeg(-6.1086523819801535), -350.0);
    assert_almost_equal(todeg(0.0), -0.0);
    assert_almost_equal(todeg(6.1086523819801535), 350.0);
    assert_almost_equal(todeg(6.981317007977318), 400.0);
}

void test_dsin_all(void) {
    assert_almost_equal(dsin(-400.0), -0.6427876096865393);
    assert_almost_equal(dsin(-350.0), 0.1736481776669304);
    assert_almost_equal(dsin(-0.0), 0.0);
    assert_almost_equal(dsin(350.0), -0.1736481776669304);
    assert_almost_equal(dsin(400.0), 0.6427876096865393);
}

void test_dcos_all(void) {
    assert_almost_equal(dcos(-400.0), 0.7660444431189781);
    assert_almost_equal(dcos(-350.0), 0.984807753012208);
    assert_almost_equal(dcos(-0.0), 1.0);
    assert_almost_equal(dcos(350.0), 0.984807753012208);
    assert_almost_equal(dcos(400.0), 0.7660444431189781);
}

void test_epl_all(void) {
    char buf[5];

    sprintf(buf, "%d%s", EPL(0));
    assert(strcmp(buf, "0s") == 0);

    sprintf(buf, "%d%s", EPL(1));
    assert(strcmp(buf, "1") == 0);

    sprintf(buf, "%d%s", EPL(2));
    assert(strcmp(buf, "2s") == 0);
}

void test_moonphase_regular(void) {
    MoonPhase mphase;
    time_t timestamp = 794886000;

    moonphase(&mphase, &timestamp);

    assert_almost_equal(mphase.julian_date, 2449787.5694444445);
    assert_almost_equal(mphase.utc_timestamp, 794886000);

    assert(mphase.utc_datetime.tm_year == 95);
    assert(mphase.utc_datetime.tm_mon == 2);
    assert(mphase.utc_datetime.tm_mday == 11);
    assert(mphase.utc_datetime.tm_wday == 6);
    assert(mphase.utc_datetime.tm_hour == 1);
    assert(mphase.utc_datetime.tm_min == 40);
    assert(mphase.utc_datetime.tm_sec == 0);

    assert_almost_equal(mphase.age, 8.861826144635483);
    assert_almost_equal(mphase.fraction_of_lunation, 0.3000897219037586);
    assert_almost_equal(mphase.phase, 3);
    assert(strcmp(mphase.phase_name, "Waxing Gibbous") == 0);
    assert(strcmp(mphase.phase_icon, "🌔") == 0);
    assert_almost_equal(mphase.fraction_illuminated, 0.6547765466116484);
    assert_almost_equal(mphase.distance_to_earth_km, 402304.145927074);
    assert_almost_equal(mphase.distance_to_earth_earth_radii, 63.07526715025556);
    assert_almost_equal(mphase.subtends, 0.49504376257683796);
    assert_almost_equal(mphase.sun_distance_to_earth_km, 148602888.21560264);
    assert_almost_equal(
        mphase.sun_distance_to_earth_astronomical_units, 0.9933447742831822
    );
    assert_almost_equal(mphase.sun_subtends, 0.5366998587018451);
}

void test_moonphase_multiple_creations(void) {
    MoonPhase mphase;
    time_t timestamp = 794886000;

    moonphase(&mphase, &timestamp);

    MoonPhase other;
    moonphase(&other, &timestamp);

    // Ensure `struct tm`s are their own copies.
    assert(&mphase.utc_datetime != &other.utc_datetime);
}

void test_moonphase_display(void) {
    MoonPhase mphase;
    time_t timestamp = 794886000;

    moonphase(&mphase, &timestamp);

    char buf[1000];
    moonphase_to_strbuf(&mphase, buf);

    redact_local_time(buf);

    assert(
        strcmp(
            buf,
            "Phase\n"
            "=====\n"
            "\n"
            "Julian date:\t\t2449787.56944   (0h variant: 2449788.06944)\n"
            "Universal time:\t\tSaturday   1:40:00 11 March 1995\n"
            "Local time:\t\t--------------------------------\n"
            "\n"
            "Age of moon:\t\t8 days, 20 hours, 41 minutes.\n"
            "Lunation:\t\t30.01%   (\U0001f314 Waxing Gibbous)\n"
            "Moon phase:\t\t65.48%   (0% = New, 100% = Full)\n"
            "\n"
            "Moon's distance:\t402304 kilometres, 63.1 Earth radii.\n"
            "Moon subtends:\t\t0.4950 degrees.\n"
            "\n"
            "Sun's distance:\t\t148602888 kilometres, 0.993 astronomical units.\n"
            "Sun subtends:\t\t0.5367 degrees."
        )
        == 0
    );
}

void test_mooncalendar_regular(void) {
    MoonCalendar mcal;
    time_t timestamp = 794886000;

    mooncal(&mcal, &timestamp);

    assert(mcal.lunation == 893);
    assert_almost_equal(mcal.last_new_moon, 2449777.9930243203);
    assert(mcal.last_new_moon_utc.tm_year == 95);
    assert(mcal.last_new_moon_utc.tm_mon == 2);
    assert(mcal.last_new_moon_utc.tm_mday == 1);
    assert(mcal.last_new_moon_utc.tm_wday == 3);
    assert(mcal.last_new_moon_utc.tm_hour == 11);
    assert(mcal.last_new_moon_utc.tm_min == 49);
    assert(mcal.last_new_moon_utc.tm_sec == 57);

    assert_almost_equal(mcal.first_quarter, 2449785.9259425676);
    assert(mcal.first_quarter_utc.tm_year == 95);
    assert(mcal.first_quarter_utc.tm_mon == 2);
    assert(mcal.first_quarter_utc.tm_mday == 9);
    assert(mcal.first_quarter_utc.tm_wday == 4);
    assert(mcal.first_quarter_utc.tm_hour == 10);
    assert(mcal.first_quarter_utc.tm_min == 13);
    assert(mcal.first_quarter_utc.tm_sec == 21);

    assert_almost_equal(mcal.full_moon, 2449793.5607311586);
    assert(mcal.full_moon_utc.tm_year == 95);
    assert(mcal.full_moon_utc.tm_mon == 2);
    assert(mcal.full_moon_utc.tm_mday == 17);
    assert(mcal.full_moon_utc.tm_wday == 5);
    assert(mcal.full_moon_utc.tm_hour == 1);
    assert(mcal.full_moon_utc.tm_min == 27);
    assert(mcal.full_moon_utc.tm_sec == 27);

    assert_almost_equal(mcal.last_quarter, 2449800.3410721812);
    assert(mcal.last_quarter_utc.tm_year == 95);
    assert(mcal.last_quarter_utc.tm_mon == 2);
    assert(mcal.last_quarter_utc.tm_mday == 23);
    assert(mcal.last_quarter_utc.tm_wday == 4);
    assert(mcal.last_quarter_utc.tm_hour == 20);
    assert(mcal.last_quarter_utc.tm_min == 11);
    assert(mcal.last_quarter_utc.tm_sec == 9);

    assert_almost_equal(mcal.next_new_moon, 2449807.5908233593);
    assert(mcal.next_new_moon_utc.tm_year == 95);
    assert(mcal.next_new_moon_utc.tm_mon == 2);
    assert(mcal.next_new_moon_utc.tm_mday == 31);
    assert(mcal.next_new_moon_utc.tm_wday == 5);
    assert(mcal.next_new_moon_utc.tm_hour == 2);
    assert(mcal.next_new_moon_utc.tm_min == 10);
    assert(mcal.next_new_moon_utc.tm_sec == 47);
}

void test_mooncalendar_multiple_creations(void) {
    MoonCalendar mcal;
    time_t timestamp = 794886000;

    mooncal(&mcal, &timestamp);

    MoonCalendar other;
    mooncal(&other, &timestamp);

    // Ensure `struct tm`s are their own copies.
    assert(&mcal.last_new_moon_utc != &other.last_new_moon_utc);
    assert(&mcal.first_quarter_utc != &other.first_quarter_utc);
    assert(&mcal.full_moon_utc != &other.full_moon_utc);
    assert(&mcal.last_quarter_utc != &other.last_quarter_utc);
    assert(&mcal.next_new_moon_utc != &other.next_new_moon_utc);
}

void test_mooncalendar_display(void) {
    MoonCalendar mcal;
    time_t timestamp = 794886000;

    mooncal(&mcal, &timestamp);

    char buf[500];
    mooncal_to_strbuf(&mcal, buf);

    assert(
        strcmp(
            buf,
            "Moon Calendar\n"
            "=============\n"
            "\n"
            "Last new moon:\t\tWednesday 11:49 UTC  1 March 1995\tLunation: 893\n"
            "First quarter:\t\tThursday  10:13 UTC  9 March 1995\n"
            "Full moon:\t\tFriday     1:27 UTC 17 March 1995\n"
            "Last quarter:\t\tThursday  20:11 UTC 23 March 1995\n"
            "Next new moon:\t\tFriday     2:10 UTC 31 March 1995\tLunation: 894"
        )
        == 0
    );
}


void test_fraction_of_lunation_to_phase_number(void) {
    int new_moon_start = fraction_of_lunation_to_phase(0);
    assert(new_moon_start == 0);

    int waxing_crescent = fraction_of_lunation_to_phase(0.15);
    assert(waxing_crescent == 1);

    int first_quarter = fraction_of_lunation_to_phase(0.25);
    assert(first_quarter == 2);

    int waxing_gibbous = fraction_of_lunation_to_phase(0.35);
    assert(waxing_gibbous == 3);

    int full_moon = fraction_of_lunation_to_phase(0.5);
    assert(full_moon == 4);

    int waning_gibbous = fraction_of_lunation_to_phase(0.65);
    assert(waning_gibbous == 5);

    int last_quarter = fraction_of_lunation_to_phase(0.75);
    assert(last_quarter == 6);

    int waning_crescent = fraction_of_lunation_to_phase(0.85);
    assert(waning_crescent == 7);

    int new_moon_end = fraction_of_lunation_to_phase(1);
    assert(new_moon_end == 0);
}

void test_fraction_of_lunation_to_phase_name(void) {
    char* new_moon_start = phaname[fraction_of_lunation_to_phase(0)];
    assert(strcmp(new_moon_start, "New Moon") == 0);

    char* waxing_crescent = phaname[fraction_of_lunation_to_phase(0.15)];
    assert(strcmp(waxing_crescent, "Waxing Crescent") == 0);

    char* first_quarter = phaname[fraction_of_lunation_to_phase(0.25)];
    assert(strcmp(first_quarter, "First Quarter") == 0);

    char* waxing_gibbous = phaname[fraction_of_lunation_to_phase(0.35)];
    assert(strcmp(waxing_gibbous, "Waxing Gibbous") == 0);

    char* full_moon = phaname[fraction_of_lunation_to_phase(0.5)];
    assert(strcmp(full_moon, "Full Moon") == 0);

    char* waning_gibbous = phaname[fraction_of_lunation_to_phase(0.65)];
    assert(strcmp(waning_gibbous, "Waning Gibbous") == 0);

    char* last_quarter = phaname[fraction_of_lunation_to_phase(0.75)];
    assert(strcmp(last_quarter, "Last Quarter") == 0);

    char* waning_crescent = phaname[fraction_of_lunation_to_phase(0.85)];
    assert(strcmp(waning_crescent, "Waning Crescent") == 0);

    char* new_moon_end = phaname[fraction_of_lunation_to_phase(1)];
    assert(strcmp(new_moon_end, "New Moon") == 0);
}

void test_fraction_of_lunation_to_phase_icon(void) {
    char* new_moon_start = moonicn[fraction_of_lunation_to_phase(0)];
    assert(strcmp(new_moon_start, "🌑") == 0);

    char* waxing_crescent = moonicn[fraction_of_lunation_to_phase(0.15)];
    assert(strcmp(waxing_crescent, "🌒") == 0);

    char* first_quarter = moonicn[fraction_of_lunation_to_phase(0.25)];
    assert(strcmp(first_quarter, "🌓") == 0);

    char* waxing_gibbous = moonicn[fraction_of_lunation_to_phase(0.35)];
    assert(strcmp(waxing_gibbous, "🌔") == 0);

    char* full_moon = moonicn[fraction_of_lunation_to_phase(0.5)];
    assert(strcmp(full_moon, "🌕") == 0);

    char* waning_gibbous = moonicn[fraction_of_lunation_to_phase(0.65)];
    assert(strcmp(waning_gibbous, "🌖") == 0);

    char* last_quarter = moonicn[fraction_of_lunation_to_phase(0.75)];
    assert(strcmp(last_quarter, "🌗") == 0);

    char* waning_crescent = moonicn[fraction_of_lunation_to_phase(0.85)];
    assert(strcmp(waning_crescent, "🌘") == 0);

    char* new_moon_end = moonicn[fraction_of_lunation_to_phase(1)];
    assert(strcmp(new_moon_end, "🌑") == 0);
}

void test_fmt_phase_time_regular(void) {
    struct tm gm = {
        .tm_year = 95,
        .tm_mon = 2,
        .tm_mday = 12,
        .tm_wday = 0,
        .tm_hour = 11,
        .tm_min = 16,
        .tm_sec = 26,
        .tm_isdst = 0,
    };
    char buf[80];

    fmt_phase_time(&gm, buf);

    assert(strcmp(buf, "Sunday    11:16 UTC 12 March 1995") == 0);
}

void test_fmt_phase_time_month_padding(void) {
    struct tm gm = {
        .tm_year = 95,
        .tm_mon = 2,
        .tm_mday = 12,
        .tm_wday = 0,
        .tm_hour = 11,
        .tm_min = 16,
        .tm_sec = 26,
        .tm_isdst = 0,
    };
    char buf[80];

    gm.tm_mon = 4;  // May (shortest)
    fmt_phase_time(&gm, buf);
    assert(strcmp(buf, "Sunday    11:16 UTC 12 May   1995") == 0);

    gm.tm_mon = 8;  // September (longest)
    fmt_phase_time(&gm, buf);
    assert(strcmp(buf, "Sunday    11:16 UTC 12 September 1995") == 0);

    gm.tm_mon = 6;  // July (4 chars = 1 char padding)
    fmt_phase_time(&gm, buf);
    assert(strcmp(buf, "Sunday    11:16 UTC 12 July  1995") == 0);

    gm.tm_mon = 2;  // March (5 chars = exact)
    fmt_phase_time(&gm, buf);
    assert(strcmp(buf, "Sunday    11:16 UTC 12 March 1995") == 0);

    gm.tm_mon = 7;  // August (6 chars = no padding)
    fmt_phase_time(&gm, buf);
    assert(strcmp(buf, "Sunday    11:16 UTC 12 August 1995") == 0);
}

void test_fmt_phase_time_at_boundaries(void) {
    struct tm gm = {
        .tm_year = 95,
        .tm_mon = 2,
        .tm_mday = 12,
        .tm_wday = 0,
        .tm_hour = 11,
        .tm_min = 16,
        .tm_sec = 26,
        .tm_isdst = 0,
    };
    char buf[80];

    gm.tm_wday = 0;  // Sunday
    fmt_phase_time(&gm, buf);
    assert(strcmp(buf, "Sunday    11:16 UTC 12 March 1995") == 0);

    gm.tm_wday = 1;  // Monday
    fmt_phase_time(&gm, buf);
    assert(strcmp(buf, "Monday    11:16 UTC 12 March 1995") == 0);

    gm.tm_wday = 6;  // Saturday
    fmt_phase_time(&gm, buf);
    assert(strcmp(buf, "Saturday  11:16 UTC 12 March 1995") == 0);

    gm.tm_mon = 0;  // January
    fmt_phase_time(&gm, buf);
    assert(strcmp(buf, "Saturday  11:16 UTC 12 January 1995") == 0);

    gm.tm_mon = 11;  // December
    fmt_phase_time(&gm, buf);
    assert(strcmp(buf, "Saturday  11:16 UTC 12 December 1995") == 0);
}

void test_jtime_regular(void) {
    struct tm gm = {
        .tm_year = 95,
        .tm_mon = 2,
        .tm_mday = 11,
        .tm_hour = 1,
        .tm_min = 40,
        .tm_sec = 0,
    };

    double jd = jtime(&gm);

    assert_almost_equal(jd, 2449787.5694444445);
}

void test_jtime_january(void) {
    struct tm gm = {
        .tm_year = 95,
        .tm_mon = 0,
        .tm_mday = 1,
        .tm_hour = 0,
        .tm_min = 0,
        .tm_sec = 0,
    };

    double jd = jtime(&gm);

    assert_almost_equal(jd, 2449718.5);
}

void test_jtime_zero(void) {
    struct tm gm = {
        .tm_year = -6612,
        .tm_mon = 0,
        .tm_mday = 1,
        .tm_hour = 12,
        .tm_min = 0,
        .tm_sec = 0,
    };

    double jd = jtime(&gm);

    assert_almost_equal(jd, 0.0);
}

void test_jtime_negative(void) {
    struct tm gm = {
        .tm_year = -9900,
        .tm_mon = 0,
        .tm_mday = 1,
        .tm_hour = 0,
        .tm_min = 0,
        .tm_sec = 0,
    };

    double jd = jtime(&gm);

    assert_almost_equal(jd, -1200941.5);
}

void test_ucttoj_regular(void) {
    double julian_date = ucttoj(1995, 2, 11, 0, 0, 0);

    assert_almost_equal(julian_date, 2449787.5);
}

void test_ucttoj_month_lte_2(void) {
    double julian_date = ucttoj(1900, 1, 1, 0, 0, 0);

    assert_almost_equal(julian_date, 2415051.5);
}

void test_ucttoj_year_1582(void) {
    double julian_date = ucttoj(1582, 9, 4, 0, 0, 0);

    assert_almost_equal(julian_date, 2299159.5);
}

void test_jtouct_regular(void) {
    struct tm gm;

    jtouct(2438749.732639, &gm);

    assert(gm.tm_year == 64);
    assert(gm.tm_mon == 11);
    assert(gm.tm_mday == 20);
    assert(gm.tm_wday == 0);
    assert(gm.tm_hour == 5);
    assert(gm.tm_min == 35);
    assert(gm.tm_sec == 0);
}

void test_jyear_regular(void) {
    long yy;
    int mm, dd;

    jyear(2460426.09191, &yy, &mm, &dd);

    assert(yy == 2024);
    assert(mm == 4);
    assert(dd == 25);
}

void test_jyear_before_october_15_1582(void) {
    long yy;
    int mm, dd;

    jyear(2299160.0, &yy, &mm, &dd);

    assert(yy == 1582);
    assert(mm == 10);
    assert(dd == 4);
}

void test_jyear_on_october_15_1582(void) {
    long yy;
    int mm, dd;

    jyear(2299160.9, &yy, &mm, &dd);

    assert(yy == 1582);
    assert(mm == 10);
    assert(dd == 15);
}

void test_jhms_regular(void) {
    int h, m, s;

    jhms(2438749.732639, &h, &m, &s);  // P

    assert(h == 5);
    assert(m == 35);
    assert(s == 0);
}

void test_jhms_zero(void) {
    int h, m, s;

    jhms(0.0, &h, &m, &s);

    assert(h == 12);
    assert(m == 0);
    assert(s == 0);
}

void test_jhms_negative(void) {
    int h, m, s;

    jhms(-1200941.5, &h, &m, &s);

    assert(h == 0);
    assert(m == 0);
    assert(s == 0);
}

void test_jwday_regular(void) {
    int wday = jwday(2439913.881944);  // M

    assert(wday == 2);
}

void test_jwday_positive_all_days(void) {
    assert(jwday(2439912.0) == 0);  // Sunday
    assert(jwday(2439913.0) == 1);
    assert(jwday(2439914.0) == 2);
    assert(jwday(2439915.0) == 3);
    assert(jwday(2439916.0) == 4);
    assert(jwday(2439917.0) == 5);
    assert(jwday(2439918.0) == 6);
    assert(jwday(2439919.0) == 0);
}

void test_meanphase_regular(void) {
    double meanph = meanphase(2460381.612639, 1535.0);

    assert_almost_equal(meanph, 2460350.2129780464);
}

void test_truephase_lt_0_01(void) {
    double trueph = truephase(1537.0, 0);

    assert_almost_equal(trueph, 2460409.266218814);
}

void test_truephase_abs_min_0_25_lt_0_01_and_lt_0_5(void) {
    double trueph = truephase(1537.0, 0.25);

    assert_almost_equal(trueph, 2460416.3017252507);
}

void test_truephase_abs_min_0_75_lt_0_01_and_gte_0_5(void) {
    double trueph = truephase(1537.0, 0.75);

    assert_almost_equal(trueph, 2460431.9776856042);
}

void test_phasehunt_regular(void) {
    double phasar[5];

    phasehunt(2449818.3, phasar);

    assert_almost_equal(phasar[0], 2449807.5908233593);
    assert_almost_equal(phasar[1], 2449815.7327970425);
    assert_almost_equal(phasar[2], 2449823.006760471);
    assert_almost_equal(phasar[3], 2449829.6385180936);
    assert_almost_equal(phasar[4], 2449837.2348421547);
}

void test_kepler_regular(void) {
    double ec = kepler(111.615376, 0.016718);

    assert_almost_equal(ec, 1.9635011880995301);
}

void test_phase_regular(void) {
    double cphase, aom, cdist, cangdia, csund, csuang;

    double p = phase(2449818.7, &cphase, &aom, &cdist, &cangdia, &csund, &csuang);

    assert_almost_equal(p, 0.34488787994113507);

    assert_almost_equal(cphase, 0.7807502920288827);
    assert_almost_equal(aom, 10.184742123258882);
    assert_almost_equal(cdist, 389080.0632791394);
    assert_almost_equal(cangdia, 0.5118693474590013);
    assert_almost_equal(csund, 149916135.21839374);
    assert_almost_equal(csuang, 0.5319984336029933);
}

int main(void) {
    // Utils
    test_abs_all();
    test_fixangle_all();
    test_torad_all();
    test_todeg_all();
    test_dsin_all();
    test_dcos_all();
    test_epl_all();

    // Custom API

    test_moonphase_regular();
    test_moonphase_multiple_creations();
    test_moonphase_display();

    test_mooncalendar_regular();
    test_mooncalendar_multiple_creations();
    test_mooncalendar_display();

    // Moon

    test_fraction_of_lunation_to_phase_number();
    test_fraction_of_lunation_to_phase_name();
    test_fraction_of_lunation_to_phase_icon();

    test_fmt_phase_time_regular();
    test_fmt_phase_time_month_padding();
    test_fmt_phase_time_at_boundaries();

    test_jtime_regular();
    test_jtime_january();
    test_jtime_zero();
    test_jtime_negative();

    test_ucttoj_regular();
    test_ucttoj_month_lte_2();
    test_ucttoj_year_1582();

    test_jtouct_regular();

    test_jyear_regular();
    test_jyear_before_october_15_1582();
    test_jyear_on_october_15_1582();

    test_jhms_regular();
    test_jhms_zero();
    test_jhms_negative();

    test_jwday_regular();
    test_jwday_positive_all_days();

    test_meanphase_regular();

    test_truephase_lt_0_01();
    test_truephase_abs_min_0_25_lt_0_01_and_lt_0_5();
    test_truephase_abs_min_0_75_lt_0_01_and_gte_0_5();

    test_phasehunt_regular();

    test_kepler_regular();

    test_phase_regular();

    printf("\x1b[0;92mSuccess! All tests passed.\x1b[0m\n");
}
