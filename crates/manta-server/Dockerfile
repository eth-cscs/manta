# Multi-stage build for the manta HTTP server binary.
#
# Build from the workspace root (the context must be the workspace so
# Cargo.lock and the workspace manifest are visible):
#
#     docker build -f crates/manta-server/Dockerfile -t manta-server .
#
# Run with a mounted config + TLS material; expose the listen port:
#
#     docker run --rm -p 8443:8443 \
#       -v $HOME/.config/manta:/root/.config/manta:ro \
#       -v /etc/manta/tls:/etc/manta/tls:ro \
#       manta-server
#
# The server reads `~/.config/manta/server.toml` for its [server]
# block (port, listen address, TLS paths, console-inactivity timeout,
# auth rate limit) and [sites.*] table. Override the path with
# MANTA_SERVER_CONFIG, or any of these flags: --port, --listen-address,
# --cert, --key.

FROM rust:1.83-bookworm AS builder
# cmake + libcurl4-openssl-dev are needed to build librdkafka, which
# the audit subsystem depends on. Matches the apt step in CI.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        cmake libcurl4-openssl-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/manta
COPY . .
RUN cargo install --path crates/manta-server --root /usr/local

FROM debian:bookworm-slim
# Runtime libraries:
#   ca-certificates — HTTPS verification to CSM/OCHAMI backends
#   librdkafka1     — dynamically linked by the rdkafka crate
#   libcurl4        — pulled in transitively by librdkafka
# rustls (used by axum-server and reqwest) handles TLS itself, so no
# libssl3 is needed.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates libcurl4 librdkafka1 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/bin/manta-server /usr/local/bin/manta-server
# Default port per the server.toml.example template. Operators that
# override the port in their mounted server.toml should `-p` the new
# port instead.
EXPOSE 8443
ENTRYPOINT ["manta-server"]
