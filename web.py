#!/usr/bin/env python3

"""The Moontool CLI served over the web."""

import argparse
import subprocess
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib import parse

HTML_TEMPLATE: str = """
<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8" />
    <title>Moontool%{DATE}</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <style>
      body {
        margin: 0;
        padding: 0;
        width: 100vw;
        height: 100vh;
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


def parse_args() -> argparse.Namespace:
    parser: argparse.ArgumentParser = argparse.ArgumentParser()
    parser.add_argument(
        "-p",
        "--port",
        type=int,
        help="bind web server to given port (default: 2222)",
        dest="port",
        default=2222,
    )
    return parser.parse_args()


def moontool(date: str) -> str:
    command: list[str] = ["moontool"]
    if date:
        command.append(date)
    res: bytes = subprocess.run(
        command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True
    )
    return res.stdout


def serve(port: int) -> None:
    class MoonServer(BaseHTTPRequestHandler):
        def do_GET(self):
            url: parse.ParseResult = parse.urlparse(self.path)
            query: dict = parse.parse_qs(url.query)
            if url.path == "/":
                date: list = query.get("date") or query.get("d") or []
                date: str = date[0] if date else ""
                self.index(date)
            else:
                self.error()

        def index(self, date: str) -> None:
            self.send_response(200)
            self.send_header("Content-type", "text/html")
            self.end_headers()
            output: str = moontool(date)
            html: str = HTML_TEMPLATE
            html = html.replace("%{DATE}", f" - {date}" if date else "")
            html = html.replace("%{OUTPUT}", output)
            self.wfile.write(bytes(html, "utf-8"))

        def error(self) -> None:
            self.send_response(404)
            self.end_headers()

    try:
        print(f"Serving on http://0.0.0.0:{port}")
        moon_server = HTTPServer(("0.0.0.0", port), MoonServer)
        moon_server.serve_forever()
    except KeyboardInterrupt:
        moon_server.server_close()


def main() -> None:
    args: argparse.Namespace = parse_args()
    serve(args.port)


if __name__ == "__main__":
    main()
