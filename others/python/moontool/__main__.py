"""Command Line Interface for moon.py."""

import datetime as dt
import sys

from .moon import MoonCalendar, MoonPhase


def main(args: list[str]) -> int:
    if not args:
        for_now()
        return 0

    arg = args.pop(0)

    if arg == "--help" or arg == "-h":
        print_help()
        return 0

    datetime: dt.datetime
    try:
        datetime = try_from_timestamp(arg)
    except ValueError:
        try:
            datetime = try_from_datetime(arg)
        except ValueError:
            print("Error reading date and time from input.", file=sys.stderr)
            return 2

    for_custom_datetime(datetime)

    return 0


def for_now() -> None:
    print()
    print(MoonPhase.now())
    print(MoonCalendar.now())


def print_help() -> None:
    print("usage: moontool [-h] [] [DATETIME] [±TIMESTAMP]\n")
    print("optional arguments:")
    print("  -h, --help            show this help message and exit")
    print("  []                    without arguments, defaults to now")
    print("  [DATETIME]            local datetime (e.g., 1994-12-22T14:53:34+01:00)")
    print("  [±TIMESTAMP]          Unix timestamp (e.g., 788104414)")


def for_custom_datetime(datetime: dt.datetime) -> None:
    print()
    print(MoonPhase.for_datetime(datetime))
    print(MoonCalendar.for_datetime(datetime))


def try_from_timestamp(timestamp: str) -> dt.datetime:
    t = int(timestamp)
    return dt.datetime.fromtimestamp(t, dt.UTC)


def try_from_datetime(datetime: str) -> dt.datetime:
    local = dt.datetime.fromisoformat(datetime)
    return local.astimezone(dt.UTC)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
