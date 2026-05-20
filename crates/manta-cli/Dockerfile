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
#       -v $HOME/.config/manta:/root/.config/manta \
#       -e MANTA_CSM_TOKEN \
#       manta-cli get redfish-endpoints
#
# (The image ENTRYPOINT is `manta`, so anything after the image name
#  becomes CLI args.)

FROM rust:1.83-bookworm AS builder
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

FROM debian:bookworm-slim
# Runtime libraries:
#   ca-certificates — HTTPS verification against manta-server
#   librdkafka1     — dynamically linked by the rdkafka crate
#   libcurl4        — pulled in transitively by librdkafka
# rustls handles TLS itself, so no libssl3 is needed.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates libcurl4 librdkafka1 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/bin/manta /usr/local/bin/manta
ENTRYPOINT ["manta"]
