#ifndef MOON_MOON_H_
#define MOON_MOON_H_

#include <time.h>


#ifdef __cplusplus
namespace moon {
extern "C" {
#endif

typedef struct {
    double julian_date;
    time_t utc_timestamp;
    struct tm* utc_datetime;
    double age_of_moon;
    double fraction_of_lunation;
    int phase;
    double moon_fraction_illuminated;
    double moon_distance_to_earth_km;
    double moon_distance_to_earth_earth_radii;
    double moon_subtends;
    double sun_distance_to_earth_km;
    double sun_distance_to_earth_astronomical_units;
    double sun_subtends;
} MoonPhase;


typedef struct {
    double last_new_moon;
    long lunation;
    double first_quarter;
    double full_moon;
    double last_quarter;
    double next_new_moon;
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

#ifdef __cplusplus
}  // extern "C"
}  // namespace moon
#endif

#endif  // MOON_MOON_H_
