#!/usr/bin/env python3
"""Find stub functions in the Rust port codebase for migration ledger tracking.

Usage:
    python3 scripts/find_stubs.py
    # writes scripts/stub_report.json
"""

import json
import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).parent.parent
CRATES_DIR = ROOT / "crates"
MANIFEST_PATH = ROOT / "rust_symbol_manifest.json"
OUTPUT_PATH = Path(__file__).parent / "stub_report.json"


def strip_test_blocks(src: str) -> str:
    """Replace #[cfg(test)] mod/impl { ... } blocks with spaces (newlines preserved)."""
    marker = "#[cfg(test)]"
    chars = list(src)
    i = 0
    while i < len(src):
        if src[i : i + len(marker)] == marker:
            j = i + len(marker)
            # Skip whitespace and non-brace tokens (e.g. "mod tests", "impl Foo")
            # until we find the opening '{' of the associated item
            while j < len(src) and src[j] != "{":
                # If we hit another '#' or a newline after the first non-whitespace
                # word, there's no directly attached block — bail out.
                if src[j] == "#":
                    break
                j += 1
            if j < len(src) and src[j] == "{":
                depth, k = 1, j + 1
                while k < len(src) and depth:
                    if src[k] == "{":
                        depth += 1
                    elif src[k] == "}":
                        depth -= 1
                    k += 1
                # Blank out [i, k) but keep newlines so line numbers stay valid
                for m in range(i, k):
                    if chars[m] != "\n":
                        chars[m] = " "
                i = k
                continue
        i += 1
    return "".join(chars)


_FN_RE = re.compile(
    r"^\s*"
    r"(?:pub\s+)?(?:pub\s*\([^)]*\)\s+)?"                    # visibility
    r"(?:(?:async\s+)?(?:unsafe\s+)?|(?:unsafe\s+)?(?:async\s+)?)?"  # qualifiers (both orderings)
    r"fn\s+(\w+)"                                              # fn keyword + name
)


def find_fn_bodies(lines: list) -> list:
    """Return [{fn_name, start_line, end_line, body}] for every fn in lines.

    Uses brace-depth tracking over a 300-line lookahead window.
    start_line and end_line are 1-indexed.
    """
    results = []
    n = len(lines)
    i = 0
    while i < n:
        m = _FN_RE.match(lines[i])
        if not m:
            i += 1
            continue
        fn_name = m.group(1)
        start_line = i + 1
        # Join a lookahead window and track balanced braces
        window = "\n".join(lines[i : min(i + 300, n)])
        depth = 0
        open_idx = -1
        found = False
        in_string = False
        escape_next = False
        for pos, ch in enumerate(window):
            if escape_next:
                escape_next = False
                continue
            if ch == "\\" and in_string:
                escape_next = True
                continue
            if ch == '"' and not in_string:
                in_string = True
                continue
            if ch == '"' and in_string:
                in_string = False
                continue
            if in_string:
                continue
            if ch == "{":
                depth += 1
                if open_idx == -1:
                    open_idx = pos
            elif ch == "}":
                depth -= 1
                if depth == 0 and open_idx != -1:
                    body = window[open_idx + 1 : pos]
                    end_offset = window[:pos].count("\n")
                    results.append(
                        {
                            "fn_name": fn_name,
                            "start_line": start_line,
                            "end_line": i + end_offset + 1,
                            "body": body,
                        }
                    )
                    i += end_offset + 1
                    found = True
                    break
        if not found:
            i += 1
    return results


_TRIVIAL_BODY_RE = re.compile(
    r"^\s*"
    r"(?:"
    r"Ok\s*\(\s*\(\s*\)\s*\)"   # Ok(())
    r"|Err\s*\([^)]{0,80}\)"     # Err(something short)
    r"|Default::default\s*\(\)"  # Default::default()
    r"|false"
    r"|true"
    r"|None"
    r"|String::new\s*\(\)"
    r"|vec!\s*\[\s*\]"
    r"|-?\d+"                    # numeric literal
    r")"
    r"\s*;?\s*$",
    re.DOTALL,
)


def detect_empty_bodies(lines: list, bodies: list) -> list:
    """Flag functions whose body is empty (only whitespace)."""
    hits = []
    for b in bodies:
        if not b["body"].strip():
            hits.append(
                {
                    "pattern": "empty_body",
                    "fn_name": b["fn_name"],
                    "line": b["start_line"],
                    "snippet": lines[b["start_line"] - 1].strip()[:120],
                }
            )
    return hits


def detect_trivial_bodies(lines: list, bodies: list) -> list:
    """Flag functions whose body is a single trivially-wrong default expression."""
    hits = []
    for b in bodies:
        body = b["body"].strip()
        # Must be non-empty and match exactly one trivial expression
        if body and _TRIVIAL_BODY_RE.match(body):
            hits.append(
                {
                    "pattern": "trivial_body",
                    "fn_name": b["fn_name"],
                    "line": b["start_line"],
                    "snippet": body[:120],
                }
            )
    return hits


def main() -> None:
    manifest = load_manifest(MANIFEST_PATH)
    stubs = []
    for rs_path in walk_rs_files(CRATES_DIR):
        stubs.extend(scan_file(rs_path, CRATES_DIR, manifest))
    OUTPUT_PATH.write_text(json.dumps(stubs, indent=2))
    print(f"Wrote {len(stubs)} stubs to {OUTPUT_PATH}", file=sys.stderr)


if __name__ == "__main__":
    main()
