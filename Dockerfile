# syntax=docker/dockerfile:1.6
#
# Multi-stage build for the Rust port of forgottenserver.
#
# Build context: **monorepo root** (so the data/ symlink at
# apps/poketibia/forgottenserver-rust/data → ../forgottenserver/data
# resolves to a real path during COPY).
#
# Build:
#     docker build \
#       -f apps/poketibia/forgottenserver-rust/Dockerfile \
#       -t forgottenserver-rust:latest \
#       .
#
# Run (status port only — no MariaDB needed since database wiring is
# currently PARTIAL per architectural-parity scope):
#     docker run --rm -p 7171:7171 forgottenserver-rust:latest
#
# Expected image size: ~80–120 MB (release binary 1.1 MB + debian-slim base
# + ca-certificates). Target: < 250 MB.
#
# Expected cold-cache build time: 3–5 minutes (mlua's vendored Lua 5.4 is
# the longest single compilation step; rsa + num-bigint-dig also slow).
# With registry + target caches via BuildKit, incremental builds are <30 s.

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
# .dockerignore at apps/poketibia/forgottenserver-rust/.dockerignore so
# target/, .git/, etc. are excluded.
COPY apps/poketibia/forgottenserver-rust/Cargo.toml ./Cargo.toml
COPY apps/poketibia/forgottenserver-rust/Cargo.lock ./Cargo.lock
COPY apps/poketibia/forgottenserver-rust/crates ./crates
# boot.rs embeds schema.sql at compile time via include_str!("../../../../forgottenserver/schema.sql").
# From crates/poketibia-server/src/ the 4 ".." land at /usr/src/, so the file must live here:
COPY apps/poketibia/forgottenserver/schema.sql /usr/src/forgottenserver/schema.sql

# Build the release binary. BuildKit cache mounts speed up rebuilds on
# subsequent `docker build` invocations sharing the same builder.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/forgottenserver-rust/target \
    cargo build --release --bin poketibia-server \
 && cp /usr/src/forgottenserver-rust/target/release/poketibia-server \
       /usr/local/bin/poketibia-server

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

COPY --from=build /usr/local/bin/poketibia-server /bin/poketibia-server

# Game data (items.otb, weapons.xml, etc.) — comes from the C++ vendored
# tree via the data symlink in the Rust workspace.
COPY apps/poketibia/forgottenserver/data /srv/data/

# Default config — overridable via a bind-mount on /srv/config.lua.
COPY apps/poketibia/forgottenserver-rust/crates/poketibia-server/tests/fixtures/config.lua \
     /srv/config.lua

EXPOSE 7171 7172 8080
WORKDIR /srv
ENTRYPOINT ["/bin/poketibia-server"]
CMD ["--config", "/srv/config.lua", "--data", "/srv/data"]
