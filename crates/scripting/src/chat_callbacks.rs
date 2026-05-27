//! Scripting-side adapter that wires the `game::chat::ChatLuaCallbacks` trait
//! into the `ScriptEnvironment` UID table + Lua dispatcher.
//!
//! Session 10 (forgottenserver-rust-protocolgame-wire-parity Piece C.2.1)
//! ships this as a minimal stub: each trait method returns the deterministic
//! default the C++ implementation falls back to when no Lua handler is
//! registered. Hook-name → callback-id resolution + real `mlua` invocation
//! is deferred to a follow-up session because it requires reading
//! `ScriptEnvironment`'s callback-id storage, which is currently coupled to
//! the per-call `current_script_id`/`current_callback_id` model rather than
//! a per-channel handler registry.
//!
//! The trait surface this implements lives in `forgottenserver_game::chat`;
//! the scripting crate already depends on `forgottenserver-game` so no new
//! cross-crate edge is introduced. Once a per-channel handler registry is
//! added to `ScriptEnvironment`, this file's stub bodies are the only thing
//! that needs to change — the game-side chat plumbing already routes
//! through this trait.

use forgottenserver_game::chat::{ChannelId, ChatLuaCallbacks, EntityId};

/// Stub adapter. Holds nothing; future revisions will hold an
/// `Arc<Mutex<ScriptEnvironment>>` (or equivalent) so Lua-side handlers
/// can be looked up by channel id.
#[derive(Debug, Default)]
pub struct ScriptingChatCallbacks;

impl ScriptingChatCallbacks {
    /// Construct a fresh stub adapter.
    pub fn new() -> Self {
        Self
    }
}

impl ChatLuaCallbacks for ScriptingChatCallbacks {
    fn can_join(&self, _channel_id: ChannelId, _player_id: EntityId) -> bool {
        // C++ default: allow join when no handler is registered.
        true
    }

    fn on_join(&self, _channel_id: ChannelId, _player_id: EntityId) -> bool {
        // C++ default: signal success when no handler is registered.
        true
    }

    fn on_leave(&self, _channel_id: ChannelId, _player_id: EntityId) {
        // C++ default: no-op when no handler is registered.
    }

    fn on_speak(
        &self,
        _channel_id: ChannelId,
        _player_id: EntityId,
        _speak_class: u8,
        _message: &str,
    ) -> Option<bool> {
        // C++ default: no handler registered → fall through to default
        // delivery path. Caller (chat manager) handles `None` correctly.
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_join_defaults_to_true() {
        let cb = ScriptingChatCallbacks::new();
        assert!(cb.can_join(7, 12345));
    }

    #[test]
    fn on_join_defaults_to_true() {
        let cb = ScriptingChatCallbacks::new();
        assert!(cb.on_join(7, 12345));
    }

    #[test]
    fn on_leave_is_noop() {
        let cb = ScriptingChatCallbacks::new();
        cb.on_leave(7, 12345);
    }

    #[test]
    fn on_speak_returns_none_when_no_handler() {
        let cb = ScriptingChatCallbacks::new();
        assert_eq!(cb.on_speak(7, 12345, 0x01, "hello"), None);
    }

    #[test]
    fn default_impl_constructs_equivalently_to_new() {
        let a: ScriptingChatCallbacks = ScriptingChatCallbacks;
        let b = ScriptingChatCallbacks::new();
        // Both must produce identical default behaviour.
        assert_eq!(a.can_join(1, 1), b.can_join(1, 1));
        assert_eq!(a.on_join(1, 1), b.on_join(1, 1));
        assert_eq!(a.on_speak(1, 1, 0, "x"), b.on_speak(1, 1, 0, "x"));
    }
}
