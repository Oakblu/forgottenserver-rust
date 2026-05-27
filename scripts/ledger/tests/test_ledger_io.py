"""Round-trip + edge-case tests for ledger_io."""
from __future__ import annotations

import os
import sys
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))

import ledger_io  # noqa: E402


class TestEmitter(unittest.TestCase):
    def test_empty_dict(self):
        self.assertEqual(ledger_io.dump({}), "\n")

    def test_simple_scalars(self):
        out = ledger_io.dump({"version": 2, "schema": "per-symbol"})
        self.assertEqual(out, "version: 2\nschema: per-symbol\n")

    def test_string_with_colon_is_quoted(self):
        out = ledger_io.dump({"signature": "void Actions::clearMap()"})
        self.assertIn('"void Actions::clearMap()"', out)

    def test_empty_list_inline(self):
        out = ledger_io.dump({"evidence": []})
        self.assertEqual(out, "evidence: []\n")

    def test_non_empty_list_block(self):
        out = ledger_io.dump({"risks": ["one", "two-three"]})
        self.assertEqual(out, "risks:\n  - one\n  - two-three\n")

    def test_nested_mapping(self):
        out = ledger_io.dump({"cpp": {"file": "src/x.cpp", "kind": "method"}})
        self.assertEqual(
            out,
            "cpp:\n"
            '  file: "src/x.cpp"\n'
            "  kind: method\n",
        )

    def test_list_of_mappings(self):
        out = ledger_io.dump({
            "rust": [{"file": "crates/a.rs", "kind": "impl_method"}],
        })
        self.assertEqual(
            out,
            "rust:\n"
            '  - file: "crates/a.rs"\n'
            "    kind: impl_method\n",
        )

    def test_string_with_quotes(self):
        out = ledger_io.dump({"k": 'has "quotes"'})
        # JSON-escaped
        self.assertIn(r'has \"quotes\"', out)


class TestReader(unittest.TestCase):
    def test_round_trip_simple(self):
        doc = {
            "version": 2,
            "schema": "per-symbol",
            "last_updated": "2026-05-26",
        }
        self.assertEqual(ledger_io.load(ledger_io.dump(doc)), doc)

    def test_round_trip_nested(self):
        doc = {
            "symbols": [
                {
                    "id": "x__y__z",
                    "cpp": {
                        "file": "src/x.cpp",
                        "symbol": "X::y",
                        "signature": "void X::y(int)",
                        "kind": "method",
                        "body_hash": "abc123",
                    },
                    "rust": [
                        {
                            "file": "crates/a.rs",
                            "symbol": "a::y",
                            "signature": "pub fn y(_: i32)",
                            "kind": "impl_method",
                            "body_hash": "def456",
                        },
                    ],
                    "status": "uncertain",
                    "migration_type": "unknown",
                    "confidence": "low",
                    "evidence": [],
                    "semantic_differences": [],
                    "risks": [],
                    "tests_required": [],
                    "differential_scenarios": [],
                    "notes": "",
                }
            ]
        }
        self.assertEqual(ledger_io.load(ledger_io.dump(doc)), doc)

    def test_comments_ignored(self):
        text = "# header\n# more\nversion: 2\n"
        self.assertEqual(ledger_io.load(text), {"version": 2})

    def test_empty_list_round_trip(self):
        doc = {"a": [], "b": ["x"]}
        self.assertEqual(ledger_io.load(ledger_io.dump(doc)), doc)

    def test_string_with_special_chars(self):
        sig = "std::vector<std::pair<int, bool>>& foo(const T& t)"
        doc = {"signature": sig}
        self.assertEqual(ledger_io.load(ledger_io.dump(doc)), doc)

    def test_numeric_looking_string_stays_string(self):
        # body_hash values can be all-digit (e.g. "2369646527424954").
        # They must round-trip as `str`, not `int`.
        doc = {"cpp": {"body_hash": "2369646527424954"}}
        out = ledger_io.load(ledger_io.dump(doc))
        self.assertIsInstance(out["cpp"]["body_hash"], str)
        self.assertEqual(out, doc)

    def test_odd_indent_rejected(self):
        with self.assertRaises(ledger_io.LedgerParseError):
            ledger_io.load("a:\n   b: 1\n")


class TestRoundTripWithRealisticRow(unittest.TestCase):
    def test_full_schema_row_round_trip(self):
        row = {
            "id": "actions__cpp__actions-clearmap",
            "cpp": {
                "file": "src/actions.cpp",
                "symbol": "Actions::clearMap",
                "signature": "void Actions::clearMap(ActionUseMap& map, bool fromLua)",
                "kind": "method",
                "body_hash": "0458bbf80f103b5b",
            },
            "rust": [],
            "status": "uncertain",
            "migration_type": "unknown",
            "confidence": "low",
            "evidence": [],
            "semantic_differences": [],
            "risks": [],
            "tests_required": [],
            "differential_scenarios": [],
            "notes": "",
        }
        doc = {"symbols": [row]}
        self.assertEqual(ledger_io.load(ledger_io.dump(doc)), doc)


if __name__ == "__main__":
    unittest.main()
