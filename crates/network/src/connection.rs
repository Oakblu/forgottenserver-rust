//! Connection abstraction over generic Read/Write handles.
//!
//! Migrated from forgottenserver connection.h / connection.cpp.
//!
//! Uses `std::io::{Read, Write}` generics so that tests can use
//! `std::io::Cursor<Vec<u8>>` without real sockets.

use std::collections::VecDeque;
use std::io::{self, Read, Write};

use forgottenserver_common::networkmessage::NetworkMessage;
use forgottenserver_common::outputmessage::OutputMessage;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Runtime configuration for a [`Connection`].
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// Read timeout in milliseconds (informational — not enforced in tests).
    pub read_timeout_ms: u64,
    /// Maximum number of bytes allowed in the output buffer.
    pub max_output_buffer: usize,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            read_timeout_ms: 30_000,
            max_output_buffer: 65_536,
        }
    }
}

// ---------------------------------------------------------------------------
// ConnectionState — mirrors C++ `ConnectionState_t`
// ---------------------------------------------------------------------------

/// Lifecycle state of a connection.
///
/// Maps to the C++ `ConnectionState_t` enum defined in `connection.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Connection accepted but not yet classified (initial state).
    Pending,
    /// Waiting for character-list credentials.
    RequestCharlist,
    /// Performing game-world authentication handshake.
    GameworldAuth,
    /// Fully authenticated — in-game.
    Game,
    /// Connection has been closed (gracefully or forcefully).
    Disconnected,
    /// Close was requested but the send queue is not yet drained.
    RequestClose,
}

// ---------------------------------------------------------------------------
// Connection
// ---------------------------------------------------------------------------

/// A protocol connection backed by generic `Read` and `Write` handles.
pub struct Connection<R: Read, W: Write> {
    reader: R,
    writer: W,
    config: ConnectionConfig,
    state: ConnectionState,
    /// Outbound message queue for back-pressure management.
    ///
    /// C++: `std::list<OutputMessage_ptr> messageQueue`
    message_queue: VecDeque<Vec<u8>>,
    /// XTEA session key received during handshake (4 × u32).
    ///
    /// C++: stored by the protocol layer and passed back to the connection.
    session_key: Option<[u32; 4]>,
}

impl<R: Read, W: Write> Connection<R, W> {
    /// Creates a new connection in the [`ConnectionState::Pending`] state.
    pub fn new(reader: R, writer: W, config: ConnectionConfig) -> Self {
        Self {
            reader,
            writer,
            config,
            state: ConnectionState::Pending,
            message_queue: VecDeque::new(),
            session_key: None,
        }
    }

    // -----------------------------------------------------------------------
    // State accessors
    // -----------------------------------------------------------------------

    /// Returns the current [`ConnectionState`].
    pub fn state(&self) -> ConnectionState {
        self.state
    }

    /// Returns `true` when the connection is in the
    /// [`ConnectionState::Disconnected`] state.
    pub fn is_closed(&self) -> bool {
        self.state == ConnectionState::Disconnected
    }

    /// Returns `true` when a close has been requested but the send queue
    /// is not yet drained ([`ConnectionState::RequestClose`]).
    pub fn is_request_close(&self) -> bool {
        self.state == ConnectionState::RequestClose
    }

    // -----------------------------------------------------------------------
    // State transitions
    // -----------------------------------------------------------------------

    /// Immediately closes the connection.
    ///
    /// Sets the state to [`ConnectionState::Disconnected`] regardless of the
    /// current state (idempotent).
    ///
    /// Mirrors C++ `Connection::close(force = true)` which sets
    /// `connectionState = CONNECTION_STATE_DISCONNECTED`.
    pub fn close(&mut self) {
        self.state = ConnectionState::Disconnected;
    }

    /// Requests a graceful close.
    ///
    /// If the send queue is non-empty the connection moves to
    /// [`ConnectionState::RequestClose`] so that the remaining messages can be
    /// flushed before the socket is shut down.  If the queue is already empty
    /// it transitions directly to [`ConnectionState::Disconnected`].
    ///
    /// Idempotent: calling on an already-closed connection is a no-op.
    pub fn request_close(&mut self) {
        match self.state {
            ConnectionState::Disconnected => {}
            _ => {
                if self.message_queue.is_empty() {
                    self.state = ConnectionState::Disconnected;
                } else {
                    self.state = ConnectionState::RequestClose;
                }
            }
        }
    }

    /// Transitions to [`ConnectionState::RequestCharlist`] from
    /// [`ConnectionState::Pending`].
    pub fn accept_pending(&mut self) {
        if self.state == ConnectionState::Pending {
            self.state = ConnectionState::RequestCharlist;
        }
    }

    /// Transitions to [`ConnectionState::GameworldAuth`].
    pub fn accept_with_protocol(&mut self) {
        self.state = ConnectionState::GameworldAuth;
    }

    /// Transitions from [`ConnectionState::GameworldAuth`] to
    /// [`ConnectionState::Game`] once the handshake is complete.
    pub fn handshake_complete(&mut self) {
        if self.state == ConnectionState::GameworldAuth {
            self.state = ConnectionState::Game;
        }
    }

    // -----------------------------------------------------------------------
    // Session key (XTEA handshake)
    // -----------------------------------------------------------------------

    /// Stores the four-word XTEA session key received during the handshake.
    ///
    /// C++: set by `Protocol::onRecvFirstMessage` and used by
    /// `OutputMessage` encryption helpers.
    pub fn set_session_key(&mut self, key: [u32; 4]) {
        self.session_key = Some(key);
    }

    /// Returns the stored XTEA session key, or `None` if the handshake has
    /// not yet completed.
    pub fn session_key(&self) -> Option<[u32; 4]> {
        self.session_key
    }

    // -----------------------------------------------------------------------
    // Send with back-pressure queue
    // -----------------------------------------------------------------------

    /// Enqueues an `OutputMessage` for sending.
    ///
    /// The raw bytes are captured into the internal queue.  Callers can then
    /// call [`flush_queue`] to actually write pending bytes to the underlying
    /// writer.
    ///
    /// Returns `Err` when:
    /// - the connection is [`ConnectionState::Disconnected`], or
    /// - the message payload exceeds `config.max_output_buffer`.
    ///
    /// C++: `messageQueue.emplace_back(msg)` with async-write initiated when
    /// the queue was previously empty.
    pub fn send(&mut self, msg: &OutputMessage) -> io::Result<()> {
        if self.state == ConnectionState::Disconnected {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "connection is closed",
            ));
        }
        let buf = msg.get_output_buffer();
        if buf.len() > self.config.max_output_buffer {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "output buffer exceeds maximum allowed size",
            ));
        }
        self.message_queue.push_back(buf.to_vec());
        Ok(())
    }

    /// Flushes all queued messages to the underlying writer.
    ///
    /// If a graceful close was requested ([`ConnectionState::RequestClose`])
    /// and the queue becomes empty after flushing, the connection transitions
    /// to [`ConnectionState::Disconnected`].
    pub fn flush_queue(&mut self) -> io::Result<()> {
        while let Some(buf) = self.message_queue.pop_front() {
            self.writer.write_all(&buf)?;
        }
        // Finish graceful close once the queue is drained.
        if self.state == ConnectionState::RequestClose && self.message_queue.is_empty() {
            self.state = ConnectionState::Disconnected;
        }
        Ok(())
    }

    /// Returns the number of messages currently waiting in the send queue.
    pub fn queue_len(&self) -> usize {
        self.message_queue.len()
    }

    // -----------------------------------------------------------------------
    // Receive
    // -----------------------------------------------------------------------

    /// Reads exactly `expected_len` bytes from the underlying reader and
    /// returns them as a `NetworkMessage`.
    ///
    /// Returns `Err` if the read fails or the connection is closed.
    pub fn recv_message(&mut self, expected_len: usize) -> io::Result<NetworkMessage> {
        if self.state == ConnectionState::Disconnected {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "connection is closed",
            ));
        }
        let mut buf = vec![0u8; expected_len];
        self.reader.read_exact(&mut buf)?;

        let mut msg = NetworkMessage::new();
        msg.add_bytes(&buf);
        msg.set_buffer_position(0); // reset read cursor to start of payload
        Ok(msg)
    }
}

// ---------------------------------------------------------------------------
// parseHeader/parsePacket + getIP helpers (Session 34 ledger closure)
// ---------------------------------------------------------------------------

/// Strip the 2-byte little-endian length prefix from a raw socket
/// buffer and return the payload slice. Mirrors the combined behaviour
/// of C++ `Connection::parseHeader` (reads the 2-byte u16 length) +
/// `Connection::parsePacket` (reads the body and trusts the length).
///
/// Rust returns the payload slice instead of writing into an internal
/// buffer because the per-port handlers (`status_handler`,
/// `admin_handler`) own the buffer and just need the body bytes. The
/// validation rules match the C++ guard chain:
///
/// * Buffer too short to contain the 2-byte prefix → `None`.
/// * The advertised length is larger than the remaining buffer
///   (truncated read) → `None`.
/// * Otherwise → `Some(&buf[2..2+len])` with `len = u16::from_le_bytes`.
///
/// The "trailing extra bytes" case (declared length < buffer remainder)
/// is allowed — the C++ side reads exactly `length` bytes and discards
/// the rest. Caller may slice further if it cares.
pub fn parse_length_prefixed_packet(buf: &[u8]) -> Option<&[u8]> {
    if buf.len() < 2 {
        return None;
    }
    let len = u16::from_le_bytes([buf[0], buf[1]]) as usize;
    let end = 2usize.checked_add(len)?;
    if end > buf.len() {
        return None;
    }
    Some(&buf[2..end])
}

/// Convert a socket address into the u32 IPv4 representation that C++
/// `Connection::getIP()` returns. IPv6 addresses map to `0` (matching
/// the C++ `ip::address_v4()` cast on a non-v4 endpoint — the original
/// codebase doesn't run on v6-only sockets in practice).
pub fn peer_ip_to_u32(addr: std::net::SocketAddr) -> u32 {
    match addr.ip() {
        std::net::IpAddr::V4(v4) => u32::from_be_bytes(v4.octets()),
        std::net::IpAddr::V6(_) => 0,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn make_conn(data: Vec<u8>) -> Connection<Cursor<Vec<u8>>, Vec<u8>> {
        Connection::new(Cursor::new(data), Vec::new(), ConnectionConfig::default())
    }

    // -----------------------------------------------------------------------
    // Initial state
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_is_not_closed() {
        let conn = make_conn(vec![]);
        assert!(!conn.is_closed());
    }

    #[test]
    fn test_new_state_is_pending() {
        let conn = make_conn(vec![]);
        assert_eq!(conn.state(), ConnectionState::Pending);
    }

    #[test]
    fn test_new_has_no_session_key() {
        let conn = make_conn(vec![]);
        assert!(conn.session_key().is_none());
    }

    #[test]
    fn test_new_queue_is_empty() {
        let conn = make_conn(vec![]);
        assert_eq!(conn.queue_len(), 0);
    }

    // -----------------------------------------------------------------------
    // close → state = Disconnected, is_closed = true
    // -----------------------------------------------------------------------

    #[test]
    fn test_close_marks_closed() {
        let mut conn = make_conn(vec![]);
        conn.close();
        assert!(conn.is_closed());
    }

    #[test]
    fn test_close_sets_disconnected_state() {
        let mut conn = make_conn(vec![]);
        conn.close();
        assert_eq!(conn.state(), ConnectionState::Disconnected);
    }

    #[test]
    fn test_close_idempotent() {
        let mut conn = make_conn(vec![]);
        conn.close();
        conn.close(); // second call must not panic or change state
        assert!(conn.is_closed());
        assert_eq!(conn.state(), ConnectionState::Disconnected);
    }

    // -----------------------------------------------------------------------
    // request_close
    // -----------------------------------------------------------------------

    #[test]
    fn test_request_close_with_empty_queue_disconnects() {
        let mut conn = make_conn(vec![]);
        // queue is empty → should go directly to Disconnected
        conn.request_close();
        assert_eq!(conn.state(), ConnectionState::Disconnected);
        assert!(conn.is_closed());
    }

    #[test]
    fn test_request_close_with_queued_message_sets_request_close() {
        let mut conn = make_conn(vec![]);
        let mut msg = OutputMessage::new();
        msg.add_u8(0xAA);
        msg.write_message_length();
        conn.send(&msg).unwrap(); // enqueue without flushing
        conn.request_close();
        assert_eq!(conn.state(), ConnectionState::RequestClose);
        assert!(conn.is_request_close());
        assert!(!conn.is_closed());
    }

    #[test]
    fn test_request_close_on_disconnected_is_noop() {
        let mut conn = make_conn(vec![]);
        conn.close(); // already Disconnected
        conn.request_close(); // must be a no-op
        assert_eq!(conn.state(), ConnectionState::Disconnected);
    }

    #[test]
    fn test_flush_queue_after_request_close_transitions_to_disconnected() {
        let mut conn = make_conn(vec![]);
        let mut msg = OutputMessage::new();
        msg.add_u8(0xBB);
        msg.write_message_length();
        conn.send(&msg).unwrap();
        conn.request_close();
        assert_eq!(conn.state(), ConnectionState::RequestClose);
        conn.flush_queue().unwrap();
        assert_eq!(conn.state(), ConnectionState::Disconnected);
    }

    // -----------------------------------------------------------------------
    // is_closed returns correct values per state
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_closed_false_for_pending() {
        let conn = make_conn(vec![]);
        assert!(!conn.is_closed());
    }

    #[test]
    fn test_is_closed_false_for_request_charlist() {
        let mut conn = make_conn(vec![]);
        conn.accept_pending();
        assert_eq!(conn.state(), ConnectionState::RequestCharlist);
        assert!(!conn.is_closed());
    }

    #[test]
    fn test_is_closed_false_for_game() {
        let mut conn = make_conn(vec![]);
        conn.accept_with_protocol();
        conn.handshake_complete();
        assert_eq!(conn.state(), ConnectionState::Game);
        assert!(!conn.is_closed());
    }

    #[test]
    fn test_is_closed_false_for_request_close() {
        let mut conn = make_conn(vec![]);
        let mut msg = OutputMessage::new();
        msg.add_u8(0x01);
        msg.write_message_length();
        conn.send(&msg).unwrap();
        conn.request_close();
        assert!(!conn.is_closed());
    }

    #[test]
    fn test_is_closed_true_for_disconnected() {
        let mut conn = make_conn(vec![]);
        conn.close();
        assert!(conn.is_closed());
    }

    // -----------------------------------------------------------------------
    // send writes bytes to queue (back-pressure)
    // -----------------------------------------------------------------------

    #[test]
    fn test_send_enqueues_message() {
        let mut conn = make_conn(vec![]);
        let mut msg = OutputMessage::new();
        msg.add_u8(0xAB);
        msg.write_message_length();
        conn.send(&msg).expect("send should succeed");
        assert_eq!(conn.queue_len(), 1);
    }

    #[test]
    fn test_send_multiple_messages_queues_all() {
        let mut conn = make_conn(vec![]);
        for _ in 0..3 {
            let mut msg = OutputMessage::new();
            msg.add_u8(0x01);
            msg.write_message_length();
            conn.send(&msg).unwrap();
        }
        assert_eq!(conn.queue_len(), 3);
    }

    #[test]
    fn test_flush_queue_writes_to_writer_and_empties_queue() {
        let mut conn = make_conn(vec![]);
        let mut msg = OutputMessage::new();
        msg.add_u8(0xAB);
        msg.add_u8(0xCD);
        msg.write_message_length();
        conn.send(&msg).unwrap();
        conn.flush_queue().unwrap();
        assert_eq!(conn.queue_len(), 0);
        assert!(!conn.writer.is_empty());
    }

    #[test]
    fn test_flush_queue_writes_correct_bytes() {
        let mut conn = make_conn(vec![]);
        let mut msg = OutputMessage::new();
        msg.add_u8(0xAB);
        msg.add_u8(0xCD);
        msg.write_message_length();
        let expected = msg.get_output_buffer().to_vec();
        conn.send(&msg).unwrap();
        conn.flush_queue().unwrap();
        assert_eq!(&conn.writer, &expected);
    }

    #[test]
    fn test_send_after_close_returns_error() {
        let mut conn = make_conn(vec![]);
        conn.close();

        let msg = OutputMessage::new();
        let result = conn.send(&msg);
        assert!(result.is_err(), "send after close should return an error");
    }

    #[test]
    fn test_send_oversized_message_returns_error() {
        let cfg = ConnectionConfig {
            max_output_buffer: 2,
            ..Default::default()
        };
        let mut conn: Connection<Cursor<Vec<u8>>, Vec<u8>> =
            Connection::new(Cursor::new(vec![]), Vec::new(), cfg);

        let mut msg = OutputMessage::new();
        msg.add_u8(0x01);
        msg.add_u8(0x02);
        msg.add_u8(0x03); // 3 bytes payload + 2 header → exceeds max=2
        msg.write_message_length();
        let result = conn.send(&msg);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // Session key stored correctly after handshake
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_session_key_stores_key() {
        let mut conn = make_conn(vec![]);
        let key = [0x1234_5678_u32, 0xDEAD_BEEF, 0xCAFE_BABE, 0x0000_FFFF];
        conn.set_session_key(key);
        assert_eq!(conn.session_key(), Some(key));
    }

    #[test]
    fn test_set_session_key_overwrites_previous_key() {
        let mut conn = make_conn(vec![]);
        let first = [1u32, 2, 3, 4];
        let second = [5u32, 6, 7, 8];
        conn.set_session_key(first);
        conn.set_session_key(second);
        assert_eq!(conn.session_key(), Some(second));
    }

    #[test]
    fn test_session_key_none_before_handshake() {
        let conn = make_conn(vec![]);
        assert!(conn.session_key().is_none());
    }

    // -----------------------------------------------------------------------
    // recv_message reads from reader into NetworkMessage
    // -----------------------------------------------------------------------

    #[test]
    fn test_recv_message_reads_bytes() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let mut conn = make_conn(data.clone());

        let mut msg = conn.recv_message(4).expect("recv should succeed");
        let read_back = msg.get_bytes(4);
        assert_eq!(read_back, data);
    }

    #[test]
    fn test_recv_after_close_returns_error() {
        let mut conn = make_conn(vec![0, 0, 0, 0]);
        conn.close();

        let result = conn.recv_message(4);
        assert!(result.is_err(), "recv after close should return an error");
    }

    // -----------------------------------------------------------------------
    // State transitions: accept_pending, accept_with_protocol, handshake_complete
    // -----------------------------------------------------------------------

    #[test]
    fn test_accept_pending_transitions_from_pending() {
        let mut conn = make_conn(vec![]);
        conn.accept_pending();
        assert_eq!(conn.state(), ConnectionState::RequestCharlist);
    }

    #[test]
    fn test_accept_pending_noop_if_not_pending() {
        let mut conn = make_conn(vec![]);
        conn.accept_with_protocol(); // → GameworldAuth
        conn.accept_pending(); // should be a no-op
        assert_eq!(conn.state(), ConnectionState::GameworldAuth);
    }

    #[test]
    fn test_accept_with_protocol_sets_gameworld_auth() {
        let mut conn = make_conn(vec![]);
        conn.accept_with_protocol();
        assert_eq!(conn.state(), ConnectionState::GameworldAuth);
    }

    #[test]
    fn test_handshake_complete_transitions_to_game() {
        let mut conn = make_conn(vec![]);
        conn.accept_with_protocol();
        conn.handshake_complete();
        assert_eq!(conn.state(), ConnectionState::Game);
    }

    #[test]
    fn test_handshake_complete_noop_if_not_gameworld_auth() {
        let mut conn = make_conn(vec![]);
        // Not in GameworldAuth → should not change state
        conn.handshake_complete();
        assert_eq!(conn.state(), ConnectionState::Pending);
    }

    // -----------------------------------------------------------------------
    // ConnectionConfig defaults
    // -----------------------------------------------------------------------

    #[test]
    fn test_default_config() {
        let cfg = ConnectionConfig::default();
        assert_eq!(cfg.read_timeout_ms, 30_000);
        assert!(cfg.max_output_buffer > 0);
    }

    // -----------------------------------------------------------------------
    // Error paths: writer fails during flush_queue, reader fails during recv
    // -----------------------------------------------------------------------

    /// Writer that fails on every `write` call — used to exercise the
    /// `?` error-propagation path in [`Connection::flush_queue`].
    struct FailingWriter;
    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("writer failure"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_flush_queue_returns_error_when_writer_fails() {
        let mut conn: Connection<Cursor<Vec<u8>>, FailingWriter> = Connection::new(
            Cursor::new(vec![]),
            FailingWriter,
            ConnectionConfig::default(),
        );

        let mut msg = OutputMessage::new();
        msg.add_u8(0xEE);
        msg.write_message_length();
        conn.send(&msg).unwrap();

        let result = conn.flush_queue();
        assert!(result.is_err(), "flush_queue must surface writer errors");
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Other);

        // Ensure FailingWriter::flush is exercised so coverage is complete.
        let mut w = FailingWriter;
        assert!(w.flush().is_ok());
    }

    /// Reader that returns an error before any byte is read — used to
    /// exercise the `?` error-propagation path in
    /// [`Connection::recv_message`].
    struct FailingReader;
    impl Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "reader failure",
            ))
        }
    }

    #[test]
    fn test_recv_message_returns_error_when_reader_fails() {
        let mut conn: Connection<FailingReader, Vec<u8>> =
            Connection::new(FailingReader, Vec::new(), ConnectionConfig::default());

        let result = conn.recv_message(4);
        assert!(result.is_err(), "recv_message must surface reader errors");
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::ConnectionAborted);
    }

    // ── parse_length_prefixed_packet (Session 34) ───────────────────────

    /// Buffer with valid 2-byte LE prefix + matching body → Some(body).
    #[test]
    fn parse_length_prefixed_valid_returns_body_slice() {
        // length = 3 (0x03, 0x00) + payload bytes [0xAA, 0xBB, 0xCC]
        let buf = [0x03, 0x00, 0xAA, 0xBB, 0xCC];
        assert_eq!(
            parse_length_prefixed_packet(&buf),
            Some(&[0xAA, 0xBB, 0xCC][..])
        );
    }

    /// Empty buffer → None (no room for the 2-byte header).
    #[test]
    fn parse_length_prefixed_empty_buffer_returns_none() {
        assert!(parse_length_prefixed_packet(&[]).is_none());
    }

    /// 1-byte buffer → None (header truncated).
    #[test]
    fn parse_length_prefixed_one_byte_returns_none() {
        assert!(parse_length_prefixed_packet(&[0xFF]).is_none());
    }

    /// Length declared but body truncated → None (C++ would block on
    /// the async read; Rust caller drops the frame).
    #[test]
    fn parse_length_prefixed_truncated_body_returns_none() {
        // length = 5 but only 3 body bytes provided.
        let buf = [0x05, 0x00, 0xAA, 0xBB, 0xCC];
        assert!(parse_length_prefixed_packet(&buf).is_none());
    }

    /// Trailing extra bytes (length < remaining) → body is exactly the
    /// declared length, matching the C++ "discard trailing" behaviour.
    #[test]
    fn parse_length_prefixed_extra_trailing_bytes_clipped_to_declared_length() {
        // length = 2 but 4 body bytes present; only the first 2 are
        // returned.
        let buf = [0x02, 0x00, 0xAA, 0xBB, 0xCC, 0xDD];
        assert_eq!(parse_length_prefixed_packet(&buf), Some(&[0xAA, 0xBB][..]));
    }

    /// Zero-length packet → empty body slice (valid header, no body).
    #[test]
    fn parse_length_prefixed_zero_length_returns_empty_slice() {
        let buf = [0x00, 0x00];
        assert_eq!(parse_length_prefixed_packet(&buf), Some(&[][..]));
    }

    // ── peer_ip_to_u32 (Session 34) ─────────────────────────────────────

    #[test]
    fn peer_ip_v4_returns_be_u32() {
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)), 7172);
        // 192.168.0.1 = 0xC0A80001 (big-endian byte order).
        assert_eq!(peer_ip_to_u32(addr), 0xC0A80001);
    }

    #[test]
    fn peer_ip_v4_localhost_returns_127_0_0_1() {
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7172);
        assert_eq!(peer_ip_to_u32(addr), 0x7F000001);
    }

    #[test]
    fn peer_ip_v6_returns_zero() {
        use std::net::{IpAddr, Ipv6Addr, SocketAddr};
        let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 7172);
        assert_eq!(peer_ip_to_u32(addr), 0);
    }
}
