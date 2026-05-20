FROM rust:1.83.0 AS builder
WORKDIR /usr/src/manta
COPY . .
# Build just the CLI binary — the server lives in its own crate and
# is shipped via a separate image / install path.
RUN cargo install --path crates/manta-cli --root /usr/local

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*
# `cargo install --path crates/manta-cli` produces a binary called
# `manta` (per the `[[bin]] name = "manta"` block in that crate's
# Cargo.toml).
COPY --from=builder /usr/local/bin/manta /usr/local/bin/manta
ENTRYPOINT ["manta"]
