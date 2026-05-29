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
