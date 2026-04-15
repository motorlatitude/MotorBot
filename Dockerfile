FROM --platform=$BUILDPLATFORM tonistiigi/xx AS xx

# Builder Image
FROM --platform=$BUILDPLATFORM rust:slim AS builder
RUN apt-get update && apt-get install -y clang lld pkg-config make g++ libssl-dev libc6 tzdata
ARG TARGETPLATFORM

WORKDIR /app
COPY . .
COPY --from=xx / /

RUN xx-cargo build --release --target-dir ./build && \
    xx-verify ./build/$(xx-cargo --print-target-triple)/release/motorbot

RUN cp ./build/$(xx-cargo --print-target-triple)/release/motorbot /bin/motorbot

# Final Image
FROM debian:trixie-slim

LABEL org.opencontainers.image.authors="MotorBot Contributors"
LABEL org.opencontainers.image.description="A simple Discord Bot written in Rust"
LABEL org.opencontainers.image.source="https://github.com/motorlatitude/motorbot"
LABEL org.opencontainers.image.title="MotorBot"

RUN apt-get update -y && \
  apt-get install -y pkg-config make g++ libssl-dev libc6 tzdata

RUN mkdir -p /data

COPY --from=builder /bin/motorbot /bin/motorbot
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

ENTRYPOINT [ "/bin/motorbot" ]