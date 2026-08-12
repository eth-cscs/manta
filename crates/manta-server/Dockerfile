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
#       -v $HOME/.config/manta:/home/manta/.config/manta:ro \
#       -v /etc/manta/tls:/etc/manta/tls:ro \
#       manta-server
#
# The server reads `~/.config/manta/server.toml` for its [server]
# block (port, listen address, TLS paths, console-inactivity timeout,
# auth rate limit) and [sites.*] table. Override the path with
# MANTA_SERVER_CONFIG, or any of these flags: --port, --listen-address,
# --cert, --key.

# 1.88 is the effective MSRV: edition 2024 lands in 1.85, but the
# `aws-sdk-sts` crate (pulled in transitively via csm-rs) requires
# 1.88. Bump in lockstep with the CLI Dockerfile.
FROM rust:1.88-bookworm AS builder
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
# OCI image-spec labels. `image.source` lets GHCR auto-link this
# package to the repo (inherited visibility + permissions); the rest
# show up in the ghcr.io package UI.
LABEL org.opencontainers.image.source="https://github.com/eth-cscs/manta"
LABEL org.opencontainers.image.description="manta server — Axum HTTPS frontend to CSM/OCHAMI backends"
LABEL org.opencontainers.image.licenses="MIT"
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
# Run as a non-root user. Production Kubernetes deployments routinely
# enforce `runAsNonRoot: true` and would refuse a root-only image.
# UID 1000 matches the conventional first-user UID on most distros, so
# bind-mounted config files written by a developer's host user can be
# read inside the container without extra chmod.
RUN useradd --uid 1000 --user-group --create-home --shell /usr/sbin/nologin manta
USER manta
# Make explicit the signal the runtime expects on shutdown. `docker
# stop` and k8s pod termination both send SIGTERM; the binary's
# graceful-shutdown handler drains in-flight requests for up to 30s.
STOPSIGNAL SIGTERM
# Default port per the server.toml.example template. Operators that
# override the port in their mounted server.toml should `-p` the new
# port instead.
EXPOSE 8443
ENTRYPOINT ["manta-server"]
