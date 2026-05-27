"""Pipeline tests: build_seed → validate → rollup on a synthetic fixture."""
from __future__ import annotations

import json
import os
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))

import build_seed  # noqa: E402
import ledger_io  # noqa: E402
import rollup  # noqa: E402
import validate as validator  # noqa: E402


CPP_FIXTURE = [
    {
        "file": "src/position.cpp",
        "kind": "method",
        "qualified_name": "Position::operator==",
        "signature": "bool Position::operator==(Position const&) const",
        "visibility": "public",
        "body_hash": "aaaa1111",
    },
    {
        "file": "src/position.cpp",
        "kind": "method",
        "qualified_name": "Position::getDistanceX",
        "signature": "int32_t Position::getDistanceX(Position const&) const",
        "visibility": "public",
        "body_hash": "bbbb2222",
    },
    {
        "file": "src/tools.cpp",
        "kind": "free_function",
        "qualified_name": "stringToFruit",
        "signature": "Fruit stringToFruit(std::string const&)",
        "visibility": "public",
        "body_hash": "cccc3333",
    },
    {
        "file": "src/legacy_dropped.cpp",
        "kind": "free_function",
        "qualified_name": "deprecatedHelper",
        "signature": "void deprecatedHelper()",
        "visibility": "public",
        "body_hash": "dddd4444",
    },
    {
        "file": "src/const.h",
        "kind": "constant",
        "qualified_name": "MAX_PLAYERS",
        "signature": "constexpr int MAX_PLAYERS = 64;",
        "visibility": "public",
        "body_hash": "eeee5555",
    },
]

RUST_FIXTURE = [
    {
        "file": "crates/common/src/position.rs",
        "kind": "impl_method",
        "qualified_name": "common::position::Position::eq",
        "signature": "pub fn eq(&self, other: &Position) -> bool",
        "body_hash": "ffff6666",
    },
    {
        "file": "crates/common/src/position.rs",
        "kind": "impl_method",
        "qualified_name": "common::position::Position::get_distance_x",
        "signature": "pub fn get_distance_x(&self, other: &Position) -> i32",
        "body_hash": "9999aaaa",
    },
    {
        "file": "crates/common/src/tools.rs",
        "kind": "free_function",
        "qualified_name": "common::tools::string_to_fruit",
        "signature": "pub fn string_to_fruit(s: &str) -> Fruit",
        "body_hash": "8888bbbb",
    },
    {
        "file": "crates/common/src/constants.rs",
        "kind": "constant",
        "qualified_name": "common::constants::MAX_PLAYERS",
        "signature": "pub const MAX_PLAYERS: i32 = 64;",
        "body_hash": "7777cccc",
    },
]


class TestBuildSeed(unittest.TestCase):
    def test_seed_has_one_row_per_cpp_symbol(self):
        doc = build_seed.build_doc(CPP_FIXTURE, RUST_FIXTURE)
        self.assertEqual(doc["schema"], "per-symbol")
        self.assertEqual(len(doc["symbols"]), len(CPP_FIXTURE))

    def test_seed_ids_unique(self):
        doc = build_seed.build_doc(CPP_FIXTURE, RUST_FIXTURE)
        ids = [r["id"] for r in doc["symbols"]]
        self.assertEqual(len(ids), len(set(ids)))

    def test_seed_status_is_uncertain(self):
        doc = build_seed.build_doc(CPP_FIXTURE, RUST_FIXTURE)
        for r in doc["symbols"]:
            self.assertEqual(r["status"], "uncertain")
            self.assertEqual(r["migration_type"], "unknown")
            self.assertEqual(r["confidence"], "low")

    def test_seed_finds_rust_candidates_when_obvious(self):
        doc = build_seed.build_doc(CPP_FIXTURE, RUST_FIXTURE)
        rows = {r["cpp"]["symbol"]: r for r in doc["symbols"]}
        # stringToFruit → string_to_fruit
        cands = rows["stringToFruit"]["rust"]
        self.assertTrue(any("string_to_fruit" in c["symbol"] for c in cands))
        # deprecatedHelper has no Rust analogue
        self.assertEqual(rows["deprecatedHelper"]["rust"], [])
        # MAX_PLAYERS → MAX_PLAYERS
        self.assertTrue(any("MAX_PLAYERS" in c["symbol"] for c in rows["MAX_PLAYERS"]["rust"]))

    def test_seed_round_trips_through_yaml(self):
        doc = build_seed.build_doc(CPP_FIXTURE, RUST_FIXTURE)
        text = ledger_io.dump(doc)
        parsed = ledger_io.load(text)
        self.assertEqual(parsed["symbols"], doc["symbols"])


class TestValidate(unittest.TestCase):
    def _write_inputs(self, ledger_doc):
        tmp = tempfile.mkdtemp()
        cpp_path = os.path.join(tmp, "cpp.json")
        rust_path = os.path.join(tmp, "rust.json")
        ledger_path = os.path.join(tmp, "MIGRATION_LEDGER.yml")
        intentional_path = os.path.join(tmp, "intentional_differences.yml")
        with open(cpp_path, "w") as f:
            json.dump(CPP_FIXTURE, f)
        with open(rust_path, "w") as f:
            json.dump(RUST_FIXTURE, f)
        with open(ledger_path, "w") as f:
            f.write(ledger_io.dump(ledger_doc))
        with open(intentional_path, "w") as f:
            f.write("differences:\n  - id: handles-instead-of-raw-pointers\n")
        return cpp_path, rust_path, ledger_path, intentional_path

    def test_clean_seed_passes(self):
        doc = build_seed.build_doc(CPP_FIXTURE, RUST_FIXTURE)
        cpp_path, rust_path, ledger_path, intentional_path = self._write_inputs(doc)
        rc = validator.main([
            "--cpp", cpp_path, "--rust", rust_path,
            "--ledger", ledger_path, "--intentional", intentional_path,
        ])
        self.assertEqual(rc, 0)

    def test_missing_manifest_row_is_caught(self):
        doc = build_seed.build_doc(CPP_FIXTURE, RUST_FIXTURE)
        doc["symbols"] = doc["symbols"][:-1]   # drop one row
        cpp_path, rust_path, ledger_path, intentional_path = self._write_inputs(doc)
        rc = validator.main([
            "--cpp", cpp_path, "--rust", rust_path,
            "--ledger", ledger_path, "--intentional", intentional_path,
        ])
        self.assertEqual(rc, 1)

    def test_migrated_without_test_evidence_fails(self):
        doc = build_seed.build_doc(CPP_FIXTURE, RUST_FIXTURE)
        # Force first row to migrated with high confidence but no test: evidence.
        r = doc["symbols"][0]
        r["status"] = "migrated"
        r["migration_type"] = "direct"
        r["confidence"] = "high"
        r["rust"] = [{
            "file": "crates/common/src/position.rs",
            "symbol": "common::position::Position::eq",
            "signature": "pub fn eq(&self, other: &Position) -> bool",
            "kind": "impl_method",
            "body_hash": "ffff6666",
        }]
        r["evidence"] = ["cpp:src/position.cpp:10-12 — operator==", "rust:crates/common/src/position.rs:5-7 — eq"]
        cpp_path, rust_path, ledger_path, intentional_path = self._write_inputs(doc)
        rc = validator.main([
            "--cpp", cpp_path, "--rust", rust_path,
            "--ledger", ledger_path, "--intentional", intentional_path,
        ])
        self.assertEqual(rc, 1)

    def test_unknown_intentional_id_caught(self):
        doc = build_seed.build_doc(CPP_FIXTURE, RUST_FIXTURE)
        r = doc["symbols"][0]
        r["notes"] = "intentional:nonexistent-id"
        cpp_path, rust_path, ledger_path, intentional_path = self._write_inputs(doc)
        rc = validator.main([
            "--cpp", cpp_path, "--rust", rust_path,
            "--ledger", ledger_path, "--intentional", intentional_path,
        ])
        self.assertEqual(rc, 1)

    def test_bad_status_caught(self):
        doc = build_seed.build_doc(CPP_FIXTURE, RUST_FIXTURE)
        doc["symbols"][0]["status"] = "DONE"  # legacy value, no longer allowed
        cpp_path, rust_path, ledger_path, intentional_path = self._write_inputs(doc)
        rc = validator.main([
            "--cpp", cpp_path, "--rust", rust_path,
            "--ledger", ledger_path, "--intentional", intentional_path,
        ])
        self.assertEqual(rc, 1)


class TestRollup(unittest.TestCase):
    def test_all_uncertain_yields_pending(self):
        doc = build_seed.build_doc(CPP_FIXTURE, RUST_FIXTURE)
        rolled = rollup.build_rollup(doc["symbols"])
        for r in rolled:
            self.assertEqual(r["status"], "PENDING")

    def test_all_migrated_yields_done(self):
        doc = build_seed.build_doc(CPP_FIXTURE, RUST_FIXTURE)
        for r in doc["symbols"]:
            r["status"] = "migrated"
        rolled = rollup.build_rollup(doc["symbols"])
        for r in rolled:
            self.assertEqual(r["status"], "DONE")

    def test_one_partial_yields_partial(self):
        doc = build_seed.build_doc(CPP_FIXTURE, RUST_FIXTURE)
        for r in doc["symbols"]:
            r["status"] = "migrated"
        # mark position rows as partial
        for r in doc["symbols"]:
            if r["cpp"]["file"] == "src/position.cpp":
                r["status"] = "partial"
        rolled = rollup.build_rollup(doc["symbols"])
        by_cpp = {r["cpp"]: r["status"] for r in rolled}
        self.assertEqual(by_cpp["src/position.cpp"], "PARTIAL")
        self.assertEqual(by_cpp["src/tools.cpp"], "DONE")


if __name__ == "__main__":
    unittest.main()
