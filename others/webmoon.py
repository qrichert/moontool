#!/usr/bin/env python3

"""The Moontool CLI, served over the web.

First, make sure you've installed the Rust CLI version:

```shell
make && sudo make install
```

Then start the web server:

```shell
# Requires Python >= 3.10
pip install "fastapi[standard]"
fastapi run --port 2222 webmoon.py
```

(or `fastapi dev` instead of `run` for development mode.)

```shell
nohup fastapi run --port 2222 webmoon.py > /dev/null 2>&1 < /dev/null &
```

Now you can query the server like this:

```
http://0.0.0.0:2222/?date=2024-05-28
http://0.0.0.0:2222/?date=2024-05-28T19:16:00
http://0.0.0.0:2222/?date=2024-05-28T19:16:00Z&moon=1
http://0.0.0.0:2222/?graph=1&verbose=1
http://0.0.0.0:2222/docs
```
"""

import json
import subprocess
from typing import Annotated

from fastapi import FastAPI, Query  # pyright: ignore
from fastapi.responses import HTMLResponse  # pyright: ignore

HTML_TEMPLATE: str = """
<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8" />
    <title>Moontool%{DATE}</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <link
      rel="icon"
      type="image/svg+xml"
      href="data:image/svg+xml,%3Csvg%20xmlns='http://www.w3.org/2000/svg'%20viewBox='0%200%2016%2016'%3E%3Ctext%20x='0'%20y='14'%3E%{MOON}%3C/text%3E%3C/svg%3E"
    />
    <style>
      body {
        margin: 0;
        padding: 15px;
        width: 100vw;
        min-height: 100vh;
        box-sizing: border-box;
        color: white;
        background-color: black;
        font-family: monospace;
        display: flex;
        justify-content: center;
        align-items: center;
      }
    </style>
  </head>
  <body>
    <pre>%{OUTPUT}</pre>
  </body>
</html>
"""

app = FastAPI()


def moontool(
    date: str | None,
    verbose: bool,
    do_render_moon: bool,
    do_graph_lunation: bool,
) -> str:
    command: list[str] = ["moontool"]
    if date:
        command.append(date)
    if verbose:
        command.append("--verbose")
    if do_render_moon:
        command.append("--moon")
    if do_graph_lunation:
        command.append("--graph")

    res: subprocess.CompletedProcess = subprocess.run(
        command,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    return ansi_color_codes_to_html(res.stdout)


def ansi_color_codes_to_html(html: str) -> str:
    html = html.replace("\x1b[0;91m", '<span style="color: red;">')
    html = html.replace("\x1b[0m", "</span>")
    return html


def moon_icon(date: str | None) -> str:
    command: list[str] = ["moontool", "--json"]
    if date:
        command.append(date)

    res: subprocess.CompletedProcess = subprocess.run(
        command,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    moon: dict = json.loads(res.stdout)
    return moon["phase"]["phase"]["icon"]


@app.get("/", response_class=HTMLResponse)
async def index(
    date: Annotated[str | None, Query(description="Date in ISO format.")] = None,
    verbose: Annotated[bool, Query(description="Verbose output.")] = False,
    moon: Annotated[bool, Query(description="Render the Moon.")] = False,
    graph: Annotated[bool, Query(description="Graph lunation.")] = False,
) -> str:
    output: str = moontool(date, verbose, moon, graph)
    html: str = HTML_TEMPLATE.replace("%{DATE}", f" - {date}" if date else "")
    html = html.replace("%{OUTPUT}", output)
    html = html.replace("%{MOON}", moon_icon(date))
    return HTMLResponse(content=html, status_code=200)
