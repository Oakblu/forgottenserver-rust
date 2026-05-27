//! Migrated from forgottenserver/src/wildcardtree.h + wildcardtree.cpp
//!
//! A trie (prefix tree) used for player-name lookup with optional wildcard (`*`)
//! support.  A trailing `*` in the search query is treated as "match any
//! completion of this prefix".
//!
//! Names are stored and searched in a case-insensitive manner (all lowercase).

use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Error returned by [`WildcardTree::search`] when a wildcard query matches
/// more than one name and `accept_multiples` is `false`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WildcardError {
    /// The wildcard query matched more than one stored name.
    TooManyResults,
}

impl std::fmt::Display for WildcardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WildcardError::TooManyResults => write!(f, "too many results (ambiguous wildcard)"),
        }
    }
}

// ---------------------------------------------------------------------------
// Internal trie node
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct TrieNode {
    /// `true` when this node marks the end of an inserted word.
    breakpoint: bool,
    children: BTreeMap<char, TrieNode>,
}

impl TrieNode {
    fn new(breakpoint: bool) -> Self {
        Self {
            breakpoint,
            children: BTreeMap::new(),
        }
    }

    /// Add or reach the child for `ch`, setting the breakpoint if requested.
    fn add_child(&mut self, ch: char, breakpoint: bool) -> &mut TrieNode {
        let child = self
            .children
            .entry(ch)
            .or_insert_with(|| TrieNode::new(false));
        if breakpoint {
            child.breakpoint = true;
        }
        child
    }

    /// Insert `str` (as a lowercase iterator) into the trie rooted here.
    fn insert(&mut self, s: &str) {
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len();
        if len == 0 {
            self.breakpoint = true;
            return;
        }

        let mut cur = self;
        for (i, &ch) in chars.iter().enumerate() {
            let is_last = i + 1 == len;
            cur = cur.add_child(ch, is_last);
        }
    }

    /// Remove `s` from the trie. Returns `true` if the word was present.
    fn remove(&mut self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        Self::remove_recursive(self, &chars, 0)
    }

    fn remove_recursive(node: &mut TrieNode, chars: &[char], depth: usize) -> bool {
        if depth == chars.len() {
            if !node.breakpoint {
                return false; // not present
            }
            node.breakpoint = false;
            return true;
        }

        let ch = chars[depth];
        let child_exists = node.children.contains_key(&ch);
        if !child_exists {
            return false;
        }

        let removed = {
            let child = node.children.get_mut(&ch).unwrap();
            Self::remove_recursive(child, chars, depth + 1)
        };

        if removed {
            // If the child is now a dead-end leaf with no breakpoint, prune it.
            let child = node.children.get(&ch).unwrap();
            if !child.breakpoint && child.children.is_empty() {
                node.children.remove(&ch);
            }
        }

        removed
    }

    /// Walk down the trie following each character of `query`.
    /// Returns the node at the end of the path, or `None` if any character is missing.
    fn walk(&self, query: &str) -> Option<&TrieNode> {
        let mut cur = self;
        for ch in query.chars() {
            cur = cur.children.get(&ch)?;
        }
        Some(cur)
    }

    /// Try to find exactly one completion from `node`, appending characters to `result`.
    /// Returns `Ok(true)` on unambiguous match, `Err(TooManyResults)` on ambiguity.
    fn find_one_completion(
        node: &TrieNode,
        result: &mut String,
        accept_multiples: bool,
    ) -> Result<(), WildcardError> {
        let mut cur = node;
        loop {
            let num_children = cur.children.len();
            if num_children == 0 {
                // Leaf — must be a breakpoint for a valid word.
                return Ok(());
            }

            if num_children > 1 || cur.breakpoint {
                // Ambiguous: multiple completions exist.
                if !accept_multiples {
                    return Err(WildcardError::TooManyResults);
                }
                // With accept_multiples we return the first (lexicographically smallest) match.
                // Walk to the breakpoint of the first branch.
                let (ch, child) = cur.children.iter().next().unwrap();
                result.push(*ch);
                cur = child;
                continue;
            }

            // Exactly one child, not a breakpoint — keep descending.
            let (ch, child) = cur.children.iter().next().unwrap();
            result.push(*ch);
            cur = child;
        }
    }
}

// ---------------------------------------------------------------------------
// WildcardTree
// ---------------------------------------------------------------------------

/// A trie that supports wildcard (`*`) suffix searches.
pub struct WildcardTree {
    root: TrieNode,
    accept_multiples: bool,
}

impl WildcardTree {
    /// Create a new empty `WildcardTree`.
    ///
    /// If `accept_multiples` is `true`, a wildcard query that matches several
    /// names returns `Ok(Some(first_match))`.  If `false`, such a query
    /// returns `Err(WildcardError::TooManyResults)`.
    pub fn new(accept_multiples: bool) -> Self {
        Self {
            root: TrieNode::new(false),
            accept_multiples,
        }
    }

    /// Insert `name` (stored as lowercase) into the trie.
    pub fn insert(&mut self, name: &str) {
        let lower = name.to_lowercase();
        self.root.insert(&lower);
    }

    /// Remove `name` from the trie.
    ///
    /// Returns `false` when the name was not present.
    pub fn remove(&mut self, name: &str) -> bool {
        let lower = name.to_lowercase();
        self.root.remove(&lower)
    }

    /// Search for `query` in the trie.
    ///
    /// * If `query` ends with `*` it is a prefix (wildcard) search.
    /// * Otherwise it is an exact search.
    ///
    /// Returns:
    /// * `Ok(Some(name))` — exactly one match found
    /// * `Ok(None)` — no match
    /// * `Err(WildcardError::TooManyResults)` — multiple matches, `accept_multiples = false`
    pub fn search(&self, query: &str) -> Result<Option<String>, WildcardError> {
        let lower = query.to_lowercase();

        if lower.ends_with('*') {
            let prefix = &lower[..lower.len() - 1];
            return self.search_wildcard(prefix);
        }

        // Exact search
        if let Some(node) = self.root.walk(&lower) {
            if node.breakpoint {
                return Ok(Some(lower.to_owned()));
            }
        }
        Ok(None)
    }

    fn search_wildcard(&self, prefix: &str) -> Result<Option<String>, WildcardError> {
        let Some(node) = self.root.walk(prefix) else {
            return Ok(None);
        };

        // If the prefix itself is a stored word AND has no children, return it.
        // If it has children, we need to find one unique completion.
        if node.breakpoint && node.children.is_empty() {
            return Ok(Some(prefix.to_owned()));
        }

        let mut result = prefix.to_owned();
        TrieNode::find_one_completion(node, &mut result, self.accept_multiples)?;

        // After completion, verify the result ends at a breakpoint.
        // (find_one_completion always terminates at a leaf/breakpoint by construction)
        Ok(Some(result))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Test 1: Insert "hello" → search "hello" found
    // -----------------------------------------------------------------------
    #[test]
    fn test_exact_match_found() {
        let mut tree = WildcardTree::new(false);
        tree.insert("hello");
        let result = tree.search("hello").expect("no error");
        assert_eq!(result, Some("hello".into()));
    }

    // -----------------------------------------------------------------------
    // Test 2: Insert "hello" → search "hell" not found (exact, not a prefix)
    // -----------------------------------------------------------------------
    #[test]
    fn test_exact_match_prefix_not_found() {
        let mut tree = WildcardTree::new(false);
        tree.insert("hello");
        let result = tree.search("hell").expect("no error");
        assert_eq!(result, None);
    }

    // -----------------------------------------------------------------------
    // Test 3: wildcard with accept_multiples=false → Err when multiple matches
    // -----------------------------------------------------------------------
    #[test]
    fn test_wildcard_multiple_matches_error() {
        let mut tree = WildcardTree::new(false);
        tree.insert("hello");
        tree.insert("help");
        let result = tree.search("hel*");
        assert_eq!(result, Err(WildcardError::TooManyResults));
    }

    // -----------------------------------------------------------------------
    // Test 4: wildcard with accept_multiples=true → Ok(Some) when multiple matches
    // -----------------------------------------------------------------------
    #[test]
    fn test_wildcard_multiple_matches_accepted() {
        let mut tree = WildcardTree::new(true);
        tree.insert("hello");
        tree.insert("help");
        let result = tree.search("hel*").expect("no error");
        // Should return one of the matches (the first alphabetically)
        assert!(result.is_some());
        let name = result.unwrap();
        assert!(name == "hello" || name == "help");
    }

    // -----------------------------------------------------------------------
    // Test 5: Remove "hello" → search "hello" not found
    // -----------------------------------------------------------------------
    #[test]
    fn test_remove_then_not_found() {
        let mut tree = WildcardTree::new(false);
        tree.insert("hello");
        let removed = tree.remove("hello");
        assert!(removed);
        let result = tree.search("hello").expect("no error");
        assert_eq!(result, None);
    }

    // -----------------------------------------------------------------------
    // Test 6: remove returns false when name not present
    // -----------------------------------------------------------------------
    #[test]
    fn test_remove_absent_returns_false() {
        let mut tree = WildcardTree::new(false);
        assert!(!tree.remove("ghost"));
    }

    // -----------------------------------------------------------------------
    // Test 7: Empty tree → search returns Ok(None)
    // -----------------------------------------------------------------------
    #[test]
    fn test_empty_tree_search_returns_none() {
        let tree = WildcardTree::new(false);
        assert_eq!(tree.search("anything").expect("no error"), None);
        assert_eq!(tree.search("any*").expect("no error"), None);
    }

    // -----------------------------------------------------------------------
    // Test 8: Unique wildcard completion
    // -----------------------------------------------------------------------
    #[test]
    fn test_wildcard_unique_completion() {
        let mut tree = WildcardTree::new(false);
        tree.insert("hello");
        let result = tree.search("hel*").expect("no error");
        assert_eq!(result, Some("hello".into()));
    }

    // -----------------------------------------------------------------------
    // Test 9: Case insensitive storage
    // -----------------------------------------------------------------------
    #[test]
    fn test_case_insensitive_insert_search() {
        let mut tree = WildcardTree::new(false);
        tree.insert("Hello");
        // Stored as "hello", searching "hello" should find it
        let result = tree.search("hello").expect("no error");
        assert_eq!(result, Some("hello".into()));
    }

    // -----------------------------------------------------------------------
    // Test 10: Wildcard with no prefix matches prefix itself
    // -----------------------------------------------------------------------
    #[test]
    fn test_wildcard_star_only_returns_first() {
        let mut tree = WildcardTree::new(true);
        tree.insert("alpha");
        tree.insert("beta");
        // "*" matches everything; with accept_multiples=true it returns the first
        let result = tree.search("*").expect("no error");
        assert!(result.is_some());
    }

    // -----------------------------------------------------------------------
    // Test 11: Remove one of two names; the other still found via wildcard
    // -----------------------------------------------------------------------
    #[test]
    fn test_remove_one_of_two_wildcard_still_finds_other() {
        let mut tree = WildcardTree::new(false);
        tree.insert("hello");
        tree.insert("help");
        tree.remove("help");
        // Now only "hello" — wildcard should resolve uniquely
        let result = tree.search("hel*").expect("no error");
        assert_eq!(result, Some("hello".into()));
    }

    // -----------------------------------------------------------------------
    // Test 12: Exact wildcard star at the end of an exact stored word
    // -----------------------------------------------------------------------
    #[test]
    fn test_wildcard_exact_word_star() {
        let mut tree = WildcardTree::new(false);
        tree.insert("go");
        // "go*" — "go" is a breakpoint with no children, so it returns "go"
        let result = tree.search("go*").expect("no error");
        assert_eq!(result, Some("go".into()));
    }

    // -----------------------------------------------------------------------
    // Test 13: Wildcard suffix match — `*` on a unique suffix completes correctly
    // -----------------------------------------------------------------------
    #[test]
    fn test_wildcard_suffix_match_unique() {
        let mut tree = WildcardTree::new(false);
        tree.insert("abcdef");
        // "abc*" should uniquely complete to "abcdef"
        let result = tree.search("abc*").expect("no error");
        assert_eq!(result, Some("abcdef".into()));
    }

    // -----------------------------------------------------------------------
    // Test 14: multi-insert then remove — shared prefix nodes survive
    // -----------------------------------------------------------------------
    #[test]
    fn test_multi_insert_remove_shared_prefix() {
        let mut tree = WildcardTree::new(false);
        // Both share the prefix "ab"
        tree.insert("abc");
        tree.insert("abd");
        // Remove "abc"; "abd" must still be found
        let removed = tree.remove("abc");
        assert!(removed, "abc should have been present");
        assert_eq!(tree.search("abc").unwrap(), None, "abc should be gone");
        assert_eq!(
            tree.search("abd").unwrap(),
            Some("abd".into()),
            "abd must survive"
        );
        // Wildcard on the shared prefix should now uniquely resolve to "abd"
        let wc = tree.search("ab*").expect("no error");
        assert_eq!(wc, Some("abd".into()));
    }

    // -----------------------------------------------------------------------
    // Test 15: Remove a word that is a strict prefix of another word
    // -----------------------------------------------------------------------
    #[test]
    fn test_remove_prefix_word_leaves_longer_word() {
        let mut tree = WildcardTree::new(false);
        tree.insert("go");
        tree.insert("gone");
        // Remove the shorter word; the longer one must survive
        assert!(tree.remove("go"));
        assert_eq!(tree.search("go").unwrap(), None);
        assert_eq!(tree.search("gone").unwrap(), Some("gone".into()));
        // Wildcard from the shared root should now uniquely resolve to "gone"
        let wc = tree.search("go*").expect("no error");
        assert_eq!(wc, Some("gone".into()));
    }

    // -----------------------------------------------------------------------
    // Test 16: Empty tree wildcard returns None (no panic)
    // -----------------------------------------------------------------------
    #[test]
    fn test_empty_tree_wildcard_returns_none() {
        let tree = WildcardTree::new(false);
        assert_eq!(tree.search("x*").unwrap(), None);
    }

    // -----------------------------------------------------------------------
    // Test 17: Display impl for WildcardError::TooManyResults
    //
    // Covers lines 24-28 — the Display implementation. Other tests only
    // compare the error via PartialEq, never format it.
    // -----------------------------------------------------------------------
    #[test]
    fn test_wildcard_error_display() {
        let err = WildcardError::TooManyResults;
        let rendered = format!("{}", err);
        assert_eq!(rendered, "too many results (ambiguous wildcard)");
    }

    // -----------------------------------------------------------------------
    // Test 18: Insert empty string sets the root as a breakpoint
    //
    // Covers lines 61-62 — the `len == 0` early-return branch of
    // `TrieNode::insert`. Exact search for "" should then return Some("").
    // -----------------------------------------------------------------------
    #[test]
    fn test_insert_empty_string_marks_root_breakpoint() {
        let mut tree = WildcardTree::new(false);
        tree.insert("");
        // Exact search for "" must hit the root breakpoint.
        let result = tree.search("").expect("no error");
        assert_eq!(result, Some(String::new()));
    }

    // -----------------------------------------------------------------------
    // Test 19: Remove a path that exists but is not a breakpoint
    //
    // Covers line 81 (the `!node.breakpoint` branch of remove at full depth)
    // and line 104 (the `removed == false` path that skips pruning). We
    // insert "abcd" so the prefix nodes "a", "ab", "abc" exist without
    // breakpoints, then attempt to remove "abc": the descent succeeds but
    // the final node has breakpoint=false, so remove must return false and
    // the original word "abcd" must remain intact.
    // -----------------------------------------------------------------------
    #[test]
    fn test_remove_existing_path_without_breakpoint_returns_false() {
        let mut tree = WildcardTree::new(false);
        tree.insert("abcd");
        // "abc" is a path in the trie but not a stored word.
        let removed = tree.remove("abc");
        assert!(!removed, "removing a non-breakpoint path must return false");
        // "abcd" must still be searchable.
        assert_eq!(tree.search("abcd").unwrap(), Some("abcd".into()));
        // Wildcard from "abc*" must still uniquely complete to "abcd".
        assert_eq!(tree.search("abc*").unwrap(), Some("abcd".into()));
    }
}
