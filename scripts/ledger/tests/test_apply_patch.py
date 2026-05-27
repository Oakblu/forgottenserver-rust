"""Tests for apply_patch.py."""
from __future__ import annotations

import json
import os
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))

import apply_patch  # noqa: E402
import build_seed  # noqa: E402
import ledger_io  # noqa: E402

CPP_FIXTURE = [
    {
        "file": "src/x.cpp",
        "kind": "method",
        "qualified_name": "X::foo",
        "signature": "void X::foo()",
        "body_hash": "aa1111",
    },
    {
        "file": "src/x.cpp",
        "kind": "method",
        "qualified_name": "X::bar",
        "signature": "void X::bar()",
        "body_hash": "bb2222",
    },
    {
        "file": "src/y.cpp",
        "kind": "free_function",
        "qualified_name": "helper",
        "signature": "void helper()",
        "body_hash": "cc3333",
    },
]

RUST_FIXTURE = [
    {
        "file": "crates/x.rs",
        "kind": "impl_method",
        "qualified_name": "x::X::foo",
        "signature": "pub fn foo(&self)",
        "body_hash": "dd4444",
    },
]


def _seed_ledger(path: str) -> dict:
    doc = build_seed.build_doc(CPP_FIXTURE, RUST_FIXTURE)
    with open(path, "w") as f:
        f.write(ledger_io.dump(doc))
    return doc


class TestApplyPatch(unittest.TestCase):
    def test_in_scope_update_works(self):
        with tempfile.TemporaryDirectory() as tmp:
            ledger_path = os.path.join(tmp, "MIGRATION_LEDGER.yml")
            seed = _seed_ledger(ledger_path)
            target_id = next(r["id"] for r in seed["symbols"] if r["cpp"]["symbol"] == "X::foo")
            patch_path = os.path.join(tmp, "layer-0.json")
            patch = {
                "layer": 0,
                "scope_files": ["src/x.cpp"],
                "rows": [{
                    "id": target_id,
                    "status": "migrated",
                    "migration_type": "renamed",
                    "confidence": "high",
                    "rust": [{
                        "file": "crates/x.rs",
                        "symbol": "x::X::foo",
                        "signature": "pub fn foo(&self)",
                        "kind": "impl_method",
                        "body_hash": "dd4444",
                    }],
                    "evidence": [
                        "cpp:src/x.cpp:1-3 — foo body",
                        "rust:crates/x.rs:1-3 — foo body",
                        "test:x::tests::test_foo — asserts foo behavior",
                    ],
                    "semantic_differences": [],
                    "risks": [],
                    "tests_required": [],
                    "differential_scenarios": [],
                    "notes": "",
                }],
            }
            with open(patch_path, "w") as f:
                json.dump(patch, f)
            rc = apply_patch.main([
                "--ledger", ledger_path,
                "--skip-validate",
                patch_path,
            ])
            self.assertEqual(rc, 0)
            with open(ledger_path) as f:
                merged = ledger_io.load(f.read())
            row = next(r for r in merged["symbols"] if r["id"] == target_id)
            self.assertEqual(row["status"], "migrated")
            self.assertEqual(row["confidence"], "high")

    def test_out_of_scope_row_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            ledger_path = os.path.join(tmp, "MIGRATION_LEDGER.yml")
            seed = _seed_ledger(ledger_path)
            # Patch claims scope src/x.cpp but tries to edit y.cpp row.
            target_id = next(r["id"] for r in seed["symbols"] if r["cpp"]["file"] == "src/y.cpp")
            patch_path = os.path.join(tmp, "layer-0.json")
            patch = {
                "layer": 0,
                "scope_files": ["src/x.cpp"],
                "rows": [{"id": target_id, "notes": "tampering"}],
            }
            with open(patch_path, "w") as f:
                json.dump(patch, f)
            rc = apply_patch.main(["--ledger", ledger_path, "--skip-validate", patch_path])
            self.assertEqual(rc, 1)
            # Ledger unchanged.
            with open(ledger_path) as f:
                after = ledger_io.load(f.read())
            row = next(r for r in after["symbols"] if r["id"] == target_id)
            self.assertEqual(row["notes"], "")

    def test_cpp_block_in_patch_is_ignored(self):
        with tempfile.TemporaryDirectory() as tmp:
            ledger_path = os.path.join(tmp, "MIGRATION_LEDGER.yml")
            seed = _seed_ledger(ledger_path)
            target_id = next(r["id"] for r in seed["symbols"] if r["cpp"]["symbol"] == "X::foo")
            patch_path = os.path.join(tmp, "layer-0.json")
            patch = {
                "layer": 0,
                "scope_files": ["src/x.cpp"],
                "rows": [{
                    "id": target_id,
                    "cpp": {"file": "src/HACKED.cpp", "symbol": "HACKED"},  # ignored
                    "notes": "ok",
                }],
            }
            with open(patch_path, "w") as f:
                json.dump(patch, f)
            rc = apply_patch.main(["--ledger", ledger_path, "--skip-validate", patch_path])
            self.assertEqual(rc, 0)
            with open(ledger_path) as f:
                merged = ledger_io.load(f.read())
            row = next(r for r in merged["symbols"] if r["id"] == target_id)
            self.assertEqual(row["cpp"]["file"], "src/x.cpp")
            self.assertEqual(row["notes"], "ok")

    def test_unknown_id_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            ledger_path = os.path.join(tmp, "MIGRATION_LEDGER.yml")
            _seed_ledger(ledger_path)
            patch_path = os.path.join(tmp, "layer-0.json")
            patch = {
                "layer": 0,
                "scope_files": ["src/x.cpp"],
                "rows": [{"id": "totally-made-up", "notes": "x"}],
            }
            with open(patch_path, "w") as f:
                json.dump(patch, f)
            rc = apply_patch.main(["--ledger", ledger_path, "--skip-validate", patch_path])
            self.assertEqual(rc, 1)


if __name__ == "__main__":
    unittest.main()
