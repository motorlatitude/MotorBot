FROM --platform=$BUILDPLATFORM tonistiigi/xx AS xx

# Builder Image
FROM --platform=$BUILDPLATFORM rust:slim AS builder
ARG TARGETPLATFORM

RUN if [ "${TARGETPLATFORM}" = "linux/arm64" ]; then \
    dpkg --add-architecture arm64 && \
    apt update && \
    apt install -y \
        clang \
        lld \
        pkg-config \
        make \
        libc6-dev:arm64 \
        tzdata \
        perl \
        gcc-aarch64-linux-gnu \
        g++-aarch64-linux-gnu \
        libssl-dev:arm64; \
    fi && \
    rm -rf /var/lib/apt/lists/*

RUN if [ "${TARGETPLATFORM}" = "linux/amd64" ]; then \
    apt update && \
    apt install -y \
        clang \
        lld \
        pkg-config \
        make \
        g++ \
        libc6 \
        tzdata \
        perl \
        libssl-dev; \
    fi && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
COPY --from=xx / /

RUN xx-cargo build --release --target-dir ./build && \
    xx-verify ./build/$(xx-cargo --print-target-triple)/release/motorbot

RUN cp ./build/$(xx-cargo --print-target-triple)/release/motorbot /bin/motorbot

# Final Image
FROM debian:trixie-slim AS runtime

LABEL org.opencontainers.image.authors="MotorBot Contributors"
LABEL org.opencontainers.image.description="A simple Discord Bot written in Rust"
LABEL org.opencontainers.image.source="https://github.com/motorlatitude/motorbot"
LABEL org.opencontainers.image.title="MotorBot"

RUN apt-get update -y && \
    apt-get install -y tzdata && \
    rm -rf /var/lib/apt/lists/*

RUN mkdir -p /data

COPY --from=builder /bin/motorbot /bin/motorbot
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

ENTRYPOINT [ "/bin/motorbot" ]