//! Per-method byte-parity tests for `crates/network/src/protocolgame.rs`.
//!
//! Each per-cluster module (`container`, `creature`, `player`, `ui`, ...)
//! contains one parity test per `serialize_*` it covers.  Tests assert
//! the Rust output bytes against literals captured directly from
//! `apps/poketibia/forgottenserver/src/protocolgame.cpp` (cited inline).
//!
//! All shared fixtures and the `expected_bytes!` macro live in
//! `tests/fixtures/mod.rs`.

#[macro_use]
mod fixtures;

mod combat_misc;
mod container;
mod creature;
mod market;
mod player;
mod shop;
mod trade;
mod ui;
