FROM rust:1.64.0 AS build

WORKDIR /root

RUN apt-get update -y
RUN apt-get upgrade -y

RUN rustup target add x86_64-unknown-linux-gnu

ENV KG_CONFIG_SYSROOT_DIR=/
RUN USER=root cargo new manta
COPY Cargo.toml Cargo.lock config /manta/
WORKDIR /manta
COPY src src
RUN cargo clean
RUN apt-get install -y pkg-config libssl-dev musl-tools
RUN cargo build --release

RUN cargo install --target x86_64-unknown-linux-gnu --path .

# FROM scratch
FROM rust:1.64.0-alpine
COPY --from=build /usr/local/cargo/bin/manta /
COPY --from=build /manta/config /root/.config/manta/
ENTRYPOINT ["./manta"]
