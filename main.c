/**
 * Command Line Interface for moon.c.
 *
 * If you need a C++ example, look at the `main.cpp` and `Makefile`
 * files at commit `2df0bdef6d898bff955ea360075c20900af4c025`.
 */

#define _GNU_SOURCE

#include "moon/moon.h"

#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>


void print_help(void);
void for_now(void);
void for_custom_timestamp(const long timestamp);
int is_arg_timestamp(const char* arg);
time_t timestamp_str_to_timestamp(const char* timestamp);
time_t datetime_str_to_timestamp(const char* datetime);

int main(int argc, char** argv) {
    if (argc < 2) {
        for_now();
        return EXIT_SUCCESS;
    }

    const char* arg = argv[1];

    if (strcmp(arg, "--help") == 0 || strcmp(arg, "-h") == 0) {
        print_help();
        return EXIT_SUCCESS;
    }

    time_t timestamp = 0;
    if (is_arg_timestamp(arg)) {
        timestamp = timestamp_str_to_timestamp(arg);
    } else {
        timestamp = datetime_str_to_timestamp(arg);
    }
    for_custom_timestamp(timestamp);

    return 0;
}

void print_help(void) {
    printf("usage: moontool [-h] [] [DATETIME] [±TIMESTAMP]\n\n");
    printf("optional arguments:\n");
    printf("  -h, --help            show this help message and exit\n");
    printf("  []                    without arguments, defaults to now\n");
    printf("  [DATETIME]            universal datetime (e.g., 1994-12-22T13:53:34)\n");
    printf("  [±TIMESTAMP]          Unix timestamp (e.g., 788104414)\n");
}

void for_now(void) {
    printf("\n");
    print_moonphase(NULL);
    printf("\n");
    print_mooncal(NULL);
    printf("\n");
}

void for_custom_timestamp(const long timestamp) {
    MoonPhase mphase;
    moonphase(&mphase, &timestamp);

    MoonCalendar mcal;
    mooncal(&mcal, &timestamp);

    printf("\n");
    print_moonphase(&mphase);
    printf("\n");
    print_mooncal(&mcal);
    printf("\n");
}

int is_digit(const char character) {
    return character >= '0' && character <= '9';
}

int is_arg_timestamp(const char* arg) {
    // [+-]?[0-9]+
    int pos = 0;
    while (*arg) {
        if (pos == 0 && (*arg == '+' || *arg == '-')) {
            // First char can be '+' or '-'.
        } else if (!is_digit(*arg)) {
            return false;
        }
        ++pos;
        ++arg;
    }
    return true;
}

time_t timestamp_str_to_timestamp(const char* timestamp) {
    // The original uses time_t and long interchangeably, so OK here...
    return (time_t) atol(timestamp);
}

time_t datetime_str_to_timestamp(const char* datetime) {
    struct tm gm = {0};
    gm.tm_isdst = 0;  // Explicitly UTC.

    char* conversion;
    if (strchr(datetime, 'T') == NULL)
        conversion = strptime(datetime, "%Y-%m-%d", &gm);
    else
        conversion = strptime(datetime, "%Y-%m-%dT%H:%M:%S", &gm);

    if (conversion == NULL || *conversion != '\0') {
        fprintf(stderr, "Error reading date and time from input.\n");
        exit(EXIT_FAILURE);
    }

    return timegm(&gm);
}
