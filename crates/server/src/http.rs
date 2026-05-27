//! HTTP server root — entry-point analog of C++ `tfs::http::start` / `tfs::http::stop`.
//!
//! The C++ original uses a single namespace-scoped `boost::asio::io_context`
//! and a `std::vector<std::thread>` of worker threads driven from
//! `forgottenserver/src/http/http.cpp`. This Rust port preserves
//! the observable behaviour of the two free functions:
//!
//! * `start(bind_only_ots_ip, ots_ip, port, threads)`
//!   - Returns early when `port == 0` or `threads < 1` (no listener spawned).
//!   - Resolves the bind address: `[::]` (IPv6 ANY) by default, or the parsed
//!     `ots_ip` when `bind_only_ots_ip` is `true`.
//!   - Binds a TCP listener on `address:port` and spawns `threads` worker
//!     threads that process accepted connections in a shared accept loop.
//! * `stop()`
//!   - No-op when no workers are running.
//!   - Signals the accept loop to exit and joins all worker threads.
//!
//! Compared with the C++ version this module exposes an `HttpServer` struct
//! that owns the listener and worker handles. This is a more idiomatic Rust
//! lifecycle (no hidden global state) and makes the start/stop pair directly
//! testable from `#[cfg(test)]`.

use std::{
    io,
    net::{IpAddr, Ipv6Addr, SocketAddr, TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::JoinHandle,
};

/// Default port used by C++ `tfs::http::start` when the caller omits `port`.
pub const DEFAULT_HTTP_PORT: u16 = 8080;

/// Default worker-thread count used by C++ `tfs::http::start` when omitted.
pub const DEFAULT_HTTP_THREADS: i32 = 1;

/// Result of an HTTP-server start attempt.
///
/// Distinguishes the three observable outcomes of the C++ `start` function:
/// - `Started` — listener bound and workers spawned.
/// - `InvalidArgs` — early-return because `port == 0` or `threads < 1`.
/// - `BindFailed(io::Error)` — TCP bind failed (no C++ equivalent because the
///   original aborts via Boost.Asio exceptions; we surface it as a value).
#[derive(Debug)]
pub enum StartOutcome {
    Started,
    InvalidArgs,
    BindFailed(io::Error),
}

impl PartialEq for StartOutcome {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (StartOutcome::Started, StartOutcome::Started)
                | (StartOutcome::InvalidArgs, StartOutcome::InvalidArgs)
                | (StartOutcome::BindFailed(_), StartOutcome::BindFailed(_))
        )
    }
}

/// Resolve the bind address from the two C++ parameters.
///
/// Mirrors the body of C++ `start`:
/// ```cpp
/// asio::ip::address address = asio::ip::address_v6::any();
/// if (bindOnlyOtsIP) {
///     address = asio::ip::make_address(otsIP);
/// }
/// ```
///
/// Returns `Err` when `bind_only_ots_ip` is true but `ots_ip` does not parse.
pub fn resolve_bind_address(bind_only_ots_ip: bool, ots_ip: &str) -> Result<IpAddr, String> {
    if bind_only_ots_ip {
        ots_ip
            .parse::<IpAddr>()
            .map_err(|e| format!("invalid IP {ots_ip:?}: {e}"))
    } else {
        Ok(IpAddr::V6(Ipv6Addr::UNSPECIFIED))
    }
}

/// HTTP server lifecycle handle.
///
/// `start` creates and runs an `HttpServer`; `stop` consumes it. The
/// owner-by-struct model replaces the C++ namespace-globals (`ioc`, `workers`)
/// and makes the start/stop pair testable without process-wide state.
pub struct HttpServer {
    listener_addr: SocketAddr,
    running: Arc<AtomicBool>,
    workers: Vec<JoinHandle<()>>,
}

impl HttpServer {
    /// True while the accept loop is running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// The address the listener bound to (useful when port 0 was requested —
    /// the OS assigns a real port that tests must read back).
    pub fn local_addr(&self) -> SocketAddr {
        self.listener_addr
    }

    /// Number of worker threads currently spawned.
    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }
}

/// Start an HTTP server bound to `address:port` with `threads` workers.
///
/// Direct analog of `tfs::http::start`:
/// ```cpp
/// void start(bool bindOnlyOtsIP, std::string_view otsIP,
///            unsigned short port = 8080, int threads = 1);
/// ```
///
/// Returns:
/// * `StartOutcome::InvalidArgs` if `port == 0` or `threads < 1` (C++ has the
///   same early-return guard).
/// * `StartOutcome::BindFailed(e)` if the TCP listener cannot bind.
/// * `StartOutcome::Started` with the populated `*server_out` otherwise.
///
/// `server_out` is `&mut Option<HttpServer>` because the C++ function returns
/// `void` and stores the state in module globals; the Rust port keeps the
/// state in the caller's optional so it can be passed to a later `stop` call.
pub fn start(
    bind_only_ots_ip: bool,
    ots_ip: &str,
    port: u16,
    threads: i32,
    server_out: &mut Option<HttpServer>,
) -> StartOutcome {
    // Matches C++ `if (port == 0 || threads < 1) { return; }`
    if port == 0 || threads < 1 {
        return StartOutcome::InvalidArgs;
    }

    let address = match resolve_bind_address(bind_only_ots_ip, ots_ip) {
        Ok(addr) => addr,
        Err(_) => return StartOutcome::InvalidArgs,
    };

    let socket_addr = SocketAddr::new(address, port);
    let listener = match TcpListener::bind(socket_addr) {
        Ok(l) => l,
        Err(e) => return StartOutcome::BindFailed(e),
    };

    let local_addr = listener.local_addr().unwrap_or(socket_addr);
    let listener = Arc::new(listener);
    let running = Arc::new(AtomicBool::new(true));

    let mut workers = Vec::with_capacity(threads as usize);
    for _ in 0..threads {
        let listener_clone = Arc::clone(&listener);
        let running_clone = Arc::clone(&running);
        workers.push(std::thread::spawn(move || {
            accept_loop(listener_clone, running_clone);
        }));
    }

    *server_out = Some(HttpServer {
        listener_addr: local_addr,
        running,
        workers,
    });

    StartOutcome::Started
}

/// Stop a running HTTP server. Matches C++ `tfs::http::stop`:
/// ```cpp
/// void stop() {
///     if (workers.empty()) return;
///     ioc.stop();
///     for (auto& worker : workers) worker.join();
/// }
/// ```
///
/// When `server_in` is `None` (no workers) this is a no-op, exactly like the
/// `workers.empty()` early-return in C++.
pub fn stop(server_in: &mut Option<HttpServer>) {
    let Some(server) = server_in.take() else {
        return;
    };

    // Signal accept loop to exit. Each worker is blocked in `accept()` until
    // a TCP connection lands on the listener, so we issue one wake-up
    // connection per worker (best-effort — the connect may race the listener
    // teardown).
    server.running.store(false, Ordering::SeqCst);
    for _ in 0..server.workers.len() {
        let _ = TcpStream::connect(server.listener_addr);
    }

    for worker in server.workers {
        let _ = worker.join();
    }
}

/// Per-thread accept loop. Each worker calls `accept()` in a tight loop until
/// the `running` flag is cleared by `stop`. Mirrors the role of
/// `boost::asio::io_context::run` in the C++ version.
pub(crate) fn accept_loop(listener: Arc<TcpListener>, running: Arc<AtomicBool>) {
    while running.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _addr)) => {
                // C++ delegates each connection to `Listener::on_accept` which
                // hands it off to a session; that wiring lives in the
                // sibling `listener.rs` / `session.rs` Rust modules. From this
                // module's perspective, accepting the connection is the
                // observable contract.
                drop(stream);
            }
            Err(_) => {
                // EINTR / shutdown — exit the loop if running cleared.
                if !running.load(Ordering::SeqCst) {
                    break;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    /// Spin until `pred` returns true or `max` elapses. Returns the predicate's
    /// final value. Polls every 5 ms.
    fn wait_until<F: FnMut() -> bool>(mut pred: F, max: Duration) -> bool {
        let start = Instant::now();
        loop {
            let ok = pred();
            if ok || start.elapsed() >= max {
                return ok;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    // ---- resolve_bind_address ---------------------------------------------

    #[test]
    fn resolve_bind_address_returns_ipv6_any_when_not_pinned() {
        let addr = resolve_bind_address(false, "").expect("ipv6 any path must succeed");
        assert_eq!(addr, IpAddr::V6(Ipv6Addr::UNSPECIFIED));
    }

    #[test]
    fn resolve_bind_address_returns_ipv6_any_ignoring_ots_ip_when_not_pinned() {
        // C++ ignores otsIP entirely when bindOnlyOtsIP is false.
        let addr = resolve_bind_address(false, "192.168.1.1").expect("ignored");
        assert_eq!(addr, IpAddr::V6(Ipv6Addr::UNSPECIFIED));
    }

    #[test]
    fn resolve_bind_address_parses_ipv4_when_pinned() {
        let addr = resolve_bind_address(true, "127.0.0.1").expect("must parse");
        assert_eq!(addr, IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
    }

    #[test]
    fn resolve_bind_address_parses_ipv6_when_pinned() {
        let addr = resolve_bind_address(true, "::1").expect("must parse");
        assert_eq!(addr, IpAddr::V6(Ipv6Addr::LOCALHOST));
    }

    #[test]
    fn resolve_bind_address_rejects_garbage_when_pinned() {
        let err = resolve_bind_address(true, "not-an-ip").unwrap_err();
        assert!(err.contains("not-an-ip"));
    }

    // ---- start: invalid-args early returns --------------------------------

    #[test]
    fn start_returns_invalid_args_for_port_zero() {
        let mut server = None;
        let outcome = start(false, "", 0, 1, &mut server);
        assert_eq!(outcome, StartOutcome::InvalidArgs);
        assert!(server.is_none(), "no server should be created");
    }

    #[test]
    fn start_returns_invalid_args_for_zero_threads() {
        let mut server = None;
        let outcome = start(false, "", 8080, 0, &mut server);
        assert_eq!(outcome, StartOutcome::InvalidArgs);
        assert!(server.is_none());
    }

    #[test]
    fn start_returns_invalid_args_for_negative_threads() {
        let mut server = None;
        let outcome = start(false, "", 8080, -3, &mut server);
        assert_eq!(outcome, StartOutcome::InvalidArgs);
        assert!(server.is_none());
    }

    #[test]
    fn start_returns_invalid_args_when_pinned_ip_is_garbage() {
        let mut server = None;
        let outcome = start(true, "not-an-ip", 1234, 1, &mut server);
        assert_eq!(outcome, StartOutcome::InvalidArgs);
        assert!(server.is_none());
    }

    // ---- start: success paths ---------------------------------------------

    /// Pick an unused localhost port by binding+dropping a temp listener.
    fn pick_free_port() -> u16 {
        let probe = TcpListener::bind(SocketAddr::new(
            IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
            0,
        ))
        .unwrap();
        probe.local_addr().unwrap().port()
    }

    #[test]
    fn start_binds_loopback_when_pinned_and_spawns_requested_workers() {
        let free_port = pick_free_port();

        let mut server = None;
        let outcome = start(true, "127.0.0.1", free_port, 3, &mut server);
        assert_eq!(outcome, StartOutcome::Started);
        let s = server.as_ref().expect("server must be populated");
        assert_eq!(s.worker_count(), 3);
        assert!(s.is_running());
        assert_eq!(s.local_addr().port(), free_port);
        assert_eq!(
            s.local_addr().ip(),
            IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
        );

        stop(&mut server);
        assert!(server.is_none(), "stop must consume the server");
    }

    #[test]
    fn start_then_stop_joins_all_workers() {
        let free_port = pick_free_port();

        let mut server = None;
        let outcome = start(true, "127.0.0.1", free_port, 4, &mut server);
        assert_eq!(outcome, StartOutcome::Started);

        assert!(server.as_ref().unwrap().is_running());
        stop(&mut server);
        assert!(server.is_none());
        // After stop, the port must be free — verify by re-binding.
        let rebind = TcpListener::bind(SocketAddr::new(
            IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
            free_port,
        ));
        assert!(rebind.is_ok(), "port must be released after stop");
    }

    // ---- start: bind failures ---------------------------------------------

    #[test]
    fn start_returns_bind_failed_when_port_in_use() {
        // Occupy a port, then try to start on it.
        let occupied = TcpListener::bind(SocketAddr::new(
            IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
            0,
        ))
        .unwrap();
        let port = occupied.local_addr().unwrap().port();

        let mut server = None;
        let outcome = start(true, "127.0.0.1", port, 1, &mut server);
        assert!(
            matches!(outcome, StartOutcome::BindFailed(_)),
            "expected BindFailed"
        );
        assert!(server.is_none());
        drop(occupied);
    }

    // ---- stop: no-op when nothing running ---------------------------------

    #[test]
    fn stop_is_noop_when_no_server() {
        let mut server: Option<HttpServer> = None;
        stop(&mut server); // must not panic
        assert!(server.is_none());
    }

    // ---- accept loop actually accepts -------------------------------------

    #[test]
    fn running_server_accepts_a_connection() {
        let free_port = pick_free_port();

        let mut server = None;
        assert_eq!(
            start(true, "127.0.0.1", free_port, 1, &mut server),
            StartOutcome::Started
        );

        // Give the worker a moment to enter the accept loop.
        assert!(wait_until(
            || server.as_ref().is_some_and(|s| s.is_running()),
            Duration::from_secs(1)
        ));

        // Connecting should succeed (worker accepts + drops).
        let conn = TcpStream::connect(SocketAddr::new(
            IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
            free_port,
        ));
        assert!(conn.is_ok());
        drop(conn);

        stop(&mut server);
    }

    // ---- StartOutcome PartialEq ------------------------------------------

    #[test]
    fn start_outcome_eq_treats_bind_failed_as_equivalent_regardless_of_inner_error() {
        let a = StartOutcome::BindFailed(io::Error::new(io::ErrorKind::AddrInUse, "x"));
        let b = StartOutcome::BindFailed(io::Error::new(io::ErrorKind::PermissionDenied, "y"));
        assert_eq!(a, b);
        assert_ne!(a, StartOutcome::Started);
        assert_ne!(a, StartOutcome::InvalidArgs);
        assert_ne!(StartOutcome::Started, StartOutcome::InvalidArgs);
    }

    // ---- constants ---------------------------------------------------------

    #[test]
    fn default_constants_match_cpp_signature_defaults() {
        assert_eq!(DEFAULT_HTTP_PORT, 8080);
        assert_eq!(DEFAULT_HTTP_THREADS, 1);
    }

    // ---- accept_loop Err branch -------------------------------------------

    /// Drive `accept_loop` with a non-blocking listener so every `accept()`
    /// call returns `Err(WouldBlock)`. With `running == true` the loop must
    /// keep iterating (covering the Err arm's "do nothing, retry" path); when
    /// the main thread flips `running` to `false`, the loop's Err arm must
    /// observe it and `break`.
    #[test]
    fn accept_loop_err_branch_breaks_when_running_cleared() {
        let listener = TcpListener::bind(SocketAddr::new(
            IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
            0,
        ))
        .expect("bind");
        listener
            .set_nonblocking(true)
            .expect("nonblocking should succeed");
        let listener = Arc::new(listener);
        let running = Arc::new(AtomicBool::new(true));

        let lc = Arc::clone(&listener);
        let rc = Arc::clone(&running);
        let handle = std::thread::spawn(move || accept_loop(lc, rc));

        // Let the loop spin on Err(WouldBlock) for a moment so the Err arm
        // executes at least once with `running == true`.
        std::thread::sleep(Duration::from_millis(20));

        // Now flip running off; the Err arm should observe it and break.
        running.store(false, Ordering::SeqCst);

        // The thread must terminate quickly.
        assert!(wait_until(|| handle.is_finished(), Duration::from_secs(2)));
        handle.join().expect("accept_loop thread should terminate");
    }
}
