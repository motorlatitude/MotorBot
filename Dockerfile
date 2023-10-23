# Builder Stage
FROM ekidd/rust-musl-builder:latest as builder

WORKDIR /app
COPY . .

# Add our source code.
ADD --chown=rust:rust . ./

# Build our application.
RUN cargo build --release

FROM scratch as runtime
WORKDIR /app
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/motorbot ./

ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs
ENV APP_ENVIRONMENT production
CMD ["/app/motorbot"]