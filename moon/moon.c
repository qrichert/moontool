/*

            A Moon for C

    John Walker's moontool.c calculation routines, extracted.

        John Walker
        http://www.fourmilab.ch/
        https://fourmilab.ch/moontool/
        https://fourmilab.ch/moontoolw/

    This program is in the public domain, and in what seems to be the
    tradition of Moontool:

        "Do what thou wilt shall be the whole of the law".

    Several versions of the tool can be found, including the original
    Sun Workstation version  (moontool),  the X Window System version
    (xmoontool),  and two  Windows versions  (moontoolw),  for 16 and
    32-bit architectures.

    The  major part of the code comes from the MOONCALC.C file,  from
    moontoolw's 32-bit version (the most recent).

    Great care has been taken  to extract the relevant functions from
    the  original program.  The  code has been copied  with as little
    change as possible. Not even the formatting has been touched, and
    the original author's style  has been matched as well as possible
    where edits were necessary.

*/

// clang-format off

#include "moon.h"

#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>


#define TRUE        1
#define FALSE       0

/*  Astronomical constants  */

#define epoch       2444238.5      /* 1980 January 0.0 */

/*  Constants defining the Sun's apparent orbit  */

#define elonge      278.833540     /* Ecliptic longitude of the Sun
                                      at epoch 1980.0 */
#define elongp      282.596403     /* Ecliptic longitude of the Sun at
                                      perigee */
#define eccent      0.016718       /* Eccentricity of Earth's orbit */
#define sunsmax     1.495985e8     /* Semi-major axis of Earth's orbit, km */
#define sunangsiz   0.533128       /* Sun's angular size, degrees, at
                                      semi-major axis distance */

/*  Elements of the Moon's orbit, epoch 1980.0  */

#define mmlong      64.975464      /* Moon's mean longitude at the epoch */
#define mmlongp     349.383063     /* Mean longitude of the perigee at the
                                      epoch */
#define mlnode      151.950429     /* Mean longitude of the node at the
                                      epoch */
#define minc        5.145396       /* Inclination of the Moon's orbit */
#define mecc        0.054900       /* Eccentricity of the Moon's orbit */
#define mangsiz     0.5181         /* Moon's angular size at distance a
                                      from Earth */
#define msmax       384401.0       /* Semi-major axis of Moon's orbit in km */
#define mparallax   0.9507         /* Parallax at distance a from Earth */
#define synmonth    29.53058868    /* Synodic month (new Moon to new Moon) */
#define lunatbase   2423436.0      /* Base date for E. W. Brown's numbered
                                      series of lunations (1923 January 16) */

/*  Properties of the Earth  */

#define earthrad    6378.16        /* Radius of Earth in kilometres */

#define PI 3.14159265358979323846  /* Assume not near black hole nor in
                                      Tennessee */

/*  Handy mathematical functions  */

#define abs(x) ((x) < 0 ? (-(x)) : (x))                   /* Absolute val */
#define fixangle(a) ((a) - 360.0 * (floor((a) / 360.0)))  /* Fix angle    */
#define torad(d) ((d) * (PI / 180.0))                     /* Deg->Rad     */
#define todeg(d) ((d) * (180.0 / PI))                     /* Rad->Deg     */
#define dsin(x) (sin(torad((x))))                         /* Sin from deg */
#define dcos(x) (cos(torad((x))))                         /* Cos from deg */

static char *moname[] = {
    "January", "February", "March", "April", "May",
    "June", "July", "August", "September",
    "October", "November", "December"
};

static char *dayname[] = {
    "Sunday", "Monday", "Tuesday", "Wednesday",
    "Thursday", "Friday", "Saturday"
};

static char *phaname[] = {
    "New Moon", "Waxing Crescent", "First Quarter",
    "Waxing Gibbous", "Full Moon", "Waning Gibbous",
    "Last Quarter", "Waning Crescent"
};

static char *moonicn[] = {
    "\U0001f311", "\U0001f312", "\U0001f313",  // 🌑 🌒 🌓
    "\U0001f314", "\U0001f315", "\U0001f316",  // 🌔 🌕 🌖
    "\U0001f317", "\U0001f318"                 // 🌗 🌘
};

/*  Forward functions  */

static void fmt_phase_time(double utime, char *buf);
static double jtime(struct tm *t);
static void jyear(double td, long *yy, int *mm, int *dd);
static void jhms(double j, int *h, int *m, int *s);
static int jwday(double j);
static double ucttoj(long year, int mon, int mday, int hour, int min, int sec);
static void phasehunt(double sdate, double phases[5]);
static double phase(double pdate, double *pphase, double *mage, double *dist,
                    double *angdia, double *sudist, double *suangdia);

/* Custom API */

static int fraction_of_lunation_to_phase(double p)
{
    const double day_frac = (1 / synmonth) * 0.75;

    if (p < 0.00 + day_frac)
        return 0;
    if (p < 0.25 - day_frac)
        return 1;
    if (p < 0.25 + day_frac)
        return 2;
    if (p < 0.50 - day_frac)
        return 3;
    if (p < 0.50 + day_frac)
        return 4;
    if (p < 0.75 - day_frac)
        return 5;
    if (p < 0.75 + day_frac)
        return 6;
    if (p < 1.00 - day_frac)
        return 7;
    return 0;
}

int moonphase(MoonPhase *mphase, const time_t *timestamp)
{
    long t;  // Original implementation casts time()'s time_t to a long.
    double jd, p, aom, cphase, cdist, cangdia, csund, csuang;
    struct tm *gm;

    if (timestamp != NULL)
        t = *timestamp;
    else
        t = time(NULL);

    gm = gmtime(&t);
    if (gm == NULL)
        return FALSE;  // Check errno for error (set by gmtime()).

    jd = jtime(gm);

    p = phase(jd, &cphase, &aom, &cdist, &cangdia, &csund, &csuang);

    mphase->julian_date = jd;
    mphase->utc_timestamp = (time_t) t;
    mphase->utc_datetime = gm;
    mphase->age_of_moon = aom;
    mphase->fraction_of_lunation = p;
    mphase->phase = fraction_of_lunation_to_phase(p);
    mphase->moon_fraction_illuminated = cphase;
    mphase->moon_distance_to_earth_km = cdist;
    mphase->moon_distance_to_earth_earth_radii = cdist / earthrad;
    mphase->moon_subtends = cangdia;
    mphase->sun_distance_to_earth_km = csund;
    mphase->sun_distance_to_earth_astronomical_units = csund / sunsmax;
    mphase->sun_subtends = csuang;

    return TRUE;
}

static int init_moonphase(MoonPhase *mphase)
{
    return moonphase(mphase, NULL);
}

#define EPL(x) (x), (x) == 1 ? "" : "s"

void print_moonphase(const MoonPhase *mphase)
{
    MoonPhase p;
    if (mphase == NULL) {
        if (!init_moonphase((MoonPhase*) (mphase = &p))) {
            printf("Error computing info about the phase of the Moon.\n");
            exit(EXIT_FAILURE);
        }
    }

    double aom;
    int aom_d, aom_h, aom_m, aom_s;

    aom = mphase->age_of_moon;
    aom_d = (int) aom;
    aom_h = (int) (24 * (aom - floor(aom)));
    aom_m = (int) (1440 * (aom - floor(aom))) % 60;
    aom_s = (int) (86400 * (aom - floor(aom))) % 60;

    struct tm *gm = mphase->utc_datetime;

    printf("Phase\n=====\n\n");
    printf(
        "Julian date:\t\t%.5f   (0h variant: %.5f)\n",
        mphase->julian_date,
        mphase->julian_date + 0.5
    );
    printf(
        "Universal time:\t\t%-9s %2d:%02d:%02d %2d %s %d\n",
        dayname[gm->tm_wday],
        gm->tm_hour,
        gm->tm_min,
        gm->tm_sec,
        gm->tm_mday,
        moname[gm->tm_mon],
        gm->tm_year + 1900
    );
    gm = localtime(&mphase->utc_timestamp);
    printf(
        "Local time:\t\t%-9s %2d:%02d:%02d %2d %s %d\n\n",
        dayname[gm->tm_wday],
        gm->tm_hour,
        gm->tm_min,
        gm->tm_sec,
        gm->tm_mday,
        moname[gm->tm_mon],
        gm->tm_year + 1900
    );
    printf(
        "Age of moon:\t\t%d day%s, %d hour%s, %d minute%s, %d second%s.\n",
        EPL(aom_d),
        EPL(aom_h),
        EPL(aom_m),
        EPL(aom_s)
    );
    printf(
        "Lunation:\t\t%.2f%%   (%s %s)\n",
        mphase->fraction_of_lunation * 100,
        moonicn[mphase->phase],
        phaname[mphase->phase]
    );
    printf(
        "Moon phase:\t\t%.2f%%   (0%% = New, 100%% = Full)\n\n",
        mphase->moon_fraction_illuminated * 100
    );

    printf(
        "Moon's distance:\t%ld kilometres, %.1f Earth radii.\n",
        (long) mphase->moon_distance_to_earth_km,
        mphase->moon_distance_to_earth_earth_radii
    );
    printf("Moon subtends:\t\t%.4f degrees.\n\n", mphase->moon_subtends);

    printf(
        "Sun's distance:\t\t%.0f kilometres, %.3f astronomical units.\n",
        mphase->sun_distance_to_earth_km,
        mphase->sun_distance_to_earth_astronomical_units
    );
    printf("Sun subtends:\t\t%.4f degrees.\n", mphase->sun_subtends);
}

int mooncal(MoonCalendar *mcal, const time_t *timestamp)
{
    long lunation;
    long t;
    double jd;
    double phasar[5];
    struct tm *gm;

    if (timestamp != NULL)
        t = *timestamp;
    else
        t = time(NULL);

    gm = gmtime(&t);
    if (gm == NULL)
        return FALSE;  // Check errno for error (set by gmtime()).

    jd = jtime(gm);

    phasehunt(jd + 0.5, phasar);
    lunation = (long) floor(((phasar[0] + 7) - lunatbase) / synmonth) + 1;

    mcal->last_new_moon = phasar[0];
    mcal->lunation = lunation;
    mcal->first_quarter = phasar[1];
    mcal->full_moon = phasar[2];
    mcal->last_quarter = phasar[3];
    mcal->next_new_moon = phasar[4];

    return TRUE;
}

static int init_mooncal(MoonCalendar *mcal)
{
    return mooncal(mcal, NULL);
}

void print_mooncal(const MoonCalendar *mcal)
{
    MoonCalendar c;
    if (mcal == NULL) {
        if (!init_mooncal((MoonCalendar*) (mcal = &c))) {
            printf("Error computing the Moon calendar.\n");
            exit(EXIT_FAILURE);
        }
    }

    char tbuf[80];

    printf("Moon Calendar\n=============\n\n");
    fmt_phase_time(mcal->last_new_moon, tbuf);
    printf("Last new moon:\t\t%s\tLunation: %ld\n", tbuf, mcal->lunation);
    fmt_phase_time(mcal->first_quarter, tbuf);
    printf("First quarter:\t\t%s\n", tbuf);
    fmt_phase_time(mcal->full_moon, tbuf);
    printf("Full moon:\t\t%s\n", tbuf);
    fmt_phase_time(mcal->last_quarter, tbuf);
    printf("Last quarter:\t\t%s\n", tbuf);
    fmt_phase_time(mcal->next_new_moon, tbuf);
    printf("Next new moon:\t\t%s\tLunation: %ld\n", tbuf, mcal->lunation + 1);
}

/* Original Astronomical Calculation Routines */

/*  FMT_PHASE_TIME  --  Format  the  provided  julian  date  into  the
                        provided  buffer  in  UTC  format  for  screen
                        display  */

static void fmt_phase_time(double utime, char *buf)
{
    long yy;
    int wday, mm, dd, hh, mmm, ss;

    wday = jwday(utime);
    jyear(utime, &yy, &mm, &dd);
    jhms(utime, &hh, &mmm, &ss);
    sprintf(buf, "%-9s %2d:%02d UTC %2d %s %ld",
    dayname [wday], hh, mmm, dd, moname [mm - 1], yy);
}

/*  JTIME  --  Convert a Unix date and time (tm) structure to astronomical
               Julian time (i.e. Julian date plus day fraction,
               expressed as a double).  */

static double jtime(struct tm *t)
{
    return ucttoj(t->tm_year + 1900, t->tm_mon, t->tm_mday, t->tm_hour, t->tm_min, t->tm_sec);
}

/*  UCTTOJ  --  Convert GMT date and time to astronomical
                Julian time (i.e. Julian date plus day fraction,
                expressed as a double).  */

static double ucttoj(long year, int mon, int mday,
                     int hour, int min, int sec)
{

    // Algorithm as given in Meeus, Astronomical Algorithms, Chapter 7, page 61

    int a, b, m;
    long y;

#ifdef PARANOID
    assert(mon  >= 0 && mon  < 12);
    assert(mday >  0 && mday < 32);
    assert(hour >= 0 && hour < 24);
    assert(min  >= 0 && min  < 60);
    assert(sec  >= 0 && sec  < 60);
#endif

    m = mon + 1;
    y = year;

    if (m <= 2) {
        y--;
        m += 12;
    }

    /* Determine whether date is in Julian or Gregorian calendar based on
       canonical date of calendar reform. */

    if ((year < 1582) || ((year == 1582) && ((mon < 9) || (mon == 9 && mday < 5)))) {
        b = 0;
    } else {
        a = ((int) (y / 100));
        b = 2 - a + (a / 4);
    }

    return (((long) (365.25 * (y + 4716))) + ((int) (30.6001 * (m + 1))) +
                mday + b - 1524.5) +
            ((sec + 60L * (min + 60L * hour)) / 86400.0);
}

/*  JYEAR  --  Convert    Julian    date  to  year,  month, day, which are
               returned via integer pointers to integers (note that year is a long).  */

static void jyear(double td, long *yy, int *mm, int *dd)
{
    double z, f, a, alpha, b, c, d, e;

    td += 0.5;
    z = floor(td);
    f = td - z;

    if (z < 2299161.0) {
        a = z;
    } else {
        alpha = floor((z - 1867216.25) / 36524.25);
        a = z + 1 + alpha - floor(alpha / 4);
    }

    b = a + 1524;
    c = floor((b - 122.1) / 365.25);
    d = floor(365.25 * c);
    e = floor((b - d) / 30.6001);

    *dd = (int) (b - d - floor(30.6001 * e) + f);
    *mm = (int) ((e < 14) ? (e - 1) : (e - 13));
    *yy = (long) ((*mm > 2) ? (c - 4716) : (c - 4715));
}

/*  JHMS  --  Convert Julian time to hour, minutes, and seconds.  */

static void jhms(double j, int *h, int *m, int *s)
{
    long ij;

    j += 0.5;                 /* Astronomical to civil */
    ij = (long) (((j - floor(j)) * 86400.0) + 0.5);  // Round to nearest second
    *h = (int) (ij / 3600L);
    *m = (int) ((ij / 60L) % 60L);
    *s = (int) (ij % 60L);
}

/*  JWDAY  --  Determine day of the week for a given Julian day.  */

static int jwday(double j)
{
    return ((int) (j + 1.5)) % 7;
}

/*  MEANPHASE  --  Calculates  time  of  the mean new Moon for a given
                   base date.  This argument K to this function is the
                   precomputed synodic month index, given by:

                          K = (year - 1900) * 12.3685

                   where year is expressed as a year and fractional year.  */

static double meanphase(double sdate, double k)
{
    double t, t2, t3, nt1;

    /* Time in Julian centuries from 1900 January 0.5 */
    t = (sdate - 2415020.0) / 36525;
    t2 = t * t;                       /* Square for frequent use */
    t3 = t2 * t;                      /* Cube for frequent use */

    nt1 = 2415020.75933 + synmonth * k
            + 0.0001178 * t2
            - 0.000000155 * t3
            + 0.00033 * dsin(166.56 + 132.87 * t - 0.009173 * t2);

    return nt1;
}

/*  TRUEPHASE  --  Given a K value used to determine the mean phase of
                   the new moon, and a phase selector (0.0, 0.25, 0.5,
                   0.75), obtain the true, corrected phase time.  */

static double truephase(double k, double phase)
{
    double t, t2, t3, pt, m, mprime, f;
    int apcor = FALSE;

    k += phase;                       /* Add phase to new moon time */
    t = k / 1236.85;                  /* Time in Julian centuries from
                                         1900 January 0.5 */
    t2 = t * t;                       /* Square for frequent use */
    t3 = t2 * t;                      /* Cube for frequent use */
    pt = 2415020.75933                /* Mean time of phase */
         + synmonth * k
         + 0.0001178 * t2
         - 0.000000155 * t3
         + 0.00033 * dsin(166.56 + 132.87 * t - 0.009173 * t2);

    m = 359.2242                      /* Sun's mean anomaly */
        + 29.10535608 * k
        - 0.0000333 * t2
        - 0.00000347 * t3;
    mprime = 306.0253                 /* Moon's mean anomaly */
        + 385.81691806 * k
        + 0.0107306 * t2
        + 0.00001236 * t3;
    f = 21.2964                       /* Moon's argument of latitude */
        + 390.67050646 * k
        - 0.0016528 * t2
        - 0.00000239 * t3;
    if ((phase < 0.01) || (abs(phase - 0.5) < 0.01)) {

       /* Corrections for New and Full Moon */

       pt +=     (0.1734 - 0.000393 * t) * dsin(m)
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
                + 0.0005 * dsin(m + 2 * mprime);
       apcor = TRUE;
    } else if ((abs(phase - 0.25) < 0.01 || (abs(phase - 0.75) < 0.01))) {
       pt +=     (0.1721 - 0.0004 * t) * dsin(m)
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
                - 0.0003 * dsin(2 * m + mprime);
       if (phase < 0.5)
          /* First quarter correction */
          pt += 0.0028 - 0.0004 * dcos(m) + 0.0003 * dcos(mprime);
       else
          /* Last quarter correction */
          pt += -0.0028 + 0.0004 * dcos(m) - 0.0003 * dcos(mprime);
       apcor = TRUE;
    }
    if (!apcor) {
        fprintf(stderr,
                "TRUEPHASE called with invalid phase selector.\n");
        exit(EXIT_FAILURE);
    }
    return pt;
}

/*   PHASEHUNT  --  Find time of phases of the moon which surround the
                    current date.  Five phases are found, starting and
                    ending with the new moons which bound the  current
                    lunation.  */

static void phasehunt(double sdate, double phases[5])
{
    double adate, k1, k2, nt1, nt2;
    long yy;
    int mm, dd;

    adate = sdate - 45;

    jyear(adate, &yy, &mm, &dd);
    k1 = floor((yy + ((mm - 1) * (1.0 / 12.0)) - 1900) * 12.3685);

    adate = nt1 = meanphase(adate, k1);
    while (TRUE) {
        adate += synmonth;
        k2 = k1 + 1;
        nt2 = meanphase(adate, k2);
        if (nt1 <= sdate && nt2 > sdate)
            break;
        nt1 = nt2;
        k1 = k2;
    }
    phases[0] = truephase(k1, 0.0);
    phases[1] = truephase(k1, 0.25);
    phases[2] = truephase(k1, 0.5);
    phases[3] = truephase(k1, 0.75);
    phases[4] = truephase(k2, 0.0);
}

/*  KEPLER  --   Solve the equation of Kepler.  */

static double kepler(double m, double ecc)
{
    double e, delta;
#define EPSILON 1E-6

    e = m = torad(m);
    do {
        delta = e - ecc * sin(e) - m;
        e -= delta / (1 - ecc * cos(e));
    } while (abs(delta) > EPSILON);
    return e;
}

/*  PHASE  --  Calculate phase of moon as a fraction:

    The  argument  is  the  time  for  which  the  phase is requested,
    expressed as a Julian date and fraction.  Returns  the  terminator
    phase  angle  as a percentage of a full circle (i.e., 0 to 1), and
    stores into pointer arguments  the  illuminated  fraction  of  the
    Moon's  disc, the Moon's age in days and fraction, the distance of
    the Moon from the centre of the Earth, and  the  angular  diameter
    subtended  by the Moon as seen by an observer at the centre of the
    Earth.
*/

static double phase(
  double  pdate,                      /* Date for which to calculate phase */
  double  *pphase,                    /* Illuminated fraction */
  double  *mage,                      /* Age of moon in days */
  double  *dist,                      /* Distance in kilometres */
  double  *angdia,                    /* Angular diameter in degrees */
  double  *sudist,                    /* Distance to Sun */
  double  *suangdia)                  /* Sun's angular diameter */
{

    double Day, N, M, Ec, Lambdasun, ml, MM, MN, Ev, Ae, A3, MmP,
           mEc, A4, lP, V, lPP, NP, y, x, Lambdamoon, BetaM,
           MoonAge, MoonPhase,
           MoonDist, MoonDFrac, MoonAng, MoonPar,
           F, SunDist, SunAng;

    /* Calculation of the Sun's position */

    Day = pdate - epoch;                    /* Date within epoch */
    N = fixangle((360 / 365.2422) * Day);   /* Mean anomaly of the Sun */
    M = fixangle(N + elonge - elongp);      /* Convert from perigee
                                               co-ordinates to epoch 1980.0 */
    Ec = kepler(M, eccent);                 /* Solve equation of Kepler */
    Ec = sqrt((1 + eccent) / (1 - eccent)) * tan(Ec / 2);
    Ec = 2 * todeg(atan(Ec));               /* True anomaly */
    Lambdasun = fixangle(Ec + elongp);      /* Sun's geocentric ecliptic
                                               longitude */
    /* Orbital distance factor */
    F = ((1 + eccent * cos(torad(Ec))) / (1 - eccent * eccent));
    SunDist = sunsmax / F;                  /* Distance to Sun in km */
    SunAng = F * sunangsiz;                 /* Sun's angular size in degrees */

    /* Calculation of the Moon's position */

    /* Moon's mean longitude */
    ml = fixangle(13.1763966 * Day + mmlong);

    /* Moon's mean anomaly */
    MM = fixangle(ml - 0.1114041 * Day - mmlongp);

    /* Moon's ascending node mean longitude */
    MN = fixangle(mlnode - 0.0529539 * Day);

    /* Evection */
    Ev = 1.2739 * sin(torad(2 * (ml - Lambdasun) - MM));

    /* Annual equation */
    Ae = 0.1858 * sin(torad(M));

    /* Correction term */
    A3 = 0.37 * sin(torad(M));

    /* Corrected anomaly */
    MmP = MM + Ev - Ae - A3;

    /* Correction for the equation of the centre */
    mEc = 6.2886 * sin(torad(MmP));

    /* Another correction term */
    A4 = 0.214 * sin(torad(2 * MmP));

    /* Corrected longitude */
    lP = ml + Ev + mEc - Ae + A4;

    /* Variation */
    V = 0.6583 * sin(torad(2 * (lP - Lambdasun)));

    /* True longitude */
    lPP = lP + V;

    /* Corrected longitude of the node */
    NP = MN - 0.16 * sin(torad(M));

    /* Y inclination coordinate */
    y = sin(torad(lPP - NP)) * cos(torad(minc));

    /* X inclination coordinate */
    x = cos(torad(lPP - NP));

    /* Ecliptic longitude */
    Lambdamoon = todeg(atan2(y, x));
    Lambdamoon += NP;

    /* Ecliptic latitude */
    BetaM = todeg(asin(sin(torad(lPP - NP)) * sin(torad(minc))));

    /* Calculation of the phase of the Moon */

    /* Age of the Moon in degrees */
    MoonAge = lPP - Lambdasun;

    /* Phase of the Moon */
    MoonPhase = (1 - cos(torad(MoonAge))) / 2;

    /* Calculate distance of moon from the centre of the Earth */

    MoonDist = (msmax * (1 - mecc * mecc)) /
               (1 + mecc * cos(torad(MmP + mEc)));

    /* Calculate Moon's angular diameter */

    MoonDFrac = MoonDist / msmax;
    MoonAng = mangsiz / MoonDFrac;

    /* Calculate Moon's parallax */

    MoonPar = mparallax / MoonDFrac;

    // TODO: Unused variables.
    (void) Lambdamoon;
    (void) BetaM;
    (void) MoonPar;

    *pphase = MoonPhase;
    *mage = synmonth * (fixangle(MoonAge) / 360.0);
    *dist = MoonDist;
    *angdia = MoonAng;
    *sudist = SunDist;
    *suangdia = SunAng;
    return fixangle(MoonAge) / 360.0;
}
