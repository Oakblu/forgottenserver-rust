#!/usr/bin/env python3
"""Find stub functions in the Rust port codebase for migration ledger tracking.

Usage:
    python3 scripts/find_stubs.py
    # writes scripts/stub_report.json (legacy flat array)
    # writes scripts/stub_report_structured.json (unresolved/confirmed split)

    python3 scripts/find_stubs.py --allowlist /path/to/custom.json
    # uses a custom allowlist instead of scripts/confirmed_stubs.json
"""

import argparse
import hashlib
import json
import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).parent.parent
CRATES_DIR = ROOT / "crates"
MANIFEST_PATH = ROOT / "rust_symbol_manifest.json"
OUTPUT_PATH = Path(__file__).parent / "stub_report.json"
STRUCTURED_OUTPUT_PATH = Path(__file__).parent / "stub_report_structured.json"
DEFAULT_ALLOWLIST_PATH = Path(__file__).parent / "confirmed_stubs.json"


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
                    "body": b["body"],
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
                    "body": b["body"],
                }
            )
    return hits


_DROP_RE = re.compile(r"\bdrop\s*\(")
_PANIC_RE = re.compile(r"\b(panic!|unreachable!)\s*\(")
_COMMENT_RE = re.compile(r"^\s*//")


def detect_dropped_work(lines: list, bodies: list) -> list:
    """Flag drop() calls in functions other than fn drop(&mut self)."""
    hits = []
    for b in bodies:
        if b["fn_name"] == "drop":
            continue
        for offset, line in enumerate(b["body"].split("\n")):
            if _DROP_RE.search(line) and not _COMMENT_RE.match(line):
                hits.append(
                    {
                        "pattern": "dropped_work",
                        "fn_name": b["fn_name"],
                        "line": b["start_line"] + offset,
                        "snippet": line.strip()[:120],
                        "body": b["body"],
                    }
                )
    return hits


def _strip_string_literals(line: str) -> str:
    """Replace contents of double-quoted strings with spaces (preserves length)."""
    result = list(line)
    in_string = False
    i = 0
    while i < len(line):
        ch = line[i]
        if ch == "\\" and in_string:
            # Skip escaped char
            if i + 1 < len(line):
                result[i + 1] = " "
            i += 2
            continue
        if ch == '"':
            in_string = not in_string
            i += 1
            continue
        if in_string:
            result[i] = " "
        i += 1
    return "".join(result)


def detect_panic_stubs(src: str) -> list:
    """Flag panic!/unreachable! calls in non-comment, non-string-literal lines.

    fn_name is set to '<unknown>' here; scan_file fills it in via enclosing_fn().
    """
    hits = []
    for i, line in enumerate(src.splitlines(), 1):
        if _COMMENT_RE.match(line):
            continue
        m = _PANIC_RE.search(_strip_string_literals(line))
        if m:
            hits.append(
                {
                    "pattern": "panic_stub",
                    "fn_name": "<unknown>",
                    "line": i,
                    "snippet": line.strip()[:120],
                    "body": line,
                }
            )
    return hits


def enclosing_fn(line: int, bodies: list) -> str:
    """Return the name of the function that contains the given 1-indexed line."""
    for b in bodies:
        if b["start_line"] <= line <= b["end_line"]:
            return b["fn_name"]
    return "<unknown>"


def compute_body_hash(body: str) -> str:
    """Compute a stable SHA-256 hash of a function body (stripped of leading/trailing whitespace)."""
    normalized = body.strip()
    return hashlib.sha256(normalized.encode("utf-8")).hexdigest()


def load_manifest(path: Path) -> dict:
    """Load rust_symbol_manifest.json into a {fn_name: [entry, ...]} dict.

    Keys are the last component of qualified_name (the bare function name).
    Returns {} if the file is missing or malformed.
    """
    try:
        data = json.loads(path.read_text())
    except Exception:
        return {}
    lookup: dict = {}
    for entry in data:
        qname = entry.get("qualified_name", "")
        key = qname.split("::")[-1] if qname else ""
        if key:
            lookup.setdefault(key, []).append(entry)
    return lookup


def load_allowlist(path: Path) -> dict:
    """Load confirmed_stubs.json into a lookup dict keyed by (file, fn_name, line).

    Returns {} if the file is missing or malformed.
    Each entry maps (file, fn_name, line) -> body_hash.
    """
    try:
        data = json.loads(path.read_text())
    except Exception:
        return {}
    lookup: dict = {}
    for entry in data:
        key = (
            entry.get("file", ""),
            entry.get("fn_name", ""),
            entry.get("line", -1),
        )
        lookup[key] = entry.get("body_hash", "")
    return lookup


def walk_rs_files(root: Path):
    """Yield every .rs file under root, skipping 'tests' sub-directories."""
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d != "tests"]
        for fname in filenames:
            if fname.endswith(".rs"):
                yield Path(dirpath) / fname


def scan_file(path: Path, crates_dir: Path, manifest: dict) -> list:
    """Scan a single .rs file for stubs and return a list of hit dicts."""
    try:
        raw = path.read_text(encoding="utf-8", errors="replace")
    except Exception:
        return []

    src = strip_test_blocks(raw)
    lines = src.splitlines()
    bodies = find_fn_bodies(lines)

    hits: list = []
    hits.extend(detect_empty_bodies(lines, bodies))
    hits.extend(detect_trivial_bodies(lines, bodies))
    hits.extend(detect_dropped_work(lines, bodies))

    for hit in detect_panic_stubs(src):
        hit["fn_name"] = enclosing_fn(hit["line"], bodies)
        # Attach the body from the enclosing function for hash computation
        for b in bodies:
            if b["fn_name"] == hit["fn_name"]:
                hit["body"] = b["body"]
                break
        hits.append(hit)

    # Enrich with file metadata, body hash, and manifest cross-reference
    try:
        rel = str(path.relative_to(crates_dir))
    except ValueError:
        rel = str(path)
    crate = rel.split(os.sep)[0]

    result = []
    for hit in hits:
        body = hit.pop("body", "")
        hit["file"] = rel
        hit["crate"] = crate
        hit["body_hash"] = compute_body_hash(body)
        matches = manifest.get(hit["fn_name"], [])
        hit["ledger_symbol"] = (
            matches[0].get("qualified_name") if len(matches) == 1 else None
        )
        hit["manifest_match"] = (
            matches[0] if len(matches) == 1 else (matches if matches else None)
        )
        result.append(hit)

    return result


def partition_stubs(stubs: list, allowlist: dict) -> tuple:
    """Partition stubs into (unresolved, confirmed) lists using the allowlist.

    Emits WARN to stderr when a confirmed stub's body hash has changed.
    Returns (unresolved_list, confirmed_list).
    """
    unresolved = []
    confirmed = []
    for stub in stubs:
        key = (stub.get("file", ""), stub.get("fn_name", ""), stub.get("line", -1))
        if key not in allowlist:
            unresolved.append(stub)
        else:
            expected_hash = allowlist[key]
            actual_hash = stub.get("body_hash", "")
            if expected_hash and actual_hash and expected_hash != actual_hash:
                print(
                    f"WARN: confirmed stub body changed: "
                    f"{stub['file']}:{stub['fn_name']} (line {stub['line']})",
                    file=sys.stderr,
                )
                unresolved.append(stub)
            else:
                confirmed.append(stub)
    return unresolved, confirmed


def main(args=None) -> None:
    parser = argparse.ArgumentParser(
        description="Find stub functions in the Rust port codebase."
    )
    parser.add_argument(
        "--allowlist",
        metavar="PATH",
        default=None,
        help=(
            "Path to confirmed_stubs.json allowlist. "
            f"Defaults to {DEFAULT_ALLOWLIST_PATH} if it exists."
        ),
    )
    parsed = parser.parse_args(args)

    # Resolve allowlist path
    if parsed.allowlist is not None:
        allowlist_path = Path(parsed.allowlist)
    else:
        allowlist_path = DEFAULT_ALLOWLIST_PATH

    allowlist = load_allowlist(allowlist_path)

    manifest = load_manifest(MANIFEST_PATH)
    stubs = []
    for rs_path in walk_rs_files(CRATES_DIR):
        stubs.extend(scan_file(rs_path, CRATES_DIR, manifest))

    unresolved, confirmed = partition_stubs(stubs, allowlist)

    # Legacy backward-compat: write flat array of ALL stubs to stub_report.json
    # (strip body_hash from legacy output to preserve existing format)
    legacy_stubs = [{k: v for k, v in s.items() if k != "body_hash"} for s in stubs]
    OUTPUT_PATH.write_text(json.dumps(legacy_stubs, indent=2))

    # New structured format: write to stub_report_structured.json
    structured = {
        "unresolved": unresolved,
        "confirmed": confirmed,
    }
    STRUCTURED_OUTPUT_PATH.write_text(json.dumps(structured, indent=2))

    print(
        f"Wrote {len(stubs)} stubs to {OUTPUT_PATH} "
        f"({len(unresolved)} unresolved, {len(confirmed)} confirmed)",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
