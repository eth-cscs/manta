# Multi-stage build for the manta CLI binary.
#
# Build from the workspace root (the context must be the workspace so
# Cargo.lock and the workspace manifest are visible):
#
#     docker build -f crates/manta-cli/Dockerfile -t manta-cli .
#
# Run with a mounted config and an ACCESS_TOKEN:
#
#     docker run --rm -it --network=host \
#       -v $HOME/.config/manta:/home/manta/.config/manta \
#       -e MANTA_CSM_TOKEN \
#       manta-cli get redfish-endpoints
#
# (The image ENTRYPOINT is `manta`, so anything after the image name
#  becomes CLI args.)

# Same version as rust-toolchain.toml. `COPY . .` brings that file
# into the build, so rustup would install the pinned toolchain anyway;
# matching the image tag just skips the extra download. Bump in
# lockstep with rust-toolchain.toml and the server Dockerfile.
FROM rust:1.99.1-trixie AS builder
# cmake + libcurl4-openssl-dev are needed to build librdkafka, which
# the audit subsystem depends on. Matches the apt step in CI.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        cmake libcurl4-openssl-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/manta
COPY . .
# `cargo install --path crates/manta-cli` reads that crate's [[bin]]
# block and produces a binary called `manta` (not `manta-cli`).
RUN cargo install --path crates/manta-cli --root /usr/local

FROM debian:trixie-slim
# OCI image-spec labels. `image.source` lets GHCR auto-link this
# package to the repo (inherited visibility + permissions); the rest
# show up in the ghcr.io package UI.
LABEL org.opencontainers.image.source="https://github.com/eth-cscs/manta"
LABEL org.opencontainers.image.description="manta CLI — terminal client for the manta backend"
LABEL org.opencontainers.image.licenses="MIT"
# Runtime libraries: only ca-certificates (HTTPS verification against
# manta-server). The CLI doesn't use Kafka, and `ldd` on the binary
# shows nothing beyond libc/libm/libgcc_s/libz/liblzma, which the slim
# image already ships. rustls handles TLS itself, so no libssl3 is
# needed.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/bin/manta /usr/local/bin/manta
# Run as a non-root user — see the server Dockerfile for the rationale.
# UID 1000 matches the conventional first-user UID so host-owned config
# bind-mounts are readable inside the container without chmod.
RUN useradd --uid 1000 --user-group --create-home --shell /bin/bash manta
USER manta
ENTRYPOINT ["manta"]
