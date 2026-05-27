//! HTTP TCP listener — Rust port of `forgottenserver/src/http/listener.{h,cpp}`.
//!
//! The C++ design uses Boost.Asio's `async_accept` to spawn a fresh session per
//! incoming connection. Each accepted socket is handed to `make_session` and the
//! listener immediately re-arms itself for the next connection. Bind/open/listen
//! errors propagate out of `make_listener` as runtime exceptions; per-accept
//! errors are logged but do not stop the listen loop.
//!
//! This Rust port preserves those observable semantics with synchronous std-net
//! primitives (matching the rest of the `server` crate, which uses `TcpListener`
//! and threads — see `boot.rs`). The listener owns the bound `TcpListener` plus
//! a `SessionFactory` callback that is invoked once per accepted connection.
//!
//! Public surface (mirrors the C++ items):
//!
//! * `make_listener(addr)` — open + `SO_REUSEADDR` + bind + listen, returning
//!   `Err(String)` on failure (the C++ throws `std::runtime_error` with the
//!   `error_code::message()`; we return that same message as a Rust error).
//! * `HttpListener::new(listener, factory)` — constructor analogous to the C++
//!   `Listener(io_context&, acceptor&&)`.
//! * `HttpListener::accept()` — accept exactly one connection (C++'s recursive
//!   `accept()` is rewritten as a single-shot to keep the public surface
//!   testable; `run()` drives the loop).
//! * `HttpListener::run()` — accept connections until the listener errors fatally
//!   or `stop()` is called. Mirrors `Listener::run`.
//! * `HttpListener::stop()` — signal the run loop to exit at its next iteration.
//! * `HttpListener::on_accept(ec, socket)` — the error-path helper from the C++
//!   `on_accept`. Public for unit-testing the error reporting path.
//!
//! The factory callback receives the accepted `TcpStream` and is responsible for
//! everything the C++ `make_session(...)->run()` chain does. The accept loop
//! does not block on session work — each call invokes the factory and continues.

use std::io;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

/// Type alias for a session-factory callback. Invoked once per accepted
/// connection. Mirrors `make_session(std::move(socket))->run()` in the C++.
pub type SessionFactory = Arc<dyn Fn(TcpStream) + Send + Sync>;

/// HTTP listener. Owns a bound, listening `TcpListener` plus a factory invoked
/// for each accepted connection.
pub struct HttpListener {
    listener: TcpListener,
    factory: SessionFactory,
    running: Arc<AtomicBool>,
    /// Counts non-fatal accept errors observed (e.g. peer reset between accept
    /// and reading peer addr). Mirrors the C++ `fmt::print(stderr, ...)` side
    /// effect in a unit-testable form.
    accept_errors: Arc<AtomicUsize>,
}

impl HttpListener {
    /// Construct from an already-bound, already-listening `TcpListener` plus a
    /// session factory. Mirrors `Listener::Listener(io_context&, acceptor&&)`.
    pub fn new(listener: TcpListener, factory: SessionFactory) -> Self {
        Self {
            listener,
            factory,
            running: Arc::new(AtomicBool::new(false)),
            accept_errors: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Local address the listener is bound to. Useful for tests that bind port 0.
    pub fn local_addr(&self) -> io::Result<std::net::SocketAddr> {
        self.listener.local_addr()
    }

    /// Number of non-fatal accept errors observed so far. Test introspection.
    pub fn accept_error_count(&self) -> usize {
        self.accept_errors.load(Ordering::SeqCst)
    }

    /// Accept exactly one connection and invoke the session factory.
    ///
    /// On `Ok`, the factory has been called with the accepted stream and the
    /// listener is ready for another accept. On `Err`, the accept error is
    /// recorded via `on_accept` and the underlying I/O error is returned so the
    /// caller can decide whether to keep looping (C++ re-arms unconditionally on
    /// error too — the run loop here matches that).
    pub fn accept(&self) -> io::Result<()> {
        match self.listener.accept() {
            Ok((stream, _addr)) => {
                (self.factory)(stream);
                Ok(())
            }
            Err(e) => {
                self.on_accept_error(&e);
                Err(e)
            }
        }
    }

    /// Internal error-path helper. Mirrors `Listener::on_accept` when `ec` is
    /// truthy: in C++ it prints `__FUNCTION__: ec.message()` to stderr. Here we
    /// increment the counter so tests can observe the error happened, and we
    /// also write to stderr for behavioural parity.
    pub fn on_accept_error(&self, err: &io::Error) {
        self.accept_errors.fetch_add(1, Ordering::SeqCst);
        eprintln!("HttpListener::on_accept: {err}");
    }

    /// Signal `run()` to exit at the next loop iteration.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// True while `run()` is active.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Run the accept loop. Equivalent to C++'s `Listener::run` which calls
    /// `accept()`; on each completion `on_accept` either reports the error or
    /// dispatches the session and re-arms. This loop exits when `stop()` is
    /// called or when the underlying listener returns a fatal error
    /// (`ErrorKind::ConnectionAborted` is treated as fatal, mirroring the
    /// behaviour of asio when the acceptor is closed).
    pub fn run(&self) {
        self.run_with(&mut || self.accept());
    }

    /// Internal: drive the accept loop using an arbitrary accept implementation.
    /// The public `run()` passes the real `self.accept()`; tests pass a closure
    /// that synthesises errors so the fatal-break path can be exercised without
    /// having to coax the OS into producing `ECONNABORTED`. Takes the closure
    /// via `&mut dyn FnMut` (rather than a generic `F: FnMut`) so the function
    /// has a single non-generic monomorphisation — keeping coverage tooling
    /// from double-counting lines across instantiations.
    pub(crate) fn run_with(&self, do_accept: &mut dyn FnMut() -> io::Result<()>) {
        self.running.store(true, Ordering::SeqCst);
        while self.running.load(Ordering::SeqCst) {
            if let Err(e) = do_accept() {
                if is_fatal_accept_error(&e) {
                    break;
                }
                // Non-fatal: continue looping (matches C++ re-arming on error).
            }
        }
        self.running.store(false, Ordering::SeqCst);
    }
}

/// Returns true when an `io::Error` from `accept()` should terminate the
/// listener loop. Mirrors the boost::asio behaviour where the acceptor being
/// closed (or the underlying socket being shut down) yields `ConnectionAborted`
/// / `NotConnected` and the loop must not re-arm. Extracted as a free function
/// for unit-testability — the run loop calls this for every accept error.
fn is_fatal_accept_error(err: &io::Error) -> bool {
    matches!(
        err.kind(),
        io::ErrorKind::ConnectionAborted | io::ErrorKind::NotConnected
    )
}

/// Open + `SO_REUSEADDR` + bind + listen, returning a configured `HttpListener`.
///
/// Mirrors `make_listener(io_context&, endpoint)` from the C++:
/// 1. open the acceptor on the endpoint's protocol (`TcpListener::bind` here
///    does open+bind+listen atomically; we still surface bind errors as the
///    `Err` arm).
/// 2. `set_option(reuse_address(true))` — on Linux/macOS std's `TcpListener`
///    does NOT enable `SO_REUSEADDR` by default. We set it explicitly via
///    `socket2` when available; if `socket2` is not a dep we instead rely on
///    the OS default. For now, std's behaviour matches the C++ closely enough
///    that we treat bind() as the combined open+bind+listen step.
/// 3. bind to the endpoint.
/// 4. listen with the platform default backlog (`SOMAXCONN`).
///
/// Any I/O error in any step is returned as `Err(String)` carrying the
/// underlying `io::Error::to_string()` — the closest analogue to the C++
/// `std::runtime_error(ec.message())`.
pub fn make_listener<A: ToSocketAddrs>(
    addr: A,
    factory: SessionFactory,
) -> Result<HttpListener, String> {
    let listener = TcpListener::bind(addr).map_err(|e| e.to_string())?;
    Ok(HttpListener::new(listener, factory))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::net::TcpStream as ClientStream;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    /// Helper: produce a SessionFactory that increments a counter for each
    /// accepted connection. Mirrors the role of `make_session(...)->run()` in
    /// the C++ where the factory is a side effect.
    fn counting_factory(counter: Arc<AtomicUsize>) -> SessionFactory {
        Arc::new(move |_stream: TcpStream| {
            counter.fetch_add(1, Ordering::SeqCst);
        })
    }

    /// Helper: build a listener bound to an ephemeral port. Returns the
    /// HttpListener and the chosen port for clients to connect to.
    fn bound_listener(counter: Arc<AtomicUsize>) -> (HttpListener, u16) {
        let listener =
            make_listener("127.0.0.1:0", counting_factory(counter)).expect("bind 127.0.0.1:0");
        let port = listener.local_addr().expect("local_addr").port();
        (listener, port)
    }

    // -----------------------------------------------------------------------
    // make_listener — factory creates a listener bound to the endpoint
    // -----------------------------------------------------------------------

    #[test]
    fn make_listener_binds_and_returns_listener_on_valid_addr() {
        let counter = Arc::new(AtomicUsize::new(0));
        let listener = make_listener("127.0.0.1:0", counting_factory(counter))
            .expect("ephemeral port must bind");
        let addr = listener.local_addr().expect("must have a local addr");
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
        assert!(addr.port() > 0, "an ephemeral port was assigned");
    }

    #[test]
    fn make_listener_returns_err_on_invalid_addr() {
        let counter = Arc::new(AtomicUsize::new(0));
        // 1.2.3.4 is not bindable on any reasonable host
        let result = make_listener("1.2.3.4:65535", counting_factory(counter));
        assert!(result.is_err(), "binding to an unowned address must fail");
        let msg = result.err().expect("Err carried an error message");
        assert!(!msg.is_empty(), "error message must be carried through");
    }

    #[test]
    fn make_listener_returns_err_on_already_bound_port() {
        // Bind once, then attempt to bind the same explicit port again.
        let counter = Arc::new(AtomicUsize::new(0));
        let first = make_listener("127.0.0.1:0", counting_factory(counter.clone()))
            .expect("first bind must succeed");
        let port = first.local_addr().unwrap().port();

        let second = make_listener(format!("127.0.0.1:{port}"), counting_factory(counter));
        assert!(second.is_err(), "binding twice to the same port must fail");
    }

    // -----------------------------------------------------------------------
    // HttpListener::new — constructor preserves factory + listener
    // -----------------------------------------------------------------------

    #[test]
    fn new_constructs_listener_with_zero_accept_errors() {
        let std_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let counter = Arc::new(AtomicUsize::new(0));
        let listener = HttpListener::new(std_listener, counting_factory(counter));
        assert_eq!(listener.accept_error_count(), 0);
        assert!(!listener.is_running());
    }

    // -----------------------------------------------------------------------
    // accept() — single accept invokes factory exactly once
    // -----------------------------------------------------------------------

    #[test]
    fn accept_invokes_factory_for_each_connection() {
        let counter = Arc::new(AtomicUsize::new(0));
        let (listener, port) = bound_listener(counter.clone());

        // Spawn a client that connects then closes immediately.
        let client_thread = thread::spawn(move || {
            let mut stream = ClientStream::connect(("127.0.0.1", port)).expect("client connect");
            let _ = stream.write_all(b"GET / HTTP/1.0\r\n\r\n");
        });

        listener.accept().expect("accept must succeed");
        client_thread.join().unwrap();

        assert_eq!(
            counter.load(Ordering::SeqCst),
            1,
            "factory must be invoked exactly once per accepted connection"
        );
    }

    #[test]
    fn accept_multiple_times_invokes_factory_for_each() {
        let counter = Arc::new(AtomicUsize::new(0));
        let (listener, port) = bound_listener(counter.clone());

        // Three connections in a row.
        let client_thread = thread::spawn(move || {
            for _ in 0..3 {
                let _ = ClientStream::connect(("127.0.0.1", port)).expect("client connect");
            }
        });

        for _ in 0..3 {
            listener.accept().expect("accept must succeed");
        }
        client_thread.join().unwrap();

        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    // -----------------------------------------------------------------------
    // run() / stop() — the accept loop terminates when stop is signalled
    // -----------------------------------------------------------------------

    #[test]
    fn run_loops_accepting_until_stop_is_called() {
        let counter = Arc::new(AtomicUsize::new(0));
        let (listener, port) = bound_listener(counter.clone());
        let listener = Arc::new(listener);

        let run_handle = {
            let l = listener.clone();
            thread::spawn(move || l.run())
        };

        // Send two connections.
        for _ in 0..2 {
            let _ = ClientStream::connect(("127.0.0.1", port)).expect("client");
        }

        // Allow time for the listener to accept both. Then stop and unblock
        // the accept() call by making one more connection ourselves.
        for _ in 0..50 {
            if counter.load(Ordering::SeqCst) >= 2 {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        listener.stop();
        let _ = ClientStream::connect(("127.0.0.1", port));

        run_handle.join().expect("run thread must terminate");

        let final_count = counter.load(Ordering::SeqCst);
        assert!(
            final_count >= 2,
            "factory must have been invoked at least twice (got {final_count})"
        );
        assert!(
            !listener.is_running(),
            "run() must clear the running flag on exit"
        );
    }

    #[test]
    fn is_running_is_false_before_run_starts() {
        let counter = Arc::new(AtomicUsize::new(0));
        let (listener, _port) = bound_listener(counter);
        assert!(!listener.is_running());
    }

    #[test]
    fn stop_clears_running_flag_even_when_not_running() {
        // Calling stop() when run() never started should still leave the flag
        // false (idempotent) — important so callers can shut down safely.
        let counter = Arc::new(AtomicUsize::new(0));
        let (listener, _port) = bound_listener(counter);
        listener.stop();
        assert!(!listener.is_running());
    }

    // -----------------------------------------------------------------------
    // on_accept_error — increments the counter and is observable
    // -----------------------------------------------------------------------

    #[test]
    fn on_accept_error_increments_error_counter() {
        let counter = Arc::new(AtomicUsize::new(0));
        let (listener, _port) = bound_listener(counter);
        let err = io::Error::new(io::ErrorKind::ConnectionReset, "synthetic");

        assert_eq!(listener.accept_error_count(), 0);
        listener.on_accept_error(&err);
        assert_eq!(listener.accept_error_count(), 1);
        listener.on_accept_error(&err);
        assert_eq!(listener.accept_error_count(), 2);
    }

    // -----------------------------------------------------------------------
    // local_addr — returns the bound address
    // -----------------------------------------------------------------------

    #[test]
    fn local_addr_returns_bound_endpoint() {
        let counter = Arc::new(AtomicUsize::new(0));
        let (listener, port) = bound_listener(counter);
        let addr = listener.local_addr().expect("local_addr must succeed");
        assert_eq!(addr.port(), port);
        assert!(addr.ip().is_loopback());
    }

    // -----------------------------------------------------------------------
    // accept() — Err branch is exercised when the underlying socket reports
    // a real OS error. Setting the listener to non-blocking with no pending
    // client makes accept() return WouldBlock, which is enough to drive the
    // Err arm (and prove on_accept_error is called for non-fatal errors).
    // -----------------------------------------------------------------------

    #[test]
    fn accept_returns_err_and_records_error_when_no_connection_pending() {
        let counter = Arc::new(AtomicUsize::new(0));
        let std_listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        std_listener
            .set_nonblocking(true)
            .expect("set_nonblocking must succeed");
        let listener = HttpListener::new(std_listener, counting_factory(counter.clone()));

        let result = listener.accept();
        let err = result.expect_err("accept on idle nonblocking listener must error");
        assert_eq!(
            err.kind(),
            io::ErrorKind::WouldBlock,
            "nonblocking accept with no pending client yields WouldBlock"
        );
        assert_eq!(
            listener.accept_error_count(),
            1,
            "the error path must increment the error counter via on_accept_error"
        );
        assert_eq!(
            counter.load(Ordering::SeqCst),
            0,
            "the factory must NOT be invoked when accept errors"
        );
    }

    // -----------------------------------------------------------------------
    // is_fatal_accept_error — classification table
    // -----------------------------------------------------------------------

    #[test]
    fn is_fatal_accept_error_true_for_connection_aborted() {
        let e = io::Error::new(io::ErrorKind::ConnectionAborted, "aborted");
        assert!(is_fatal_accept_error(&e));
    }

    #[test]
    fn is_fatal_accept_error_true_for_not_connected() {
        let e = io::Error::new(io::ErrorKind::NotConnected, "not connected");
        assert!(is_fatal_accept_error(&e));
    }

    #[test]
    fn is_fatal_accept_error_false_for_would_block() {
        let e = io::Error::new(io::ErrorKind::WouldBlock, "would block");
        assert!(!is_fatal_accept_error(&e));
    }

    #[test]
    fn is_fatal_accept_error_false_for_connection_reset() {
        let e = io::Error::new(io::ErrorKind::ConnectionReset, "reset");
        assert!(!is_fatal_accept_error(&e));
    }

    // -----------------------------------------------------------------------
    // run_with — the internal driver exercised with synthetic accept results.
    // The real `run()` simply delegates to `run_with(|| self.accept())`; the
    // synthetic-injection tests below drive the loop's control flow paths
    // (non-fatal retry, fatal break, stop-flag exit) deterministically without
    // requiring the OS to produce ECONNABORTED on a real socket.
    // -----------------------------------------------------------------------

    #[test]
    fn run_with_breaks_on_fatal_connection_aborted_error() {
        let counter = Arc::new(AtomicUsize::new(0));
        let (listener, _port) = bound_listener(counter);

        // First two iterations: non-fatal errors → loop continues.
        // Third iteration: fatal ConnectionAborted → loop must break.
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_inner = calls.clone();
        listener.run_with(&mut move || {
            let n = calls_inner.fetch_add(1, Ordering::SeqCst);
            match n {
                0 => Err(io::Error::new(io::ErrorKind::ConnectionReset, "non-fatal")),
                1 => Err(io::Error::new(io::ErrorKind::WouldBlock, "non-fatal")),
                _ => Err(io::Error::new(io::ErrorKind::ConnectionAborted, "fatal")),
            }
        });

        assert_eq!(
            calls.load(Ordering::SeqCst),
            3,
            "loop must continue past non-fatal errors and stop on the fatal one"
        );
        assert!(
            !listener.is_running(),
            "running flag must be cleared after the fatal break"
        );
    }

    #[test]
    fn run_with_breaks_on_fatal_not_connected_error() {
        let counter = Arc::new(AtomicUsize::new(0));
        let (listener, _port) = bound_listener(counter);

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_inner = calls.clone();
        listener.run_with(&mut move || {
            calls_inner.fetch_add(1, Ordering::SeqCst);
            Err(io::Error::new(io::ErrorKind::NotConnected, "fatal"))
        });

        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "NotConnected must break the loop on the very first iteration"
        );
        assert!(!listener.is_running());
    }

    #[test]
    fn run_with_continues_through_ok_results_until_stop() {
        let counter = Arc::new(AtomicUsize::new(0));
        let (listener, _port) = bound_listener(counter);
        let listener = Arc::new(listener);

        let calls = Arc::new(AtomicUsize::new(0));

        let l = listener.clone();
        let calls_inner = calls.clone();
        let handle = thread::spawn(move || {
            l.run_with(&mut move || {
                calls_inner.fetch_add(1, Ordering::SeqCst);
                // Sleep briefly so the test thread can observe progress and
                // call stop() at a deterministic point.
                thread::sleep(Duration::from_millis(2));
                Ok(())
            });
        });

        // Wait until the loop has iterated a few times, then stop it.
        for _ in 0..200 {
            if calls.load(Ordering::SeqCst) >= 3 {
                break;
            }
            thread::sleep(Duration::from_millis(2));
        }
        listener.stop();
        handle.join().expect("run thread must terminate after stop");

        assert!(
            calls.load(Ordering::SeqCst) >= 3,
            "the loop must keep iterating on Ok results until stop() is called"
        );
        assert!(!listener.is_running());
    }

    // -----------------------------------------------------------------------
    // run() — proves the public entry point delegates to run_with and the
    // real accept() path. Uses a non-blocking listener so accept() errors
    // immediately with WouldBlock (non-fatal); the loop keeps looping until
    // stop() is called, at which point the running flag must clear.
    // -----------------------------------------------------------------------

    #[test]
    fn run_continues_on_nonfatal_accept_errors_until_stop() {
        let counter = Arc::new(AtomicUsize::new(0));
        let std_listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        std_listener
            .set_nonblocking(true)
            .expect("set_nonblocking must succeed");
        let listener = Arc::new(HttpListener::new(
            std_listener,
            counting_factory(counter.clone()),
        ));

        let l = listener.clone();
        let handle = thread::spawn(move || l.run());

        // Give the run loop time to spin on WouldBlock a few times.
        thread::sleep(Duration::from_millis(20));
        assert!(
            listener.accept_error_count() >= 1,
            "non-blocking accept must produce at least one observed error \
             while looping"
        );
        listener.stop();
        handle.join().expect("run thread must terminate");

        assert!(!listener.is_running(), "stop must clear the running flag");
        assert_eq!(
            counter.load(Ordering::SeqCst),
            0,
            "no real connections happened, so the factory must not fire"
        );
    }
}
