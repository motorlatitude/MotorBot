FROM rust:slim AS builder

RUN apt-get update -y
RUN apt-get install -y pkg-config make g++ libssl-dev libc6 tzdata
RUN echo "$(rustc -vV | sed -n 's|host: ||p')" > /var/rust_target
RUN rustup target add $(cat /var/rust_target)

WORKDIR /app
COPY . .

RUN cargo build --target $(cat /var/rust_target) --release
RUN cp /app/target/$(cat /var/rust_target)/release/motorbot /bin/motorbot

FROM debian:trixie-slim
RUN apt-get update -y && \
  apt-get install -y pkg-config make g++ libssl-dev libc6 tzdata

RUN mkdir -p /data

LABEL org.opencontainers.image.authors="MotorBot Contributors" \
      org.opencontainers.image.description="A simple Discord Bot written in Rust" \
      org.opencontainers.image.source="https://github.com/motorlatitude/MotorBot" \
      org.opencontainers.image.title="MotorBot"

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /bin/motorbot /bin/motorbot
ENTRYPOINT [ "/bin/motorbot" ]
