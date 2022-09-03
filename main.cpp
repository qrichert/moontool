#include "moon/moon.h"

#include <cstdlib>


using moon::MoonCalendar;
using moon::MoonPhase;
using moon::print_mooncal;
using moon::print_moonphase;


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
        long timestamp = std::atol(argv[1]);
        custom_timestamp(timestamp);
    } else {
        now();
    }

    return 0;
}
