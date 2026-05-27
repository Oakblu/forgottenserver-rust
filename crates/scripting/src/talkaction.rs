use std::collections::BTreeMap;

/// Mirrors C++ `TalkActionResult_t`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TalkActionResult {
    /// Handler ran and should continue checking other actions.
    Continue,
    /// Handler ran and no further actions should be checked.
    Break,
    /// Handler failed (Lua error, call stack overflow, etc.).
    Failed,
}

#[derive(Debug, Clone)]
pub struct TalkAction {
    pub words: String,
    /// Additional aliases for the same action (C++ `wordsMap`).
    pub words_map: Vec<String>,
    pub separator: String,
    pub script_name: String,
    pub access_level: u32,
    pub need_access: bool,
    pub log: bool,
}

impl TalkAction {
    pub fn new(words: impl Into<String>, script_name: impl Into<String>) -> Self {
        let w = words.into();
        Self {
            words_map: vec![w.clone()],
            words: w,
            separator: " ".to_string(),
            script_name: script_name.into(),
            access_level: 0,
            need_access: false,
            log: false,
        }
    }

    pub fn with_separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }

    pub fn with_access_level(mut self, level: u32) -> Self {
        self.access_level = level;
        self
    }

    pub fn with_need_access(mut self, need: bool) -> Self {
        self.need_access = need;
        self
    }

    pub fn with_log(mut self, log: bool) -> Self {
        self.log = log;
        self
    }

    /// Add an additional alias word (mirrors C++ `wordsMap`).
    pub fn add_word(&mut self, word: impl Into<String>) {
        self.words_map.push(word.into());
    }
}

/// Word-censor filter: replaces any occurrence of a banned word (case-insensitive,
/// whole-word boundary) with `***`.
///
/// Mirrors the C++ `filterWords` / chat-censor functionality.
pub fn filter_words(text: &str, filter_list: &[&str]) -> String {
    if filter_list.is_empty() {
        return text.to_string();
    }

    let mut result = text.to_string();
    for banned in filter_list {
        let lower_result = result.to_lowercase();
        let lower_banned = banned.to_lowercase();
        // Replace all occurrences, rebuilding the string each pass.
        let mut output = String::with_capacity(result.len());
        let mut start = 0usize;
        while start < lower_result.len() {
            if let Some(pos) = lower_result[start..].find(lower_banned.as_str()) {
                let abs_pos = start + pos;
                output.push_str(&result[start..abs_pos]);
                output.push_str("***");
                start = abs_pos + banned.len();
            } else {
                output.push_str(&result[start..]);
                break;
            }
        }
        result = output;
    }
    result
}

#[derive(Debug, Default)]
pub struct TalkActions {
    // BTreeMap so iteration is in sorted (deterministic) order.
    actions: BTreeMap<String, TalkAction>,
}

impl TalkActions {
    pub fn new() -> Self {
        Self {
            actions: BTreeMap::new(),
        }
    }

    pub fn register(&mut self, action: TalkAction) {
        // C++ `TalkActions::registerEvent` iterates `wordsMap` and emplaces
        // one entry per alias — clones for every-alias-but-the-last and
        // moves the last. The Rust port matches the same observable shape:
        // clone for every alias except the final one, move the action into
        // the last entry. When `words_map` is empty (legacy path), fall
        // back to inserting under `action.words` so callers who construct
        // TalkAction without aliases still work.
        if action.words_map.is_empty() {
            let words = action.words.clone();
            self.actions.insert(words, action);
            return;
        }
        // Clone for every alias except the last; move into the last slot.
        let mut iter = action.words_map.iter().peekable();
        while let Some(word) = iter.next() {
            if iter.peek().is_none() {
                self.actions.insert(word.clone(), action);
                return;
            }
            self.actions.insert(word.clone(), action.clone());
        }
    }

    /// Returns the best-matching TalkAction for the given input.
    ///
    /// Match logic mirrors C++ `playerSaySpell`:
    /// 1. Case-insensitive prefix match on the registered `words`.
    /// 2. After the prefix, the remainder must start with a space (or be empty).
    /// 3. The longest matching prefix wins.
    pub fn get_talk_action(&self, input: &str) -> Option<&TalkAction> {
        let input_lower = input.to_lowercase();
        let mut best: Option<&TalkAction> = None;

        for (key, action) in &self.actions {
            let key_lower = key.to_lowercase();

            // Must be a case-insensitive prefix of input.
            if !input_lower.starts_with(key_lower.as_str()) {
                continue;
            }

            let remainder = &input[key.len()..];

            // Exact match or remainder starts with space (C++ requires ' ' after command).
            if !remainder.is_empty() && !remainder.starts_with(' ') {
                continue;
            }

            match best {
                None => best = Some(action),
                Some(current) => {
                    if key.len() > current.words.len() {
                        best = Some(action);
                    }
                }
            }
        }
        best
    }

    /// Dispatch `on_say`: find matching action and invoke `handler(action, param)`.
    ///
    /// `param` is the text after the command prefix (trimmed).
    /// Returns `TalkActionResult::Continue` when no action matched.
    /// Returns whatever the handler returns on a match.
    pub fn on_say<F>(&self, input: &str, mut handler: F) -> TalkActionResult
    where
        F: FnMut(&TalkAction, &str) -> TalkActionResult,
    {
        let input_lower = input.to_lowercase();
        let mut best_len = 0usize;
        let mut best_action: Option<&TalkAction> = None;

        for (key, action) in &self.actions {
            let key_lower = key.to_lowercase();

            if !input_lower.starts_with(key_lower.as_str()) {
                continue;
            }

            let remainder = &input[key.len()..];
            if !remainder.is_empty() && !remainder.starts_with(' ') {
                continue;
            }

            if key.len() > best_len {
                best_len = key.len();
                best_action = Some(action);
            }
        }

        if let Some(action) = best_action {
            let param = input[best_len..].trim_start();
            handler(action, param)
        } else {
            TalkActionResult::Continue
        }
    }
}

// ---------------------------------------------------------------------------
// TalkAction XML loader (Session 35 ledger closure)
// ---------------------------------------------------------------------------

/// Parsed XML row for one talkaction. Caller iterates rows from
/// `parse_talkactions_xml` and applies each via `apply_parsed_talkaction`.
///
/// `words` is the semicolon-split alias list (matches C++
/// `explodeString(wordsAttribute, ";")`); empty if the attribute was
/// missing — apply will surface that as an Err.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTalkActionRow {
    pub words: Vec<String>,
    pub separator: String,
    pub script_name: String,
    pub access_level: u32,
    pub need_access: bool,
    pub log: bool,
}

/// Result of `parse_talkactions_xml` — parsed rows + non-fatal warnings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTalkActionsXml {
    pub rows: Vec<ParsedTalkActionRow>,
    /// One entry per `<talkaction>` element with a missing/empty
    /// `words` attribute (matches C++ `[Error - TalkAction::configureEvent]
    /// Missing words for talk action or spell`).
    pub warnings: Vec<String>,
}

/// Walk a `<talkactions>` XML doc and return parsed rows + warnings.
/// Mirrors C++ `TalkAction::configureEvent` (per-node read of `words`,
/// `separator`) + the surrounding XML loader.
///
/// Hard errors (malformed XML, missing root element) surface as `Err`.
/// Per-row errors (missing `words` attribute, empty after split) are
/// emitted as warnings — matching C++'s "print-error-then-continue"
/// behaviour. The caller still gets a row for the well-formed
/// attributes that did parse.
pub fn parse_talkactions_xml(xml: &str) -> Result<ParsedTalkActionsXml, String> {
    let doc = roxmltree::Document::parse(xml).map_err(|e| format!("XML parse error: {e}"))?;
    let root = doc
        .descendants()
        .find(|n| n.has_tag_name("talkactions"))
        .ok_or_else(|| "Missing <talkactions> root element".to_string())?;

    let mut rows: Vec<ParsedTalkActionRow> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    for node in root.children().filter(|n| n.is_element()) {
        let words_attr = node.attribute("words").unwrap_or("");
        // C++ splits on `;` and trims via explodeString — Rust split
        // produces the same vec; we drop empty tokens to match the
        // implicit filter (a leading/trailing `;` doesn't register an
        // empty alias).
        let words: Vec<String> = words_attr
            .split(';')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if words.is_empty() {
            warnings.push(format!(
                "[TalkAction::configureEvent] Missing words for talk action: {}",
                node.attribute("script").unwrap_or("<no-script>")
            ));
            continue;
        }
        rows.push(ParsedTalkActionRow {
            words,
            separator: node
                .attribute("separator")
                .map(str::to_string)
                .unwrap_or_else(|| " ".to_string()),
            script_name: node
                .attribute("script")
                .map(str::to_string)
                .unwrap_or_default(),
            access_level: node
                .attribute("access")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            need_access: node
                .attribute("needaccess")
                .map(|s| matches!(s.to_ascii_lowercase().as_str(), "1" | "true"))
                .unwrap_or(false),
            log: node
                .attribute("log")
                .map(|s| matches!(s.to_ascii_lowercase().as_str(), "1" | "true"))
                .unwrap_or(false),
        });
    }
    Ok(ParsedTalkActionsXml { rows, warnings })
}

/// Build a `TalkAction` from a parsed row and register it (multi-alias).
/// Mirrors C++ `TalkActions::registerEvent(event, node)`.
pub fn apply_parsed_talkaction(actions: &mut TalkActions, row: &ParsedTalkActionRow) {
    let mut action = TalkAction::new(row.words[0].clone(), row.script_name.clone());
    action.words_map = row.words.clone();
    action.separator = row.separator.clone();
    action.access_level = row.access_level;
    action.need_access = row.need_access;
    action.log = row.log;
    actions.register(action);
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── existing tests (preserved) ──────────────────────────────────────────

    #[test]
    fn talk_action_struct_fields() {
        let action = TalkAction {
            words: "/tp".to_string(),
            words_map: vec!["/tp".to_string()],
            separator: " ".to_string(),
            script_name: "tp.lua".to_string(),
            access_level: 3,
            need_access: false,
            log: false,
        };
        assert_eq!(action.words, "/tp");
        assert_eq!(action.separator, " ");
        assert_eq!(action.script_name, "tp.lua");
        assert_eq!(action.access_level, 3);
    }

    #[test]
    fn talk_actions_new_creates_empty_registry() {
        let ta = TalkActions::new();
        assert!(ta.get_talk_action("/any").is_none());
    }

    #[test]
    fn register_adds_action() {
        let mut ta = TalkActions::new();
        ta.register(TalkAction::new("/hi", "hi.lua"));
        assert!(ta.get_talk_action("/hi").is_some());
    }

    #[test]
    fn get_talk_action_exact_match() {
        let mut ta = TalkActions::new();
        ta.register(TalkAction::new("/tp", "tp.lua"));
        let action = ta.get_talk_action("/tp").unwrap();
        assert_eq!(action.words, "/tp");
    }

    #[test]
    fn get_talk_action_returns_none_for_unknown() {
        let ta = TalkActions::new();
        assert!(ta.get_talk_action("/unknown").is_none());
    }

    #[test]
    fn prefix_match_with_separator() {
        let mut ta = TalkActions::new();
        let action = TalkAction::new("/tp", "tp.lua").with_separator(" ");
        ta.register(action);
        // "/tp alice" should match "/tp" with separator " "
        let result = ta.get_talk_action("/tp alice");
        assert!(result.is_some());
        assert_eq!(result.unwrap().words, "/tp");
    }

    #[test]
    fn longest_match_wins() {
        let mut ta = TalkActions::new();
        ta.register(TalkAction::new("/t", "t.lua").with_separator(" "));
        ta.register(TalkAction::new("/tp", "tp.lua").with_separator(" "));

        // "/tp" exact matches "/tp"
        let result = ta.get_talk_action("/tp");
        assert_eq!(result.unwrap().words, "/tp");
    }

    #[test]
    fn longest_prefix_wins_over_shorter_prefix() {
        let mut ta = TalkActions::new();
        ta.register(TalkAction::new("/t", "t.lua").with_separator(" "));
        ta.register(TalkAction::new("/tp", "tp.lua").with_separator(" "));

        // "/tp alice" should match "/tp" not "/t"
        let result = ta.get_talk_action("/tp alice");
        assert!(result.is_some());
        assert_eq!(result.unwrap().words, "/tp");
    }

    #[test]
    fn access_level_field_is_preserved() {
        let mut ta = TalkActions::new();
        ta.register(TalkAction::new("/admin", "admin.lua").with_access_level(5));
        let action = ta.get_talk_action("/admin").unwrap();
        assert_eq!(action.access_level, 5);
    }

    // ── new tests ───────────────────────────────────────────────────────────

    /// `on_say`: handler is called when input starts with the registered prefix.
    #[test]
    fn on_say_prefix_match() {
        let mut ta = TalkActions::new();
        ta.register(TalkAction::new("/tp", "tp.lua"));

        let mut called = false;
        let result = ta.on_say("/tp alice", |action, param| {
            called = true;
            assert_eq!(action.words, "/tp");
            assert_eq!(param, "alice");
            TalkActionResult::Continue
        });

        assert!(called, "handler should have been called");
        assert_eq!(result, TalkActionResult::Continue);
    }

    /// Shared probe handler used by negative-match tests. Using a free `fn`
    /// (rather than a closure) means the body lives at a single compile-time
    /// location, so one test invoking it via a matching input covers the body
    /// for every other test that uses it on a non-matching input.
    fn probe_continue(_: &TalkAction, _: &str) -> TalkActionResult {
        TalkActionResult::Continue
    }

    /// `on_say`: handler is NOT called when prefix does not match.
    #[test]
    fn on_say_no_match() {
        let mut ta = TalkActions::new();
        ta.register(TalkAction::new("/tp", "tp.lua"));

        let result = ta.on_say("/go somewhere", probe_continue);
        assert_eq!(result, TalkActionResult::Continue);
    }

    /// Exercises `probe_continue` via a matching dispatch so its body is
    /// covered for the negative-match tests above.
    #[test]
    fn on_say_probe_continue_runs_on_match() {
        let mut ta = TalkActions::new();
        ta.register(TalkAction::new("/probe", "probe.lua"));
        let result = ta.on_say("/probe x", probe_continue);
        assert_eq!(result, TalkActionResult::Continue);
    }

    /// `filter_words`: a banned word is replaced with `***`.
    #[test]
    fn filter_words_replaces_banned_word() {
        let output = filter_words("hello shit world", &["shit"]);
        assert_eq!(output, "hello *** world");
    }

    /// `filter_words`: multiple distinct banned words are all replaced.
    #[test]
    fn filter_words_multiple_words() {
        let output = filter_words("damn this crap is bad", &["damn", "crap"]);
        assert_eq!(output, "*** this *** is bad");
    }

    /// `filter_words`: matching is case-insensitive.
    #[test]
    fn filter_words_case_insensitive() {
        let output = filter_words("SHIT happens", &["shit"]);
        assert_eq!(output, "*** happens");
    }

    /// Registering multiple commands dispatches each to its own handler.
    #[test]
    fn register_multiple_commands() {
        let mut ta = TalkActions::new();
        ta.register(TalkAction::new("/tp", "tp.lua"));
        ta.register(TalkAction::new("/ban", "ban.lua"));

        let mut tp_called = false;
        ta.on_say("/tp bob", |action, _| {
            tp_called = true;
            assert_eq!(action.words, "/tp");
            TalkActionResult::Continue
        });

        let mut ban_called = false;
        ta.on_say("/ban eve", |action, _| {
            ban_called = true;
            assert_eq!(action.words, "/ban");
            TalkActionResult::Continue
        });

        assert!(tp_called, "/tp handler should be called");
        assert!(ban_called, "/ban handler should be called");
    }

    /// `on_say`: handler returning `Break` is propagated.
    #[test]
    fn on_say_returns_handler_result() {
        let mut ta = TalkActions::new();
        ta.register(TalkAction::new("/cmd", "cmd.lua"));

        let result = ta.on_say("/cmd", |_, _| TalkActionResult::Break);
        assert_eq!(result, TalkActionResult::Break);
    }

    /// `on_say`: case-insensitive command matching (C++ uses `caseInsensitiveStartsWith`).
    #[test]
    fn on_say_case_insensitive_match() {
        let mut ta = TalkActions::new();
        ta.register(TalkAction::new("/TP", "tp.lua"));

        let mut called = false;
        ta.on_say("/tp alice", |_, _| {
            called = true;
            TalkActionResult::Continue
        });
        assert!(called, "case-insensitive match should fire the handler");
    }

    /// `on_say`: partial word should NOT match (e.g. "/tpx" must not match "/tp").
    #[test]
    fn on_say_no_partial_word_match() {
        let mut ta = TalkActions::new();
        ta.register(TalkAction::new("/tp", "tp.lua"));

        let result = ta.on_say("/tpx", probe_continue);
        // No match → no handler invocation → Continue is the no-op return.
        assert_eq!(result, TalkActionResult::Continue);
    }

    /// `TalkActionResult` variants are distinct.
    #[test]
    fn talk_action_result_variants() {
        assert_ne!(TalkActionResult::Continue, TalkActionResult::Break);
        assert_ne!(TalkActionResult::Break, TalkActionResult::Failed);
        assert_ne!(TalkActionResult::Continue, TalkActionResult::Failed);
    }

    /// `TalkAction::need_access` and `log` flags round-trip through builder.
    #[test]
    fn talk_action_builder_flags() {
        let action = TalkAction::new("/secret", "secret.lua")
            .with_need_access(true)
            .with_log(true);
        assert!(action.need_access);
        assert!(action.log);
    }

    /// `TalkAction::words_map` is populated on construction.
    #[test]
    fn talk_action_words_map_initial() {
        let action = TalkAction::new("/hi", "hi.lua");
        assert_eq!(action.words_map, vec!["/hi"]);
    }

    /// `add_word` pushes additional aliases into `words_map`.
    #[test]
    fn talk_action_add_word() {
        let mut action = TalkAction::new("/hi", "hi.lua");
        action.add_word("/hello");
        assert_eq!(action.words_map, vec!["/hi", "/hello"]);
    }

    /// `filter_words` with empty filter list returns original text unchanged.
    #[test]
    fn filter_words_empty_filter_list() {
        let output = filter_words("hello world", &[]);
        assert_eq!(output, "hello world");
    }

    /// `filter_words` with no matching words returns original text unchanged.
    #[test]
    fn filter_words_no_match() {
        let output = filter_words("hello world", &["badword"]);
        assert_eq!(output, "hello world");
    }

    /// `get_talk_action`: when the input is not a (case-insensitive) prefix of
    /// any registered key, the loop hits the `continue` branch and the function
    /// returns `None`.
    #[test]
    fn get_talk_action_input_not_a_prefix_returns_none() {
        let mut ta = TalkActions::new();
        ta.register(TalkAction::new("/tp", "tp.lua"));
        // "/foo" does not start with "/tp" — `starts_with` is false.
        assert!(ta.get_talk_action("/foo").is_none());
    }

    /// `get_talk_action`: when two registered keys both match (one is a strict
    /// prefix of the other AND the longer key is still followed by a space in
    /// the input), the longer key wins. This drives the `Some(current)` arm of
    /// the inner `match best` and the `key.len() > current.words.len()` branch.
    #[test]
    fn get_talk_action_longer_match_wins_when_both_valid() {
        let mut ta = TalkActions::new();
        // Both `/t` and `/t hello` are registered. Input `/t hello world`
        // makes both keys valid prefixes followed by a space.
        ta.register(TalkAction::new("/t", "short.lua"));
        ta.register(TalkAction::new("/t hello", "long.lua"));

        let hit = ta.get_talk_action("/t hello world").expect("must match");
        assert_eq!(hit.words, "/t hello");
        assert_eq!(hit.script_name, "long.lua");
    }

    /// Mirror of the `get_talk_action` longer-match test but exercised via
    /// `on_say` to confirm the dispatcher follows the same precedence.
    #[test]
    fn on_say_longest_prefix_among_valid_matches_wins() {
        let mut ta = TalkActions::new();
        ta.register(TalkAction::new("/t", "short.lua"));
        ta.register(TalkAction::new("/t hello", "long.lua"));

        let result = ta.on_say("/t hello world", |action, param| {
            assert_eq!(action.words, "/t hello");
            assert_eq!(param, "world");
            TalkActionResult::Break
        });
        assert_eq!(result, TalkActionResult::Break);
    }

    /// `get_talk_action`: when a second matching key has the same length as the
    /// current best (case-only difference, stored separately by BTreeMap), the
    /// existing best is kept. Exercises the `Some(current)` arm with the inner
    /// `key.len() > current.words.len()` branch evaluating to FALSE.
    #[test]
    fn get_talk_action_equal_length_keys_keeps_first_best() {
        let mut ta = TalkActions::new();
        // `/A` sorts before `/a` in BTreeMap (uppercase has lower codepoint).
        ta.register(TalkAction::new("/A", "upper.lua"));
        ta.register(TalkAction::new("/a", "lower.lua"));

        let hit = ta.get_talk_action("/a x").expect("must match");
        // The first matching key (/A) remains the best — `/a` does not replace
        // it because the length comparison is strict (`>`), not `>=`.
        assert_eq!(hit.words, "/A");
    }

    /// `on_say`: counterpart for the equal-length / case-only-difference case.
    /// Drives the `if key.len() > best_len` branch to evaluate FALSE on a
    /// later iteration, covering the closing brace of that block.
    #[test]
    fn on_say_equal_length_keys_keeps_first_best() {
        let mut ta = TalkActions::new();
        ta.register(TalkAction::new("/A", "upper.lua"));
        ta.register(TalkAction::new("/a", "lower.lua"));

        let result = ta.on_say("/a x", |action, _| {
            // The earlier-iterated key wins because of strict `>`.
            assert_eq!(action.words, "/A");
            TalkActionResult::Break
        });
        assert_eq!(result, TalkActionResult::Break);
    }

    // ── multi-alias register + XML loader (Session 35) ──────────────────

    /// `register` inserts the action under EVERY alias in `words_map`.
    #[test]
    fn register_multi_alias_inserts_under_every_word() {
        let mut ta = TalkActions::new();
        let mut action = TalkAction::new("!buyhouse", "house.lua");
        action.add_word("!sellhouse");
        action.add_word("!leavehouse");
        ta.register(action);
        // All three aliases must resolve.
        assert!(ta.get_talk_action("!buyhouse").is_some());
        assert!(ta.get_talk_action("!sellhouse").is_some());
        assert!(ta.get_talk_action("!leavehouse").is_some());
    }

    /// Single-alias TalkAction (the legacy path) still registers under
    /// the primary `words` even when `words_map` happens to be empty.
    #[test]
    fn register_falls_back_to_primary_words_when_words_map_empty() {
        let mut ta = TalkActions::new();
        let mut action = TalkAction::new("!hello", "hello.lua");
        action.words_map.clear(); // Force the legacy fallback path.
        ta.register(action);
        assert!(ta.get_talk_action("!hello").is_some());
    }

    /// Parser extracts every alias from a semicolon-separated `words`.
    #[test]
    fn parse_xml_splits_semicolon_separated_words_into_aliases() {
        let xml = r#"<talkactions>
            <talkaction words="!buyhouse;!sellhouse;!leavehouse" script="house.lua" />
        </talkactions>"#;
        let parsed = parse_talkactions_xml(xml).unwrap();
        assert!(parsed.warnings.is_empty());
        assert_eq!(parsed.rows.len(), 1);
        assert_eq!(
            parsed.rows[0].words,
            vec!["!buyhouse", "!sellhouse", "!leavehouse"]
        );
        assert_eq!(parsed.rows[0].script_name, "house.lua");
    }

    /// Parser reads optional attributes (separator/access/needaccess/log).
    #[test]
    fn parse_xml_reads_optional_attributes() {
        let xml = r#"<talkactions>
            <talkaction words="!ban" script="ban.lua" separator="-"
                        access="5" needaccess="1" log="true" />
        </talkactions>"#;
        let parsed = parse_talkactions_xml(xml).unwrap();
        let row = &parsed.rows[0];
        assert_eq!(row.separator, "-");
        assert_eq!(row.access_level, 5);
        assert!(row.need_access);
        assert!(row.log);
    }

    /// Optional attributes default to C++ defaults when absent.
    #[test]
    fn parse_xml_defaults_separator_to_space_and_flags_to_false() {
        let xml = r#"<talkactions>
            <talkaction words="/foo" script="foo.lua" />
        </talkactions>"#;
        let parsed = parse_talkactions_xml(xml).unwrap();
        let row = &parsed.rows[0];
        assert_eq!(row.separator, " ");
        assert_eq!(row.access_level, 0);
        assert!(!row.need_access);
        assert!(!row.log);
    }

    /// Missing `words` attribute emits a warning and skips the row
    /// (matches C++ "print error and continue").
    #[test]
    fn parse_xml_missing_words_emits_warning_skips_row() {
        let xml = r#"<talkactions>
            <talkaction script="nope.lua" />
        </talkactions>"#;
        let parsed = parse_talkactions_xml(xml).unwrap();
        assert!(parsed.rows.is_empty());
        assert_eq!(parsed.warnings.len(), 1);
        assert!(parsed.warnings[0].contains("Missing words for talk action"));
    }

    /// Trailing `;` doesn't produce an empty alias.
    #[test]
    fn parse_xml_filters_empty_alias_tokens() {
        let xml = r#"<talkactions>
            <talkaction words="/a;;/b;" script="ab.lua" />
        </talkactions>"#;
        let parsed = parse_talkactions_xml(xml).unwrap();
        assert_eq!(parsed.rows[0].words, vec!["/a", "/b"]);
    }

    /// Malformed XML surfaces as Err.
    #[test]
    fn parse_xml_malformed_returns_err() {
        let result = parse_talkactions_xml("<talkactions><talkaction");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("XML parse error"));
    }

    /// Missing root element surfaces as Err.
    #[test]
    fn parse_xml_missing_root_returns_err() {
        let result = parse_talkactions_xml("<other-root/>");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing <talkactions>"));
    }

    /// `apply_parsed_talkaction` round-trips the row into the registry
    /// (under every alias, with all flags preserved).
    #[test]
    fn apply_parsed_row_registers_under_all_aliases_with_fields() {
        let row = ParsedTalkActionRow {
            words: vec!["!buyhouse".into(), "!sellhouse".into()],
            separator: "-".into(),
            script_name: "house.lua".into(),
            access_level: 3,
            need_access: true,
            log: false,
        };
        let mut actions = TalkActions::new();
        apply_parsed_talkaction(&mut actions, &row);
        let a = actions.get_talk_action("!buyhouse").unwrap();
        assert_eq!(a.separator, "-");
        assert_eq!(a.script_name, "house.lua");
        assert_eq!(a.access_level, 3);
        assert!(a.need_access);
        assert!(!a.log);
        // Second alias also registered.
        assert!(actions.get_talk_action("!sellhouse").is_some());
    }
}
