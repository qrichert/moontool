#!/usr/bin/env bash

START=794058669
END=796616119
DAY_QUARTER=21600

if [[ ! -f ./build/moontool ]]; then
    echo "Moontool doesn't look like it's been built."
    echo "Run 'make' and try again!"
    exit 1
fi

for i in {0..119}; do
    t=$(($START + $DAY_QUARTER * $i))
    if [[ $t -gt $END ]]; then
        t=$END
    fi
    clear
    ./build/moontool $t
    sleep 0.25
done
