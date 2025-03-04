FROM rust:1.85.0 as builder
WORKDIR /usr/src/manta
COPY . .
RUN cargo install --path .

FROM rust:1.85.0-alpine
COPY --from=builder /usr/local/cargo/bin/manta /usr/local/bin/manta
CMD ["manta"]
