# Moontool

John Walker's moontool.c astronomical calculation routines, extracted.

## Original

The original versions, [moontool](https://www.fourmilab.ch/moontool/)
and [moontoolw](https://www.fourmilab.ch/moontoolw/), are available at
John Walker's [fourmilab.ch](https://www.fourmilab.ch/).

Backup copies are included in the [`original/`](./original/) directory
of this repository.

<p align="center">
  <a href="#" target="_blank">
    <img src="original/moontool/moontool.gif" alt="John Walker's Moontool">
  </a>
</p>

## License

Walker's program is in the public domain, and so is this project.

In Walker's words:

> Do what thou wilt shall be the whole of the law.

## Usage

To use the astronomical calculation routines, include `moon.h` and link
against C's standard math library (`-lm`).

It is C code, but can be used as-is in C++ projects.

If in doubt, take a look at how the demo is built in the
[`Makefile`](./Makefile).

## Demo

This version includes functions that print out info about the Moon in a
way similar to the original program.

The demo does just that.

Run `make` and execute the resulting `moontool` program in the `build/`
directory.

You can run it bare for real-time data, pass it a datetime string or a
Unix timestamp (negative values allowed).

Use `-h` option for help.

To install it, run `make && sudo make install`.

```
$ moontool 1994-12-22T14:53:34

Phase
=====

Julian date:            2449709.07887   (0h variant: 2449709.57887)
Universal time:         Thursday  13:53:34 22 December 1994
Local time:             Thursday  14:53:34 22 December 1994

Age of moon:            18 days, 22 hours, 29 minutes, 55 seconds.
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

### Web

To run the demo as a web app, make sure you've installed it as described
above, then run:

```shell
# Requires Python >= 3.7
python web.py [--help] [--port 2222]
```

To run it in the background, without worrying about the log:

```shell
nohup python web.py > /dev/null 2>&1 < /dev/null &
```
