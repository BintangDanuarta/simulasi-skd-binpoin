FROM rust:1.99-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock build.rs ./
COPY icons ./icons
COPY src ./src
COPY data ./data
COPY static ./static
COPY migrations ./migrations
RUN cargo build --release --locked --no-default-features --bin simulasi-skd-binpoin

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/* && useradd --system --uid 10001 --create-home app
WORKDIR /app
ENV HOME=/home/app
COPY --from=build /src/target/release/simulasi-skd-binpoin /usr/local/bin/simulasi-skd-binpoin
COPY data ./data
COPY static ./static
COPY migrations ./migrations
ENV BIND=0.0.0.0:8787 CPNS_APP_DIR=/app MIGRATIONS_DIR=/app/migrations COOKIE_SECURE=true RUST_LOG=info
EXPOSE 8787
USER app
CMD ["/usr/local/bin/simulasi-skd-binpoin"]
