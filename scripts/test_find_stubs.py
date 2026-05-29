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

    def test_string_literal_with_braces_in_body(self):
        src = 'fn tricky() { let s = "{hello}"; }\n'
        bodies = self._bodies(src)
        self.assertEqual(len(bodies), 1)
        self.assertEqual(bodies[0]["fn_name"], "tricky")


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

    def test_panic_in_string_literal_not_flagged(self):
        src = 'fn bar() { let msg = "panic!()"; }\n'
        hits = self._run(src)
        self.assertEqual(hits, [])


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
    """Integration: scan_file on synthetic Rust snippets."""

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


if __name__ == "__main__":
    unittest.main()
