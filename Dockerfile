FROM rust:slim AS builder

RUN apt-get update -y && \
  apt-get install -y pkg-config make g++ libssl-dev libc6 tzdata && \
  rustup target add x86_64-unknown-linux-gnu

WORKDIR /app
COPY . .

RUN cargo build --release --target x86_64-unknown-linux-gnu


FROM debian:latest
RUN apt-get update -y && \
  apt-get install -y libssl-dev libc6 tzdata

RUN mkdir -p /data

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/motorbot /bin/motorbot
ENTRYPOINT [ "/bin/motorbot" ]
