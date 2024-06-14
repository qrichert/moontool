#ifndef MOON_MOON_H_
#define MOON_MOON_H_

#include <time.h>


#ifdef __cplusplus
namespace moon {
extern "C" {
#endif

/**
 * Information about the phase of the Moon at given time.
 *
 * Examples:
 *
 * ```c
 * #include "moon.h"
 *
 * MoonPhase mphase;
 * time_t timestamp = 1714809600;
 *
 * moonphase(&mphase, &timestamp);
 *
 * assert(strcmp(mphase.phase_name, "Waning Crescent") == 0);
 * ```
 */
typedef struct {
    double julian_date;
    time_t timestamp;
    struct tm utc_datetime;
    double age;
    double fraction_of_lunation;
    int phase;
    char* phase_name;
    char* phase_icon;
    double fraction_illuminated;
    double distance_to_earth_km;
    double distance_to_earth_earth_radii;
    /**
     * Angular diameter.
     */
    double subtends;
    double sun_distance_to_earth_km;
    double sun_distance_to_earth_astronomical_units;
    /**
     * Sun's angular diameter.
     */
    double sun_subtends;
} MoonPhase;


/**
 * Information about past and future Moons, around given time.
 *
 * Note:
 *
 * last_new_moon`, `first_quarter`, `full_moon`, `last_quarter`, and
 * `next_new_moon`, are Julian Day Numbers (JDN)[^jdn].
 *
 * [^jdn]: https://en.wikipedia.org/wiki/Julian_day
 *
 * Examples:
 *
 * ```c
 * #include "moon.h"
 *
 * MoonCalendar mcal;
 * time_t timestamp = 1714809600;
 *
 * mooncal(&mcal, &timestamp);
 *
 * assert(mcal.lunation == 1253);
 * ```
 */
typedef struct {
    double julian_date;
    time_t timestamp;
    struct tm utc_datetime;
    /**
     * Brown Lunation Number (BLN). Numbering begins at the first
     * New Moon of 1923 (17 January 1923 at 2:41 UTC).
     */
    long lunation;
    double last_new_moon;
    struct tm last_new_moon_utc;
    double first_quarter;
    struct tm first_quarter_utc;
    double full_moon;
    struct tm full_moon_utc;
    double last_quarter;
    struct tm last_quarter_utc;
    double next_new_moon;
    struct tm next_new_moon_utc;
} MoonCalendar;


/**
 * Populate MoonPhase struct with info about the Moon at given time.
 *
 * @param mphase The MoonPhase struct.
 * @param timestamp Time of snapshot; if NULL, current UTC time is used.
 * @return 1 (true) = OK, 0 (false) = KO.
 */
int moonphase(MoonPhase* mphase, const time_t* timestamp);

/**
 * Print MoonPhase object or print info at current time.
 *
 * @param mphase Struct to print; if NULL, current UTC time is used.
 */
void print_moonphase(const MoonPhase* mphase);

/**
 * Print raw MoonPhase object or print info at current time.
 *
 * @param mphase Struct to print; if NULL, current UTC time is used.
 */
void print_moonphase_debug(const MoonPhase* mphase);

/**
 * Populate MoonCalendar struct with info about lunation at given time.
 *
 * @param mcal The MoonCalendar struct.
 * @param timestamp Time of snapshot; if NULL, current UTC time is used.
 * @return 1 (true) = OK, 0 (false) = KO.
 */
int mooncal(MoonCalendar* mcal, const time_t* timestamp);

/**
 * Print MoonCalendar object or print info at current time.
 *
 * @param mcal Struct to print; if NULL, current UTC time is used.
 */
void print_mooncal(const MoonCalendar* mcal);

/**
 * Print raw MoonCalendar object or print info at current time.
 *
 * @param mcal Struct to print; if NULL, current UTC time is used.
 */
void print_mooncal_debug(const MoonCalendar* mcal);

#ifdef __cplusplus
}  // extern "C"
}  // namespace moon
#endif

#endif  // MOON_MOON_H_
