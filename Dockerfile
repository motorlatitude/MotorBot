FROM rust:slim AS builder

RUN apt-get update -y && \
  apt-get install -y pkg-config make g++ libssl-dev libc6 tzdata && \
  rustup target add x86_64-unknown-linux-gnu

WORKDIR /app
COPY . .

RUN cargo build --release --target x86_64-unknown-linux-gnu


FROM debian:latest
RUN apt-get update -y && \
  apt-get install -y pkg-config make g++ libssl-dev libc6 tzdata

RUN mkdir -p /data

LABEL org.opencontainers.image.authors="MotorBot Contributors" \
      org.opencontainers.image.description="A simple Discord Bot written in Rust" \
      org.opencontainers.image.source="https://github.com/motorlatitude/MotorBot" \
      org.opencontainers.image.title="MotorBot"

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/motorbot /bin/motorbot
ENTRYPOINT [ "/bin/motorbot" ]
