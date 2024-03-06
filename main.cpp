/**
 * Command Line Interface for moon.c.
 */

#include "moon/moon.h"

#include <cstdlib>
#include <cstring>
#include <ctime>
#include <iomanip>
#include <iostream>
#include <regex>
#include <sstream>


using moon::MoonCalendar;
using moon::MoonPhase;
using moon::print_mooncal;
using moon::print_moonphase;


void print_help_and_exit() {
    std::cout << "usage: moontool [-h] [] [DATETIME] [±TIMESTAMP]\n\n";
    std::cout << "optional arguments:\n";
    std::cout << "  -h, --help            show this help message and exit\n";
    std::cout << "  []                    without arguments, defaults to now\n";
    std::cout << "  [DATETIME]            local datetime (e.g., 1994-12-22T14:53:34)\n";
    std::cout << "  [±TIMESTAMP]          Unix Timestamp (e.g., 788104414)\n";
    exit(EXIT_SUCCESS);
}

std::time_t timestamp_str_to_timestamp(const char* timestamp) {
    // The original uses time_t and long interchangeably, so OK here...
    return (std::time_t) std::atol(timestamp);
}

std::time_t datetime_str_to_timestamp(const char* datetime) {
    std::tm tm = {};
    tm.tm_isdst = -1;

    std::istringstream ss(datetime);
    ss >> std::get_time(&tm, "%Y-%m-%dT%H:%M:%S");
    if (ss.fail()) {
        std::cerr << "Error converting input time to timestamp.\n";
        exit(EXIT_FAILURE);
    }

    return std::mktime(&tm);
}

void custom_timestamp(const long timestamp) {
    MoonPhase mphase;
    moonphase(&mphase, &timestamp);

    MoonCalendar mcal;
    mooncal(&mcal, &timestamp);

    std::cout << "\n";
    print_moonphase(&mphase);
    std::cout << "\n";
    print_mooncal(&mcal);
    std::cout << "\n";
}

void now() {
    std::cout << "\n";
    print_moonphase(NULL);
    std::cout << "\n";
    print_mooncal(NULL);
    std::cout << "\n";
}

int main(int argc, char** argv) {
    if (argc >= 2) {
        const char* arg = argv[1];
        std::time_t timestamp = 0;
        if (std::strcmp(arg, "--help") == 0 || std::strcmp(arg, "-h") == 0)
            print_help_and_exit();
        else if (std::regex_match(arg, std::regex("[+-]?[0-9]+")))
            timestamp = timestamp_str_to_timestamp(arg);
        else
            timestamp = datetime_str_to_timestamp(arg);
        custom_timestamp(timestamp);
    } else {
        now();
    }

    return 0;
}
