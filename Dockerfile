# syntax=docker/dockerfile:1.6
#
# Multi-stage build for the Rust port of forgottenserver.
# Build context: repo root (forgottenserver-rust/).
#
# Build:   docker build -t forgottenserver-rust:latest .
# Or:      docker compose up --build
#
# Expected image size: ~80–120 MB (release binary + debian-slim base).
# Cold-cache build: ~3–5 min. Incremental with BuildKit cache: <30 s.

# ─── Stage 1: build ──────────────────────────────────────────────────────
FROM rust:1-slim-bookworm AS build

# build-essential covers gcc + make, required to compile mlua's vendored
# Lua 5.4 from C source. pkg-config is used by `openssl-sys` checks.
RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/forgottenserver-rust

# Copy the entire forgottenserver-rust workspace. The COPY honours the
# .dockerignore so target/, .git/, etc. are excluded.
COPY Cargo.toml ./Cargo.toml
COPY Cargo.lock ./Cargo.lock
COPY crates ./crates
# boot.rs embeds schema.sql at compile time via include_str!("../../../schema.sql").
# From crates/tfs/src/ the 3 ".." land at the workspace root, so the file must live here:
COPY schema.sql ./schema.sql

# Build the release binary. BuildKit cache mounts speed up rebuilds on
# subsequent `docker build` invocations sharing the same builder.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/forgottenserver-rust/target \
    cargo build --release --bin tfs \
 && cp /usr/src/forgottenserver-rust/target/release/tfs \
       /usr/local/bin/tfs

# ─── Stage 2: runtime ────────────────────────────────────────────────────
FROM debian:bookworm-slim

# Runtime deps:
#   - ca-certificates: future HTTPS clients (TLS roots).
# mlua's vendored Lua is statically linked, rsa is pure-Rust, and the
# database backend is PARTIAL — so libmariadb/libssl are NOT needed
# at runtime in this milestone.
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /usr/local/bin/tfs /bin/tfs

# Game data (items.otb, weapons.xml, etc.) from the data/ directory
# at the workspace root.
COPY data /srv/data/

# Default config — overridable via a bind-mount on /srv/config.lua.
COPY crates/tfs/tests/fixtures/config.lua /srv/config.lua

EXPOSE 7171 7172 8080
WORKDIR /srv
ENTRYPOINT ["/bin/tfs"]
CMD ["--config", "/srv/config.lua", "--data", "/srv/data"]
