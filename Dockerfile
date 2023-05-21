FROM rust:1.64.0 as builder
# RUN apt-get update -y
# RUN apt-get upgrade -y
# RUN apt-get install -y pkg-config libssl-dev musl-tools
WORKDIR /usr/src/manta
COPY . .
# RUN cargo install --target x86_64-unknown-linux-gnu --path .
RUN cargo install --path .

# FROM debian:bullseye
FROM debian:sid-slim
RUN apt-get update & apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*
# FROM rust:1.64.0-alpine
COPY --from=builder /usr/local/cargo/bin/manta /usr/local/bin/manta
COPY --from=builder /usr/src/manta/config /root/.config/manta/config
# Install CA files from linux host to the container
RUN apt-get update
RUN apt-get install -y ca-certificates
RUN update-ca-certificates
# RUN mkdir -p /etc/ssl/certs
# COPY /etc/ssl/certs/DigiCert* /etc/ssl/certs/
ENTRYPOINT ["manta"]

