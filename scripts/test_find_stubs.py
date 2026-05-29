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


if __name__ == "__main__":
    unittest.main()
