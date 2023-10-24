FROM rust:slim AS builder

RUN apt-get update -y && \
  apt-get install -y pkg-config make g++ libssl-dev && \
  rustup target add x86_64-unknown-linux-gnu

WORKDIR /app
COPY . .

RUN cargo build --release --target x86_64-unknown-linux-gnu


FROM debian:bookworm-slim
RUN apt-get update -y && \
  apt-get install -y pkg-config make g++ libssl-dev
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/motorbot /bin/motorbot
ENTRYPOINT [ "/bin/motorbot" ]