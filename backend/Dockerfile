# ---- build stage ----
FROM rust:1.90-bookworm AS builder
WORKDIR /build

# Cache dependencies across source edits. Migrations are copied too because
# the SQLx migrator embeds them at compile time.
COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
RUN mkdir -p crates && \
    for c in toolfix-contracts toolfix-domain toolfix-auth toolfix-persistence \
             toolfix-location toolfix-notifications toolfix-storage toolfix-payments \
             toolfix-matching toolfix-pricing toolfix-agent toolfix-jobs \
             toolfix-runtime toolfix-api toolfix-server; do \
        mkdir -p crates/$c/src && echo "" > crates/$c/src/lib.rs; \
    done && \
    echo 'fn main() {}' > crates/toolfix-server/src/main.rs && \
    cargo build --release -p toolfix-server --locked || true

COPY crates ./crates
COPY migrations ./migrations
RUN touch crates/*/src/lib.rs crates/toolfix-server/src/main.rs && \
    cargo build --release -p toolfix-server --locked

# ---- runtime stage ----
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*
RUN useradd --system --create-home --shell /usr/sbin/nologin toolfix

COPY --from=builder /build/target/release/toolfix-server /usr/local/bin/toolfix-server
COPY --from=builder /build/migrations /app/migrations

USER toolfix
WORKDIR /app
ENV TOOLFIX_HOST=0.0.0.0 \
    TOOLFIX_PORT=8080

EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s \
    CMD curl -fsS http://localhost:8080/health/live || exit 1

ENTRYPOINT ["/usr/local/bin/toolfix-server"]
