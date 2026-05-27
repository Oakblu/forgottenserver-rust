//! `poketibia-server` — bootable binary entrypoint for the Rust port of
//! forgottenserver.
//!
//! This crate is intentionally thin: it owns the binary's `main()` flow and
//! the cross-crate orchestration (config load → game data load → listener
//! spawn → signal install → shutdown wait). All real game logic lives in the
//! ten library crates already in the workspace
//! (`forgottenserver-{common,items,map,entity,world,database,game,scripting,
//! network,server}`); this crate adds *zero* domain logic.
//!
//! The `lib.rs` module surface exists so the integration tests in `tests/`
//! can drive the orchestration without spawning a child process.

pub mod boot;
