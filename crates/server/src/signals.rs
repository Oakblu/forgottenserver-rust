//! Signal handling for the game server.
//!
//! Migrated from forgottenserver signals.h / signals.cpp.
//!
//! The C++ implementation used Boost.Asio signal_set with:
//!   - SIGINT  → shutdown
//!   - SIGTERM → shutdown
//!   - SIGHUP  → reload config/scripts (Unix only)
//!   - SIGUSR1 → save game state (Unix only)
//!
//! This pure-Rust implementation simulates those semantics with explicit
//! `Signal` variants, atomic shutdown/reload flags, and an optional save-state
//! flag — without depending on OS-level signals in tests.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Signal enum
// ---------------------------------------------------------------------------

/// Represents OS-level signals that the game server cares about.
#[derive(Debug, Clone, PartialEq)]
pub enum Signal {
    /// SIGTERM or SIGINT — shuts the server down cleanly.
    Shutdown,
    /// SIGHUP — reloads Lua scripts and config files.
    Reload,
    /// SIGUSR1 — saves the current game state.
    SaveState,
}

// ---------------------------------------------------------------------------
// SignalHandler
// ---------------------------------------------------------------------------

/// Receives and dispatches OS signals (simulated in pure Rust).
///
/// Maintains:
/// - A log of every signal received (in order).
/// - An atomic `shutdown_flag` that is set on `Shutdown`.
/// - An atomic `reload_flag` that is set on `Reload`.
/// - An atomic `save_state_flag` that is set on `SaveState`.
/// - Optional callbacks invoked synchronously on each signal.
pub struct SignalHandler {
    received: Vec<Signal>,
    shutdown_flag: Arc<AtomicBool>,
    reload_flag: Arc<AtomicBool>,
    save_state_flag: Arc<AtomicBool>,
    on_shutdown: Option<Box<dyn Fn() + Send>>,
    on_reload: Option<Box<dyn Fn() + Send>>,
    on_save_state: Option<Box<dyn Fn() + Send>>,
}

impl SignalHandler {
    /// Creates a new `SignalHandler` with all flags cleared.
    pub fn new() -> Self {
        Self {
            received: Vec::new(),
            shutdown_flag: Arc::new(AtomicBool::new(false)),
            reload_flag: Arc::new(AtomicBool::new(false)),
            save_state_flag: Arc::new(AtomicBool::new(false)),
            on_shutdown: None,
            on_reload: None,
            on_save_state: None,
        }
    }

    // -----------------------------------------------------------------------
    // Flag accessors
    // -----------------------------------------------------------------------

    /// Returns `true` if a `Shutdown` signal has been received.
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_flag.load(Ordering::SeqCst)
    }

    /// Returns `true` if a `Reload` signal has been received.
    pub fn is_reload_requested(&self) -> bool {
        self.reload_flag.load(Ordering::SeqCst)
    }

    /// Returns `true` if a `SaveState` signal has been received.
    pub fn is_save_state_requested(&self) -> bool {
        self.save_state_flag.load(Ordering::SeqCst)
    }

    /// Returns a cloned `Arc` to the shutdown flag for sharing across threads.
    pub fn shutdown_flag(&self) -> Arc<AtomicBool> {
        self.shutdown_flag.clone()
    }

    /// Returns a cloned `Arc` to the reload flag for sharing across threads.
    pub fn reload_flag(&self) -> Arc<AtomicBool> {
        self.reload_flag.clone()
    }

    /// Clears the reload flag (after scripts have been reloaded).
    pub fn clear_reload_flag(&self) {
        self.reload_flag.store(false, Ordering::SeqCst);
    }

    /// Clears the save-state flag (after the game state has been saved).
    pub fn clear_save_state_flag(&self) {
        self.save_state_flag.store(false, Ordering::SeqCst);
    }

    // -----------------------------------------------------------------------
    // Handler registration
    // -----------------------------------------------------------------------

    pub fn set_shutdown_handler(&mut self, f: Box<dyn Fn() + Send>) {
        self.on_shutdown = Some(f);
    }

    pub fn set_reload_handler(&mut self, f: Box<dyn Fn() + Send>) {
        self.on_reload = Some(f);
    }

    pub fn set_save_state_handler(&mut self, f: Box<dyn Fn() + Send>) {
        self.on_save_state = Some(f);
    }

    // -----------------------------------------------------------------------
    // Signal dispatch
    // -----------------------------------------------------------------------

    /// Dispatches a signal: sets the appropriate flag, invokes the callback,
    /// and appends to the received log.
    pub fn handle_signal(&mut self, signal: Signal) {
        match &signal {
            Signal::Shutdown => {
                self.shutdown_flag.store(true, Ordering::SeqCst);
                if let Some(handler) = &self.on_shutdown {
                    handler();
                }
            }
            Signal::Reload => {
                self.reload_flag.store(true, Ordering::SeqCst);
                if let Some(handler) = &self.on_reload {
                    handler();
                }
            }
            Signal::SaveState => {
                self.save_state_flag.store(true, Ordering::SeqCst);
                if let Some(handler) = &self.on_save_state {
                    handler();
                }
            }
        }
        self.received.push(signal);
    }

    /// Returns all signals received so far, in order.
    pub fn received_signals(&self) -> &[Signal] {
        &self.received
    }

    /// Clears the recorded signal log (does not reset flags).
    pub fn clear_received(&mut self) {
        self.received.clear();
    }
}

impl Default for SignalHandler {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    // -----------------------------------------------------------------------
    // Existing tests (preserved)
    // -----------------------------------------------------------------------

    #[test]
    fn handle_shutdown_records_signal() {
        let mut sh = SignalHandler::new();
        sh.handle_signal(Signal::Shutdown);
        assert!(sh.received_signals().contains(&Signal::Shutdown));
    }

    #[test]
    fn handle_reload_calls_reload_handler() {
        let called = Arc::new(Mutex::new(false));
        let c = called.clone();
        let mut sh = SignalHandler::new();
        sh.set_reload_handler(Box::new(move || {
            *c.lock().unwrap() = true;
        }));
        sh.handle_signal(Signal::Reload);
        assert!(*called.lock().unwrap());
    }

    #[test]
    fn handle_shutdown_calls_shutdown_handler() {
        let called = Arc::new(Mutex::new(false));
        let c = called.clone();
        let mut sh = SignalHandler::new();
        sh.set_shutdown_handler(Box::new(move || {
            *c.lock().unwrap() = true;
        }));
        sh.handle_signal(Signal::Shutdown);
        assert!(*called.lock().unwrap());
    }

    #[test]
    fn multiple_signals_accumulate_in_order() {
        let mut sh = SignalHandler::new();
        sh.handle_signal(Signal::Shutdown);
        sh.handle_signal(Signal::Reload);
        sh.handle_signal(Signal::Shutdown);
        let received = sh.received_signals();
        assert_eq!(received.len(), 3);
        assert_eq!(received[0], Signal::Shutdown);
        assert_eq!(received[1], Signal::Reload);
        assert_eq!(received[2], Signal::Shutdown);
    }

    #[test]
    fn no_handler_set_does_not_panic() {
        let mut sh = SignalHandler::new();
        sh.handle_signal(Signal::Shutdown);
        sh.handle_signal(Signal::Reload);
        // No panic expected
        assert_eq!(sh.received_signals().len(), 2);
    }

    #[test]
    fn handle_reload_records_reload_signal() {
        let mut sh = SignalHandler::new();
        sh.handle_signal(Signal::Reload);
        assert!(sh.received_signals().contains(&Signal::Reload));
    }

    // -----------------------------------------------------------------------
    // Phase 14.7 — new flag-based tests
    // -----------------------------------------------------------------------

    /// SIGTERM equivalent: sets the shutdown flag.
    #[test]
    fn sigterm_triggers_shutdown_flag() {
        let mut sh = SignalHandler::new();
        assert!(
            !sh.is_shutdown_requested(),
            "shutdown flag should be clear initially"
        );
        sh.handle_signal(Signal::Shutdown);
        assert!(
            sh.is_shutdown_requested(),
            "SIGTERM (Shutdown) must set the shutdown flag"
        );
    }

    /// SIGINT equivalent: also sets the shutdown flag.
    #[test]
    fn sigint_triggers_shutdown_flag() {
        let mut sh = SignalHandler::new();
        // SIGINT maps to Signal::Shutdown — same as SIGTERM
        sh.handle_signal(Signal::Shutdown);
        assert!(
            sh.is_shutdown_requested(),
            "SIGINT (Shutdown) must set the shutdown flag"
        );
    }

    /// SIGHUP equivalent: sets the reload flag.
    #[test]
    fn sighup_triggers_reload_flag() {
        let mut sh = SignalHandler::new();
        assert!(
            !sh.is_reload_requested(),
            "reload flag should be clear initially"
        );
        sh.handle_signal(Signal::Reload);
        assert!(
            sh.is_reload_requested(),
            "SIGHUP (Reload) must set the reload flag"
        );
    }

    /// Default state: no shutdown has been requested.
    #[test]
    fn signals_default_no_shutdown() {
        let sh = SignalHandler::new();
        assert!(
            !sh.is_shutdown_requested(),
            "fresh SignalHandler must not have shutdown requested"
        );
        assert!(
            !sh.is_reload_requested(),
            "fresh SignalHandler must not have reload requested"
        );
        assert!(
            !sh.is_save_state_requested(),
            "fresh SignalHandler must not have save-state requested"
        );
    }

    /// SIGUSR1 equivalent: sets the save-state flag.
    #[test]
    fn sigusr1_triggers_save_state_flag() {
        let mut sh = SignalHandler::new();
        assert!(!sh.is_save_state_requested());
        sh.handle_signal(Signal::SaveState);
        assert!(
            sh.is_save_state_requested(),
            "SIGUSR1 (SaveState) must set the save-state flag"
        );
    }

    /// SIGUSR1 invokes the save-state callback.
    #[test]
    fn sigusr1_calls_save_state_handler() {
        let called = Arc::new(Mutex::new(false));
        let c = called.clone();
        let mut sh = SignalHandler::new();
        sh.set_save_state_handler(Box::new(move || {
            *c.lock().unwrap() = true;
        }));
        sh.handle_signal(Signal::SaveState);
        assert!(*called.lock().unwrap());
    }

    /// Shutdown flag remains set even after additional signals.
    #[test]
    fn shutdown_flag_persists_after_reload_signal() {
        let mut sh = SignalHandler::new();
        sh.handle_signal(Signal::Shutdown);
        sh.handle_signal(Signal::Reload);
        assert!(sh.is_shutdown_requested());
        assert!(sh.is_reload_requested());
    }

    /// Reload flag can be cleared programmatically (after the reload completes).
    #[test]
    fn reload_flag_can_be_cleared() {
        let mut sh = SignalHandler::new();
        sh.handle_signal(Signal::Reload);
        assert!(sh.is_reload_requested());
        sh.clear_reload_flag();
        assert!(!sh.is_reload_requested());
    }

    /// Save-state flag can be cleared programmatically.
    #[test]
    fn save_state_flag_can_be_cleared() {
        let mut sh = SignalHandler::new();
        sh.handle_signal(Signal::SaveState);
        assert!(sh.is_save_state_requested());
        sh.clear_save_state_flag();
        assert!(!sh.is_save_state_requested());
    }

    /// The shutdown flag Arc can be shared across threads.
    #[test]
    fn shutdown_flag_arc_is_shared() {
        let mut sh = SignalHandler::new();
        let flag = sh.shutdown_flag();
        assert!(!flag.load(std::sync::atomic::Ordering::SeqCst));
        sh.handle_signal(Signal::Shutdown);
        assert!(flag.load(std::sync::atomic::Ordering::SeqCst));
    }

    /// Clearing the received log does not affect flags.
    #[test]
    fn clear_received_does_not_reset_flags() {
        let mut sh = SignalHandler::new();
        sh.handle_signal(Signal::Shutdown);
        sh.handle_signal(Signal::Reload);
        sh.clear_received();
        assert_eq!(sh.received_signals().len(), 0);
        assert!(sh.is_shutdown_requested());
        assert!(sh.is_reload_requested());
    }

    /// SaveState signal is recorded in the log.
    #[test]
    fn save_state_signal_recorded_in_log() {
        let mut sh = SignalHandler::new();
        sh.handle_signal(Signal::SaveState);
        assert!(sh.received_signals().contains(&Signal::SaveState));
    }

    /// Default constructor produces the same state as `new()`.
    #[test]
    fn default_constructor_produces_clean_state() {
        let sh = SignalHandler::default();
        assert!(!sh.is_shutdown_requested());
        assert!(!sh.is_reload_requested());
        assert!(!sh.is_save_state_requested());
        assert_eq!(sh.received_signals().len(), 0);
    }

    /// The reload flag Arc can be shared across threads (mirrors the shutdown-flag Arc getter).
    #[test]
    fn reload_flag_arc_is_shared() {
        let mut sh = SignalHandler::new();
        let flag = sh.reload_flag();
        assert!(!flag.load(std::sync::atomic::Ordering::SeqCst));
        sh.handle_signal(Signal::Reload);
        assert!(flag.load(std::sync::atomic::Ordering::SeqCst));
    }

    /// Reload flag is not set by a Shutdown signal.
    #[test]
    fn shutdown_does_not_set_reload_flag() {
        let mut sh = SignalHandler::new();
        sh.handle_signal(Signal::Shutdown);
        assert!(!sh.is_reload_requested());
    }

    /// Shutdown flag is not set by a Reload signal.
    #[test]
    fn reload_does_not_set_shutdown_flag() {
        let mut sh = SignalHandler::new();
        sh.handle_signal(Signal::Reload);
        assert!(!sh.is_shutdown_requested());
    }
}
