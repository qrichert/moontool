FROM rust:1-slim-bookworm AS builder

WORKDIR /app
COPY rust/ .
RUN cargo build --release --locked --all-features


FROM ghcr.io/astral-sh/uv:python3.14-bookworm-slim

WORKDIR /app
COPY webmoon/pyproject.toml webmoon/uv.lock ./
RUN uv sync --frozen --no-dev

COPY webmoon/webmoon.py .
COPY --from=builder /app/target/release/moontool /usr/local/bin/moontool

RUN groupadd --system --gid 10001 webmoon && \
    useradd --system --uid 10001 --gid 10001 --create-home webmoon
USER webmoon

EXPOSE 2222

CMD ["uv", "run", "fastapi", "run", "--host", "0.0.0.0", "--port", "2222", "webmoon.py"]
