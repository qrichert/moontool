#!/usr/bin/env python3

"""Export the Full Moons of a given year to Crontab format.

First, make sure you've installed the Rust CLI version:

```shell
cd others/rust
make && sudo make install
```

Then you can do:

```shell
# Requires Python >= 3.9
python cronfullmoons.py [year]
```
"""

import sys
import subprocess
import json
import datetime as dt

DAY: list[str] = ["MON", "TUE", "WED", "THU", "FRI", "SAT", "SUN"]


def moontool(year: int) -> str:
    command = ["moontool", f"{year:0>4}-01-01", "--json", "--verbose"]
    res: subprocess.CompletedProcess = subprocess.run(
        command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True
    )
    return res.stdout


def main(argv: list[str]) -> None:
    year: int = int(argv.pop()) if argv else dt.datetime.now().year
    data = json.loads(moontool(year))
    full_moons: list[dict] = data["yearly_calendar"]["full_moons"]

    print(f"### Full Moons of {year}\n")
    for full_moon in full_moons:
        name = full_moon["name"]
        print(f"## {name}.")
        d = dt.datetime.fromisoformat(full_moon["date_utc"])
        print(f"{d.minute} {d.hour} {d.day} {d.month} {DAY[d.weekday()]} <command>")


if __name__ == "__main__":
    main(sys.argv[1:])
