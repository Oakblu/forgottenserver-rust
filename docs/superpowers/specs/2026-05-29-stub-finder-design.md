# Stub Finder Script — Design Spec

**Date:** 2026-05-29  
**Status:** Approved  
**Output:** `scripts/find_stubs.py`

## Purpose

Identify stub functions in the Rust port codebase so they can be fed back into the migration ledger. A "stub" is any function that compiles but does not implement real behavior — it silently discards work, returns a trivially-wrong default, or panics on call.

## Scope

- **Input:** All `.rs` files under `crates/`
- **Excludes:** Test code — files under `tests/` directories and code inside `#[cfg(test)]` blocks
- **Cross-reference:** `rust_symbol_manifest.json` to correlate hits with ledger entries
- **Output:** `scripts/stub_report.json`

## Architecture

Single Python 3 script, stdlib only (no third-party deps).

1. Load `rust_symbol_manifest.json` into a lookup dict keyed by `(file_basename, fn_name)`.
2. Walk every `.rs` file under `crates/`. Skip files containing `/tests/` in their path.
3. For each file, strip out `#[cfg(test)]` blocks before applying detectors.
4. Apply four pattern detectors (see below).
5. For each hit, scan backwards up to 30 lines to find the enclosing `fn` signature.
6. Look up the function name in the manifest for the ledger correlation.
7. Write all results to `scripts/stub_report.json`.

## The Four Detectors

### 1. Trivial Body
A function whose entire `{ }` body contains only a single trivially-correct expression:

- `Ok(())`
- `Err(...)` (any error value)
- `Default::default()`
- `false`
- `true`
- `0`
- `None`
- `String::new()`
- `vec![]`

Detection: multi-line regex over the function block. The body must contain nothing else (no `let`, no `if`, no method calls).

### 2. Empty Body
A function whose `{}` body is empty (only whitespace). This catches `pub fn post_add_notification(&self) {}`.

Detection: single-line or multi-line match of `fn ... { }` or `fn ... {\n}`.

### 3. Dropped Work
A `drop(` call on a variable in a non-`Drop`-impl function body, where the dropped variable was accepted or received in the same function (socket, stream, connection, request, buf). Indicates an accept loop or handler that receives input and silently discards it.

Detection: find `drop(` inside a function body where the function is not `fn drop(&mut self)`.

### 4. Panic Stubs
`panic!(` or `unreachable!(` in non-test code, outside of a `Drop` implementation. These compile but crash at runtime when the code path is hit.

Detection: line-level match for `panic!(` or `unreachable!(`, filtered to exclude test blocks and `#[allow(...)]`-suppressed lines.

## Output Schema

```json
[
  {
    "file": "server/src/http.rs",
    "line": 218,
    "crate": "server",
    "fn_name": "accept_loop",
    "pattern": "dropped_work",
    "snippet": "drop(stream);",
    "ledger_symbol": null,
    "manifest_match": null
  }
]
```

Field | Type | Description
---|---|---
`file` | string | Path relative to `crates/`
`line` | int | 1-indexed line number of the stub pattern
`crate` | string | Crate name (first path component)
`fn_name` | string | Name of the enclosing function, or `"<unknown>"` if not found
`pattern` | string | One of: `trivial_body`, `empty_body`, `dropped_work`, `panic_stub`
`snippet` | string | The matched line or expression (truncated to 120 chars)
`ledger_symbol` | string or null | Qualified name from `rust_symbol_manifest.json` if matched
`manifest_match` | object or null | Full manifest entry if matched

## Script Location

`scripts/find_stubs.py` — runnable directly:

```bash
python3 scripts/find_stubs.py
# writes scripts/stub_report.json
```

## False Positive Handling

- Test code (files in `tests/` dirs, `#[cfg(test)]` blocks) is excluded.
- `impl Drop { fn drop(...) }` bodies are excluded from the dropped-work and panic detectors.
- The trivial-body detector requires the body to contain *only* the trivial expression — a function with a `match` statement that happens to return `Default::default()` in one arm is not flagged.
- Hits in string literals or comments are filtered by checking that the match position is not inside a `//` comment or a `"..."` string span.

## Testing the Script Itself

After writing, validate by running:

```bash
python3 scripts/find_stubs.py
cat scripts/stub_report.json | python3 -m json.tool | head -40
```

Expected: the two known `drop(stream)` hits in `server/src/http.rs` appear in the output.
