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


if __name__ == "__main__":
    unittest.main()
