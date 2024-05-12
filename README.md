# Moontool

[![license: 0BSD](https://img.shields.io/badge/license-0BSD-blue)](https://opensource.org/license/0BSD)

John Walker's moontool.c astronomical calculation routines, extracted.

## Original

The original versions, [moontool](https://www.fourmilab.ch/moontool/)
and [moontoolw](https://www.fourmilab.ch/moontoolw/), are available at
John Walker's [fourmilab.ch](https://www.fourmilab.ch/).

Backup copies are included in the [`original/`](./original/) directory
of this repository.

<p align="center">
  <img src="original/moontool/moontool.gif" alt="John Walker's Moontool">
</p>

## License

Walker's program is in the public domain, and so is this project.

In Walker's words:

> Do what thou wilt shall be the whole of the law.

## Usage

To use the astronomical calculation routines, include `moon.h` and link
against C's standard math library (`-lm`).

It is C code, but can be used as-is in C++ projects[^1].

If in doubt, take a look at how the CLI is built in the
[`Makefile`](./Makefile).

[^1]:
    A C++ version of the CLI is available at
    [2df0bde](https://github.com/qrichert/moontool/blob/2df0bdef6d898bff955ea360075c20900af4c025/main.cpp).

## CLI

This version includes functions that print out info about the Moon, in a
way similar to the original program.

The CLI wraps these functions.

Run `make` and execute the resulting `moontool` program in the `build/`
directory.

You can run it bare for real-time data, pass it a datetime string or a
Unix timestamp (negative values allowed).

Use `-h` option for help.

To install it, run `make && sudo make install`.

```
$ moontool 1994-12-22T13:53:34

Phase
=====

Julian date:            2449709.07887   (0h variant: 2449709.57887)
Universal time:         Thursday  13:53:34 22 December 1994
Local time:             Thursday  14:53:34 22 December 1994

Age of moon:            18 days, 22 hours, 29 minutes.
Lunation:               64.13%   (🌖 Waning Gibbous)
Moon phase:             81.56%   (0% = New, 100% = Full)

Moon's distance:        386212 kilometres, 60.6 Earth radii.
Moon subtends:          0.5157 degrees.

Sun's distance:         147151251 kilometres, 0.984 astronomical units.
Sun subtends:           0.5420 degrees.

Moon Calendar
=============

Last new moon:          Friday    23:54 UTC  2 December 1994    Lunation: 890
First quarter:          Friday    21:06 UTC  9 December 1994
Full moon:              Sunday     2:18 UTC 18 December 1994
Last quarter:           Sunday    19:07 UTC 25 December 1994
Next new moon:          Sunday    10:56 UTC  1 January 1995     Lunation: 891
```

### Others

This repo [also includes](./others):

- `moonsim.sh` A shell script that simulates a lunation.
- `web.py` A simple Python script that serves the CLI over the web.
