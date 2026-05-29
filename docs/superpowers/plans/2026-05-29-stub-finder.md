# Stub Finder Script Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Write `scripts/find_stubs.py` — a stdlib-only Python 3 scanner that detects stub functions in all Rust source files and writes `scripts/stub_report.json` for migration ledger tracking.

**Architecture:** Single-pass file walker. Each `.rs` file is read, test blocks are stripped, function bodies are located via brace-depth tracking, four pattern detectors run over the bodies, hits are cross-referenced against `rust_symbol_manifest.json`, and everything is serialised to JSON.

**Tech Stack:** Python 3 stdlib only (`re`, `json`, `os`, `pathlib`, `unittest`)

---

## File Map

| Action | Path | Responsibility |
|--------|------|---------------|
| Create | `scripts/find_stubs.py` | Scanner + all detectors + entry point |
| Create | `scripts/test_find_stubs.py` | Unit tests for every public function |
| Generated | `scripts/stub_report.json` | Output (not committed) |

---

### Task 1: Script skeleton and test runner

**Files:**
- Create: `scripts/find_stubs.py`
- Create: `scripts/test_find_stubs.py`

- [ ] **Step 1: Create `scripts/find_stubs.py`**

```python
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


def main() -> None:
    manifest = load_manifest(MANIFEST_PATH)
    stubs = []
    for rs_path in walk_rs_files(CRATES_DIR):
        stubs.extend(scan_file(rs_path, CRATES_DIR, manifest))
    OUTPUT_PATH.write_text(json.dumps(stubs, indent=2))
    print(f"Wrote {len(stubs)} stubs to {OUTPUT_PATH}", file=sys.stderr)


if __name__ == "__main__":
    main()
```

- [ ] **Step 2: Create `scripts/test_find_stubs.py`**

```python
#!/usr/bin/env python3
"""Unit tests for scripts/find_stubs.py."""

import unittest
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import find_stubs


class TestImport(unittest.TestCase):
    def test_main_exists(self):
        self.assertTrue(callable(find_stubs.main))


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 3: Run tests**

```bash
python3 scripts/test_find_stubs.py -v
```
Expected: `OK` (1 test passes)

- [ ] **Step 4: Commit**

```bash
git add scripts/find_stubs.py scripts/test_find_stubs.py
git commit -m "feat(scripts): stub-finder skeleton and test runner"
```

---

### Task 2: `strip_test_blocks(src: str) -> str`

**Files:**
- Modify: `scripts/find_stubs.py`
- Modify: `scripts/test_find_stubs.py`

Replaces `#[cfg(test)] { ... }` blocks with spaces, preserving newlines so that all line numbers in the file remain stable.

- [ ] **Step 1: Add the failing tests**

Append to `scripts/test_find_stubs.py`:

```python
class TestStripTestBlocks(unittest.TestCase):
    def test_strips_cfg_test_mod(self):
        src = (
            "fn real() { 42 }\n"
            "\n"
            "#[cfg(test)]\n"
            "mod tests {\n"
            "    fn inner() { 99 }\n"
            "}\n"
        )
        out = find_stubs.strip_test_blocks(src)
        self.assertIn("fn real()", out)
        self.assertNotIn("fn inner()", out)
        # Line count must be identical so line numbers stay valid
        self.assertEqual(src.count("\n"), out.count("\n"))

    def test_no_test_block_is_unchanged(self):
        src = "fn foo() { 1 }\n"
        self.assertEqual(src, find_stubs.strip_test_blocks(src))

    def test_nested_braces_in_test_block_handled(self):
        src = (
            "#[cfg(test)]\n"
            "mod t {\n"
            "    fn a() { if true { 1 } else { 2 } }\n"
            "}\n"
            "fn real() {}\n"
        )
        out = find_stubs.strip_test_blocks(src)
        self.assertIn("fn real()", out)
        self.assertNotIn("fn a()", out)

    def test_multiple_test_blocks(self):
        src = (
            "#[cfg(test)]\nmod a { fn x(){} }\n"
            "fn middle() {}\n"
            "#[cfg(test)]\nmod b { fn y(){} }\n"
        )
        out = find_stubs.strip_test_blocks(src)
        self.assertIn("fn middle()", out)
        self.assertNotIn("fn x()", out)
        self.assertNotIn("fn y()", out)
```

- [ ] **Step 2: Run — expected failure**

```bash
python3 scripts/test_find_stubs.py -v 2>&1 | tail -6
```
Expected: `AttributeError: module 'find_stubs' has no attribute 'strip_test_blocks'`

- [ ] **Step 3: Implement `strip_test_blocks` in `scripts/find_stubs.py`**

Add after the constants:

```python
def strip_test_blocks(src: str) -> str:
    """Replace #[cfg(test)] { ... } blocks with spaces (newlines preserved)."""
    marker = "#[cfg(test)]"
    chars = list(src)
    i = 0
    while i < len(src):
        if src[i : i + len(marker)] == marker:
            j = i + len(marker)
            # Skip whitespace to find the opening brace
            while j < len(src) and src[j] in " \t\n\r":
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
```

- [ ] **Step 4: Run — expected pass**

```bash
python3 scripts/test_find_stubs.py -v
```
Expected: `OK` (5 tests)

- [ ] **Step 5: Commit**

```bash
git add scripts/find_stubs.py scripts/test_find_stubs.py
git commit -m "feat(scripts): implement strip_test_blocks"
```

---

### Task 3: `find_fn_bodies(lines: list) -> list`

**Files:**
- Modify: `scripts/find_stubs.py`
- Modify: `scripts/test_find_stubs.py`

Core utility. Given a list of source lines, returns all function bodies as:
`{"fn_name": str, "start_line": int, "end_line": int, "body": str}` (1-indexed).

- [ ] **Step 1: Add the failing tests**

Append to `scripts/test_find_stubs.py`:

```python
class TestFindFnBodies(unittest.TestCase):
    def _bodies(self, src):
        return find_stubs.find_fn_bodies(src.splitlines())

    def test_single_line_fn(self):
        bodies = self._bodies("fn foo() { 42 }\n")
        self.assertEqual(len(bodies), 1)
        self.assertEqual(bodies[0]["fn_name"], "foo")
        self.assertIn("42", bodies[0]["body"])

    def test_multi_line_signature(self):
        src = "fn bar(\n    x: i32,\n) -> i32 {\n    x + 1\n}\n"
        bodies = self._bodies(src)
        self.assertEqual(len(bodies), 1)
        self.assertEqual(bodies[0]["fn_name"], "bar")
        self.assertIn("x + 1", bodies[0]["body"])

    def test_two_sequential_fns(self):
        src = "fn a() { 1 }\nfn b() { 2 }\n"
        bodies = self._bodies(src)
        names = [b["fn_name"] for b in bodies]
        self.assertIn("a", names)
        self.assertIn("b", names)

    def test_empty_body(self):
        bodies = self._bodies("pub fn noop() {}\n")
        self.assertEqual(len(bodies), 1)
        self.assertEqual(bodies[0]["body"].strip(), "")

    def test_start_line_is_correct(self):
        src = "// comment\nfn foo() {\n    1\n}\n"
        bodies = self._bodies(src)
        self.assertEqual(bodies[0]["start_line"], 2)

    def test_pub_async_fn(self):
        bodies = self._bodies("pub async fn run() { loop {} }\n")
        self.assertEqual(bodies[0]["fn_name"], "run")

    def test_nested_braces_in_body(self):
        src = "fn outer() {\n    if x { a() } else { b() }\n}\n"
        bodies = self._bodies(src)
        self.assertEqual(len(bodies), 1)
        self.assertEqual(bodies[0]["fn_name"], "outer")
```

- [ ] **Step 2: Run — expected failure**

```bash
python3 scripts/test_find_stubs.py -v 2>&1 | tail -4
```
Expected: `AttributeError: module 'find_stubs' has no attribute 'find_fn_bodies'`

- [ ] **Step 3: Implement `find_fn_bodies` in `scripts/find_stubs.py`**

Add after `strip_test_blocks`:

```python
_FN_RE = re.compile(
    r"^\s*"
    r"(?:pub\s+)?(?:pub\s*\([^)]*\)\s+)?"  # visibility
    r"(?:async\s+)?(?:unsafe\s+)?"           # qualifiers
    r"fn\s+(\w+)"                             # fn keyword + name
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
        for pos, ch in enumerate(window):
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
```

- [ ] **Step 4: Run — expected pass**

```bash
python3 scripts/test_find_stubs.py -v
```
Expected: `OK` (12 tests)

- [ ] **Step 5: Commit**

```bash
git add scripts/find_stubs.py scripts/test_find_stubs.py
git commit -m "feat(scripts): implement find_fn_bodies with brace-depth tracking"
```

---

### Task 4: `detect_empty_bodies` and `detect_trivial_bodies`

**Files:**
- Modify: `scripts/find_stubs.py`
- Modify: `scripts/test_find_stubs.py`

Two detectors that operate on the output of `find_fn_bodies`. Both return a list of hit dicts with keys `pattern`, `fn_name`, `line`, `snippet`.

- [ ] **Step 1: Add the failing tests**

Append to `scripts/test_find_stubs.py`:

```python
class TestDetectEmptyBodies(unittest.TestCase):
    def _run(self, src):
        lines = src.splitlines()
        bodies = find_stubs.find_fn_bodies(lines)
        return find_stubs.detect_empty_bodies(lines, bodies)

    def test_empty_body_detected(self):
        hits = self._run("pub fn noop() {}\n")
        self.assertEqual(len(hits), 1)
        self.assertEqual(hits[0]["pattern"], "empty_body")
        self.assertEqual(hits[0]["fn_name"], "noop")

    def test_non_empty_body_not_flagged(self):
        hits = self._run("fn real() { let x = 1; x }\n")
        self.assertEqual(hits, [])

    def test_whitespace_only_body_flagged(self):
        hits = self._run("fn noop() {\n    \n}\n")
        self.assertEqual(len(hits), 1)


class TestDetectTrivialBodies(unittest.TestCase):
    def _run(self, src):
        lines = src.splitlines()
        bodies = find_stubs.find_fn_bodies(lines)
        return find_stubs.detect_trivial_bodies(lines, bodies)

    def test_ok_unit_flagged(self):
        hits = self._run("fn begin() -> Result<(), E> { Ok(()) }\n")
        self.assertEqual(len(hits), 1)
        self.assertEqual(hits[0]["pattern"], "trivial_body")

    def test_default_default_flagged(self):
        hits = self._run("fn make() -> Foo { Default::default() }\n")
        self.assertEqual(len(hits), 1)

    def test_false_flagged(self):
        hits = self._run("fn is_ready(&self) -> bool { false }\n")
        self.assertEqual(len(hits), 1)

    def test_true_flagged(self):
        hits = self._run("fn is_creature(&self) -> bool { true }\n")
        self.assertEqual(len(hits), 1)

    def test_none_flagged(self):
        hits = self._run("fn get(&self) -> Option<u32> { None }\n")
        self.assertEqual(len(hits), 1)

    def test_real_body_not_flagged(self):
        hits = self._run("fn compute(x: u32) -> u32 { x * 2 }\n")
        self.assertEqual(hits, [])

    def test_match_body_not_flagged(self):
        # A function with a match is not trivial even if one arm returns false
        hits = self._run(
            "fn check(x: u32) -> bool {\n"
            "    match x { 0 => false, _ => true }\n"
            "}\n"
        )
        self.assertEqual(hits, [])
```

- [ ] **Step 2: Run — expected failure**

```bash
python3 scripts/test_find_stubs.py -v 2>&1 | grep "ERROR\|AttributeError" | head -4
```
Expected: `AttributeError: module 'find_stubs' has no attribute 'detect_empty_bodies'`

- [ ] **Step 3: Implement both detectors in `scripts/find_stubs.py`**

Add after `find_fn_bodies`:

```python
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
```

- [ ] **Step 4: Run — expected pass**

```bash
python3 scripts/test_find_stubs.py -v
```
Expected: `OK` (22 tests)

- [ ] **Step 5: Commit**

```bash
git add scripts/find_stubs.py scripts/test_find_stubs.py
git commit -m "feat(scripts): detect_empty_bodies and detect_trivial_bodies"
```

---

### Task 5: `detect_dropped_work` and `detect_panic_stubs`

**Files:**
- Modify: `scripts/find_stubs.py`
- Modify: `scripts/test_find_stubs.py`

- [ ] **Step 1: Add the failing tests**

Append to `scripts/test_find_stubs.py`:

```python
class TestDetectDroppedWork(unittest.TestCase):
    def _run(self, src):
        lines = src.splitlines()
        bodies = find_stubs.find_fn_bodies(lines)
        return find_stubs.detect_dropped_work(lines, bodies)

    def test_drop_in_accept_loop_flagged(self):
        src = (
            "fn accept_loop() {\n"
            "    let stream = listener.accept();\n"
            "    drop(stream);\n"
            "}\n"
        )
        hits = self._run(src)
        self.assertEqual(len(hits), 1)
        self.assertEqual(hits[0]["pattern"], "dropped_work")
        self.assertEqual(hits[0]["fn_name"], "accept_loop")

    def test_impl_drop_excluded(self):
        # fn drop(&mut self) is the real Drop trait — never a stub
        src = "fn drop(&mut self) {\n    drop(self.inner);\n}\n"
        hits = self._run(src)
        self.assertEqual(hits, [])

    def test_drop_in_real_code_flagged(self):
        src = "fn handle(conn: TcpStream) {\n    drop(conn);\n}\n"
        hits = self._run(src)
        self.assertEqual(len(hits), 1)


class TestDetectPanicStubs(unittest.TestCase):
    def _run(self, src):
        return find_stubs.detect_panic_stubs(src)

    def test_panic_flagged(self):
        src = 'fn convert(x: u8) -> Foo {\n    panic!("not implemented")\n}\n'
        hits = self._run(src)
        self.assertEqual(len(hits), 1)
        self.assertEqual(hits[0]["pattern"], "panic_stub")

    def test_unreachable_flagged(self):
        src = "fn from(x: u8) -> Self {\n    unreachable!()\n}\n"
        hits = self._run(src)
        self.assertEqual(len(hits), 1)

    def test_panic_in_comment_not_flagged(self):
        src = "// panic!() would crash here\nfn foo() { 1 }\n"
        hits = self._run(src)
        self.assertEqual(hits, [])
```

- [ ] **Step 2: Run — expected failure**

```bash
python3 scripts/test_find_stubs.py -v 2>&1 | grep "AttributeError" | head -2
```
Expected: `AttributeError: module 'find_stubs' has no attribute 'detect_dropped_work'`

- [ ] **Step 3: Implement both detectors in `scripts/find_stubs.py`**

Add after `detect_trivial_bodies`:

```python
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
                    }
                )
    return hits


def detect_panic_stubs(src: str) -> list:
    """Flag panic!/unreachable! calls in non-comment lines.

    fn_name is set to '<unknown>' here; scan_file fills it in via enclosing_fn().
    """
    hits = []
    for i, line in enumerate(src.splitlines(), 1):
        if _COMMENT_RE.match(line):
            continue
        m = _PANIC_RE.search(line)
        if m:
            hits.append(
                {
                    "pattern": "panic_stub",
                    "fn_name": "<unknown>",
                    "line": i,
                    "snippet": line.strip()[:120],
                }
            )
    return hits
```

- [ ] **Step 4: Run — expected pass**

```bash
python3 scripts/test_find_stubs.py -v
```
Expected: `OK` (29 tests)

- [ ] **Step 5: Commit**

```bash
git add scripts/find_stubs.py scripts/test_find_stubs.py
git commit -m "feat(scripts): detect_dropped_work and detect_panic_stubs"
```

---

### Task 6: `enclosing_fn`, `load_manifest`, `walk_rs_files`, `scan_file`, and `main`

**Files:**
- Modify: `scripts/find_stubs.py`
- Modify: `scripts/test_find_stubs.py`

Wire everything together: find enclosing function for line-level hits, load the symbol manifest for cross-referencing, walk the crates directory, and produce the JSON output.

- [ ] **Step 1: Add the failing tests**

Append to `scripts/test_find_stubs.py`:

```python
class TestEnclosingFn(unittest.TestCase):
    def test_finds_enclosing(self):
        bodies = [{"fn_name": "foo", "start_line": 2, "end_line": 5}]
        self.assertEqual(find_stubs.enclosing_fn(3, bodies), "foo")

    def test_returns_unknown_when_outside(self):
        bodies = [{"fn_name": "foo", "start_line": 2, "end_line": 5}]
        self.assertEqual(find_stubs.enclosing_fn(10, bodies), "<unknown>")

    def test_boundary_lines_included(self):
        bodies = [{"fn_name": "bar", "start_line": 1, "end_line": 3}]
        self.assertEqual(find_stubs.enclosing_fn(1, bodies), "bar")
        self.assertEqual(find_stubs.enclosing_fn(3, bodies), "bar")


class TestLoadManifest(unittest.TestCase):
    def test_empty_file_returns_empty_dict(self):
        import tempfile, json
        with tempfile.NamedTemporaryFile(suffix=".json", mode="w", delete=False) as f:
            json.dump([], f)
            name = f.name
        result = find_stubs.load_manifest(Path(name))
        self.assertEqual(result, {})

    def test_entry_keyed_by_fn_name(self):
        import tempfile, json
        entry = {
            "file": "crates/foo/src/lib.rs",
            "kind": "fn",
            "qualified_name": "foo::bar::baz",
        }
        with tempfile.NamedTemporaryFile(suffix=".json", mode="w", delete=False) as f:
            json.dump([entry], f)
            name = f.name
        result = find_stubs.load_manifest(Path(name))
        self.assertIn("baz", result)
        self.assertEqual(result["baz"][0]["qualified_name"], "foo::bar::baz")

    def test_missing_file_returns_empty_dict(self):
        result = find_stubs.load_manifest(Path("/nonexistent/path.json"))
        self.assertEqual(result, {})


class TestScanFileIntegration(unittest.TestCase):
    """Integration: scan_file on a synthetic Rust snippet."""

    def _scan(self, src):
        import tempfile
        with tempfile.NamedTemporaryFile(
            suffix=".rs", mode="w", dir="/tmp", delete=False
        ) as f:
            f.write(src)
            p = Path(f.name)
        hits = find_stubs.scan_file(p, Path("/tmp"), {})
        p.unlink()
        return hits

    def test_drop_stub_detected(self):
        src = "fn accept() {\n    let s = listener.accept();\n    drop(s);\n}\n"
        hits = self._scan(src)
        patterns = [h["pattern"] for h in hits]
        self.assertIn("dropped_work", patterns)

    def test_empty_body_detected(self):
        src = "pub fn noop() {}\n"
        hits = self._scan(src)
        self.assertTrue(any(h["pattern"] == "empty_body" for h in hits))

    def test_test_block_excluded(self):
        src = (
            "fn real() {}\n"
            "#[cfg(test)]\n"
            "mod tests { fn stub_in_test() {} }\n"
        )
        hits = self._scan(src)
        # Only 'real' should appear; stub_in_test is in a cfg(test) block
        fn_names = [h["fn_name"] for h in hits]
        self.assertNotIn("stub_in_test", fn_names)

    def test_output_has_required_keys(self):
        src = "pub fn noop() {}\n"
        hits = self._scan(src)
        self.assertTrue(len(hits) > 0)
        required = {"file", "line", "crate", "fn_name", "pattern", "snippet",
                    "ledger_symbol", "manifest_match"}
        for hit in hits:
            self.assertEqual(required, required & hit.keys())
```

- [ ] **Step 2: Run — expected failure**

```bash
python3 scripts/test_find_stubs.py -v 2>&1 | grep "AttributeError" | head -2
```
Expected: `AttributeError: module 'find_stubs' has no attribute 'enclosing_fn'`

- [ ] **Step 3: Implement the remaining functions in `scripts/find_stubs.py`**

Add after `detect_panic_stubs`:

```python
def enclosing_fn(line: int, bodies: list) -> str:
    """Return the name of the function that contains the given 1-indexed line."""
    for b in bodies:
        if b["start_line"] <= line <= b["end_line"]:
            return b["fn_name"]
    return "<unknown>"


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
        hits.append(hit)

    # Enrich with file metadata and manifest cross-reference
    try:
        rel = str(path.relative_to(crates_dir))
    except ValueError:
        rel = str(path)
    crate = rel.split(os.sep)[0]

    for hit in hits:
        hit["file"] = rel
        hit["crate"] = crate
        matches = manifest.get(hit["fn_name"], [])
        hit["ledger_symbol"] = (
            matches[0].get("qualified_name") if len(matches) == 1 else None
        )
        hit["manifest_match"] = (
            matches[0] if len(matches) == 1 else (matches if matches else None)
        )

    return hits
```

- [ ] **Step 4: Run — expected pass**

```bash
python3 scripts/test_find_stubs.py -v
```
Expected: `OK` (41 tests)

- [ ] **Step 5: Commit**

```bash
git add scripts/find_stubs.py scripts/test_find_stubs.py
git commit -m "feat(scripts): enclosing_fn, load_manifest, walk_rs_files, scan_file"
```

---

### Task 7: End-to-end run and validation

**Files:**
- No code changes — validation only

- [ ] **Step 1: Run the script against the real codebase**

```bash
python3 scripts/find_stubs.py
```
Expected stderr: `Wrote N stubs to scripts/stub_report.json` (N should be >10 based on the earlier sweep)

- [ ] **Step 2: Validate JSON is well-formed**

```bash
python3 -m json.tool scripts/stub_report.json > /dev/null && echo "valid JSON"
```
Expected: `valid JSON`

- [ ] **Step 3: Verify the known drop(stream) stubs appear**

```bash
python3 -c "
import json
data = json.load(open('scripts/stub_report.json'))
drops = [h for h in data if h['pattern'] == 'dropped_work' and 'http' in h['file']]
for d in drops:
    print(d['file'], d['line'], d['snippet'])
print(f'Total hits: {len(data)}')
"
```
Expected: at least the two known hits from `server/src/http.rs` (lines 218 and 425) appear.

- [ ] **Step 4: Print a summary by pattern type**

```bash
python3 -c "
import json
from collections import Counter
data = json.load(open('scripts/stub_report.json'))
counts = Counter(h['pattern'] for h in data)
for pat, n in sorted(counts.items()):
    print(f'  {pat}: {n}')
print(f'  total: {len(data)}')
"
```

- [ ] **Step 5: Add `stub_report.json` to `.gitignore` (it's generated output)**

```bash
echo "scripts/stub_report.json" >> .gitignore
git add .gitignore
```

- [ ] **Step 6: Final commit**

```bash
git add scripts/find_stubs.py scripts/test_find_stubs.py .gitignore
git commit -m "feat(scripts): stub-finder complete — find_stubs.py + tests"
```

---

## Self-Review Checklist

**Spec coverage:**
- [x] All four patterns implemented (empty_body, trivial_body, dropped_work, panic_stub)
- [x] Test blocks excluded (strip_test_blocks + walk skips `tests/` dirs)
- [x] JSON output with all required keys (file, line, crate, fn_name, pattern, snippet, ledger_symbol, manifest_match)
- [x] Manifest cross-reference via load_manifest
- [x] stdlib only — no third-party deps

**Placeholder scan:** No TBD or "implement later" in any step. All code blocks are complete. ✓

**Type consistency:**
- `find_fn_bodies` returns `{"fn_name", "start_line", "end_line", "body"}` — same keys used in all detectors and `enclosing_fn`. ✓
- `detect_*` functions all return `{"pattern", "fn_name", "line", "snippet"}` — `scan_file` adds `{"file", "crate", "ledger_symbol", "manifest_match"}` to each. ✓
- `load_manifest` keyed by bare function name (`qname.split("::")[-1]`) — same key used in `scan_file` lookup. ✓
