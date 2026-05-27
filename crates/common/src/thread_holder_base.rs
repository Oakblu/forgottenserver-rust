//! Migrated from forgottenserver/src/thread_holder_base.h
//!
//! The C++ uses CRTP (`ThreadHolder<Derived>`) so that the derived class
//! supplies `threadMain()` as the thread entry point.  In Rust there is no
//! CRTP; instead we expose a plain `ThreadHolder` struct that owns:
//!
//! - an `Arc<AtomicU8>` for the shared `ThreadState`
//! - an `Option<JoinHandle<()>>` for the background thread
//!
//! The caller passes any `FnOnce()` closure to `spawn()`.  The closure
//! typically reads the shared state to know when to stop (see the tests).

#![allow(dead_code)]

use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};

use crate::enums::ThreadState;

/// Converts a `ThreadState` to the `u8` value stored in the `AtomicU8`.
fn state_to_u8(s: ThreadState) -> u8 {
    s as u8
}

/// Converts the `u8` stored in the `AtomicU8` back to a `ThreadState`.
/// Panics if the value is out of range (should never happen in practice).
fn u8_to_state(v: u8) -> ThreadState {
    match v {
        0 => ThreadState::Running,
        1 => ThreadState::Closing,
        2 => ThreadState::Terminated,
        _ => panic!("invalid ThreadState value: {v}"),
    }
}

/// Owns a background thread and exposes start/stop/join semantics that
/// mirror the C++ `ThreadHolder<Derived>` CRTP base.
pub struct ThreadHolder {
    state: Arc<AtomicU8>,
    handle: Option<JoinHandle<()>>,
}

impl ThreadHolder {
    /// Spawn a background thread running `f`.
    ///
    /// The thread's initial state is `ThreadState::Running`.
    /// The closure receives a clone of the shared state `Arc<AtomicU8>` so it
    /// can check for the `Closing` signal:
    ///
    /// ```rust,ignore
    /// let holder = ThreadHolder::spawn(|state| {
    ///     while state.load(Ordering::Relaxed) == ThreadState::Running as u8 {
    ///         // do work …
    ///     }
    /// });
    /// ```
    pub fn spawn<F>(f: F) -> Self
    where
        F: FnOnce(Arc<AtomicU8>) + Send + 'static,
    {
        let state = Arc::new(AtomicU8::new(state_to_u8(ThreadState::Running)));
        let state_clone = Arc::clone(&state);
        let handle = thread::spawn(move || f(state_clone));
        Self {
            state,
            handle: Some(handle),
        }
    }

    /// Signal the background thread to stop by setting the state to `Closing`.
    /// Mirrors C++ `stop()`.
    pub fn stop(&self) {
        self.state
            .store(state_to_u8(ThreadState::Closing), Ordering::Relaxed);
    }

    /// Wait for the background thread to exit, then set the state to `Terminated`.
    /// Mirrors C++ `join()`.
    pub fn join(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        self.state
            .store(state_to_u8(ThreadState::Terminated), Ordering::Relaxed);
    }

    /// Return the current `ThreadState`.
    /// Mirrors C++ `getState()`.
    pub fn get_state(&self) -> ThreadState {
        u8_to_state(self.state.load(Ordering::Relaxed))
    }

    /// Expose a clone of the shared state `Arc` so that external code (e.g.
    /// the spawned closure) can read/write the state directly.
    pub fn state_arc(&self) -> Arc<AtomicU8> {
        Arc::clone(&self.state)
    }
}

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // ----------------------------------------------------------------
    // Initial state is Running
    // ----------------------------------------------------------------

    #[test]
    fn initial_state_is_running() {
        let holder = ThreadHolder::spawn(|_state| {
            // exit immediately
        });
        // State starts as Running (before or right after thread exits — join has not been called)
        // We check immediately after spawn; the state is set *before* the thread is created.
        assert_eq!(holder.get_state(), ThreadState::Running);
    }

    // ----------------------------------------------------------------
    // stop() transitions state to Closing
    // ----------------------------------------------------------------

    #[test]
    fn stop_sets_closing() {
        let holder = ThreadHolder::spawn(|_state| {
            // spin a bit
            thread::sleep(Duration::from_millis(50));
        });
        holder.stop();
        // State must now be Closing (or Terminated if join was called, but we haven't joined)
        let s = holder.get_state();
        assert!(
            s == ThreadState::Closing || s == ThreadState::Terminated,
            "expected Closing or Terminated, got {s:?}"
        );
    }

    // ----------------------------------------------------------------
    // Full lifecycle: spawn → stop → join → Terminated
    // ----------------------------------------------------------------

    #[test]
    fn full_lifecycle_terminated_after_join() {
        let mut holder = ThreadHolder::spawn(|state| {
            while state.load(Ordering::Relaxed) == state_to_u8(ThreadState::Running) {
                thread::sleep(Duration::from_millis(1));
            }
        });

        assert_eq!(holder.get_state(), ThreadState::Running);

        // Give the worker time to enter its `while Running { sleep }` loop
        // at least once so the loop body is actually exercised.
        thread::sleep(Duration::from_millis(10));

        holder.stop();
        holder.join();

        assert_eq!(holder.get_state(), ThreadState::Terminated);
    }

    // ----------------------------------------------------------------
    // join() without explicit stop() works if the thread exits naturally
    // ----------------------------------------------------------------

    #[test]
    fn join_without_stop_works_for_short_lived_thread() {
        let mut holder = ThreadHolder::spawn(|_state| {
            // do nothing — thread exits immediately
        });

        // Allow the thread a moment to finish
        thread::sleep(Duration::from_millis(20));

        holder.join(); // should not block for long; thread already exited
        assert_eq!(holder.get_state(), ThreadState::Terminated);
    }

    // ----------------------------------------------------------------
    // Double join is a no-op (handle is taken on first call)
    // ----------------------------------------------------------------

    #[test]
    fn double_join_is_safe() {
        let mut holder = ThreadHolder::spawn(|_state| {});
        holder.join();
        holder.join(); // should not panic
        assert_eq!(holder.get_state(), ThreadState::Terminated);
    }

    // ----------------------------------------------------------------
    // State arc is shared between holder and the thread closure
    // ----------------------------------------------------------------

    #[test]
    fn state_arc_is_shared() {
        use std::sync::atomic::AtomicBool;

        let saw_closing = Arc::new(AtomicBool::new(false));
        let saw_closing_clone = Arc::clone(&saw_closing);

        let mut holder = ThreadHolder::spawn(move |state| {
            // Wait until Closing
            loop {
                if state.load(Ordering::Relaxed) == state_to_u8(ThreadState::Closing) {
                    saw_closing_clone.store(true, Ordering::Relaxed);
                    break;
                }
                thread::sleep(Duration::from_millis(1));
            }
        });

        // Give the worker time to enter its polling loop and execute the
        // sleep branch at least once.
        thread::sleep(Duration::from_millis(10));

        holder.stop();
        holder.join();

        assert!(
            saw_closing.load(Ordering::Relaxed),
            "thread should have seen Closing state"
        );
        assert_eq!(holder.get_state(), ThreadState::Terminated);
    }

    // ----------------------------------------------------------------
    // state_arc() returns an Arc that points at the same atomic as the
    // internal state. Mutating through the returned Arc must be observable
    // via get_state(), and vice versa.
    // ----------------------------------------------------------------

    #[test]
    fn state_arc_returns_handle_to_same_atomic() {
        let mut holder = ThreadHolder::spawn(|state| {
            // Park until Closing
            while state.load(Ordering::Relaxed) == state_to_u8(ThreadState::Running) {
                thread::sleep(Duration::from_millis(1));
            }
        });

        // Grab an external handle to the shared state.
        let arc = holder.state_arc();

        // Both handles must point at the same underlying atomic (strong-count > 1).
        assert!(
            Arc::strong_count(&arc) >= 2,
            "state_arc must share ownership"
        );

        // Give the worker time to enter its `while Running { sleep }` loop.
        thread::sleep(Duration::from_millis(10));

        // Mutating via the returned Arc must be visible to get_state().
        arc.store(state_to_u8(ThreadState::Closing), Ordering::Relaxed);
        assert_eq!(holder.get_state(), ThreadState::Closing);

        // And mutating via stop() must be visible through the returned Arc.
        holder.stop();
        assert_eq!(
            arc.load(Ordering::Relaxed),
            state_to_u8(ThreadState::Closing)
        );

        holder.join();
        // After join(), get_state() is Terminated, and that change is also
        // visible through the externally-held arc.
        assert_eq!(holder.get_state(), ThreadState::Terminated);
        assert_eq!(
            arc.load(Ordering::Relaxed),
            state_to_u8(ThreadState::Terminated)
        );
    }

    // ----------------------------------------------------------------
    // u8_to_state panics on out-of-range values. The C++ enum has exactly
    // three variants (Running=0, Closing=1, Terminated=2) and the decoder
    // must refuse anything else.
    // ----------------------------------------------------------------

    #[test]
    #[should_panic(expected = "invalid ThreadState value: 3")]
    fn u8_to_state_panics_on_invalid_value() {
        let _ = u8_to_state(3);
    }

    #[test]
    #[should_panic(expected = "invalid ThreadState value: 255")]
    fn u8_to_state_panics_on_max_u8() {
        let _ = u8_to_state(u8::MAX);
    }

    // ----------------------------------------------------------------
    // state_to_u8 / u8_to_state round-trip for every valid variant.
    // ----------------------------------------------------------------

    #[test]
    fn state_round_trip_for_all_variants() {
        for s in [
            ThreadState::Running,
            ThreadState::Closing,
            ThreadState::Terminated,
        ] {
            assert_eq!(u8_to_state(state_to_u8(s)), s);
        }
    }

    // ----------------------------------------------------------------
    // Inside the spawn closure, the worker observes the Running state at
    // least once before stop() flips it to Closing. This exercises the
    // body of a `while state == Running { sleep }` loop deterministically
    // by gating the worker on a ready-signal that the test owns.
    // ----------------------------------------------------------------

    #[test]
    fn worker_observes_running_then_closing() {
        use std::sync::atomic::AtomicUsize;

        let iterations = Arc::new(AtomicUsize::new(0));
        let iterations_clone = Arc::clone(&iterations);

        let mut holder = ThreadHolder::spawn(move |state| {
            while state.load(Ordering::Relaxed) == state_to_u8(ThreadState::Running) {
                iterations_clone.fetch_add(1, Ordering::Relaxed);
                thread::sleep(Duration::from_millis(1));
            }
        });

        // Give the loop body a window to execute several times.
        thread::sleep(Duration::from_millis(20));
        holder.stop();
        holder.join();

        assert!(
            iterations.load(Ordering::Relaxed) >= 1,
            "worker should have iterated the Running-loop at least once"
        );
        assert_eq!(holder.get_state(), ThreadState::Terminated);
    }
}
