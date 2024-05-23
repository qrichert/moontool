#!/usr/bin/env bash

# Simulate a whole lunation (lunar month).
# New Moon -> Full Moon -> New Moon

# Arguments are forwarded, this enables `./moonsim.sh --moon` et al.

START=794058669
END=796616119
DAY_QUARTER=21600

EXECUTABLE=./rust/target/release/moontool

CLEAR_SEQUENCE='\033[2J\033[1;1H'
HIDE_TEXT_CURSOR_SEQUENCE='\033[?25l'
SHOW_TEXT_CURSOR_SEQUENCE='\033[?25h'

if [[ ! -f $EXECUTABLE ]]; then
    echo "Moontool doesn't look like it's been built."
    echo "Run 'make' and try again!"
    exit 1
fi

clear
printf $HIDE_TEXT_CURSOR_SEQUENCE

for i in {0..119}; do
    t=$(($START + $DAY_QUARTER * $i))
    if [[ $t -gt $END ]]; then
        t=$END
    fi
    printf $CLEAR_SEQUENCE
    $EXECUTABLE $t $@
    sleep 0.25
done

printf $SHOW_TEXT_CURSOR_SEQUENCE
