# Build application
FROM rust:1.83-alpine3.20 AS builder

RUN apk add --no-cache openssl libc-dev openssl-dev

RUN mkdir -p /work
WORKDIR /work

COPY Cargo.toml Cargo.toml
RUN mkdir src/
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/app*

COPY . .

RUN RUSTFLAGS="-Ctarget-feature=-crt-static" cargo build --release
RUN strip /work/target/release/pod-metrics-exporter

# Create runner image
FROM alpine:3.21 AS runner

RUN apk add --no-cache libgcc openssl ca-certificates tini

ENTRYPOINT ["/sbin/tini", "--"]

COPY --from=builder /work/target/release/pod-metrics-exporter /pod-metrics-exporter

EXPOSE 3000
ENV RUST_LOG=info

CMD ["/pod-metrics-exporter"]
