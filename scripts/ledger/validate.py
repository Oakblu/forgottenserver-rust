#!/usr/bin/env python3
"""Validate MIGRATION_LEDGER.yml against the schema in design.md.

Exits 0 on success, prints a structured report and exits non-zero on
first violation class encountered. See design.md §3..§6, §11.

Usage:
    python3 scripts/ledger/validate.py [--ledger PATH] [--cpp PATH] [--rust PATH]
"""
from __future__ import annotations

import argparse
import json
import os
import sys
from typing import Any

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import ledger_io  # noqa: E402

DEFAULT_ROOT = os.path.abspath(os.path.join(HERE, os.pardir, os.pardir))

ALLOWED_STATUS = {
    "migrated",
    "migrated_with_changes",
    "partial",
    "missing",
    "intentionally_removed",
    "obsolete",
    "uncertain",
    "needs_review",
    "output_verified",
}

ALLOWED_MIGRATION_TYPE = {
    "direct",
    "renamed",
    "split",
    "merged",
    "redesigned",
    "removed",
    "unknown",
}

ALLOWED_CONFIDENCE = {"high", "medium", "low"}

# status -> allowed migration_types (per design.md §4)
STATUS_TO_TYPES = {
    "migrated": {"direct", "renamed", "split", "merged", "redesigned"},
    "migrated_with_changes": {"direct", "renamed", "split", "merged", "redesigned"},
    "partial": {"split", "merged", "redesigned"},
    "missing": {"removed"},
    "intentionally_removed": {"removed"},
    "obsolete": {"removed"},
    "uncertain": {"unknown"},
    "needs_review": ALLOWED_MIGRATION_TYPE - {"unknown"},
    "output_verified": {"direct", "renamed", "split", "merged", "redesigned"},
}

# status requiring at least one rust entry
STATUS_REQUIRES_RUST = {
    "migrated",
    "migrated_with_changes",
    "partial",
    "output_verified",
}

# status forbidden with confidence: low
STATUS_FORBIDS_LOW = {"migrated", "migrated_with_changes", "output_verified"}

STATUS_FORBIDS_HIGH = {"uncertain", "needs_review"}

# status requires tests_required to be non-empty.
# `partial` / `missing` mean the auditor knows what's broken, so they
# must enumerate the tests. `uncertain` / `needs_review` mean the
# auditor cannot yet say what would resolve it, so `tests_required`
# stays optional until the row is promoted.
STATUS_NEEDS_TESTS = {"partial", "missing"}

# status requires evidence with a `test:` bullet
STATUS_NEEDS_TEST_BULLET = {"migrated", "migrated_with_changes", "output_verified"}

EVIDENCE_PREFIXES = ("cpp:", "rust:", "test:", "manifest:cpp:", "manifest:rust:", "intentional:")

ROW_KEYS = [
    "id",
    "cpp",
    "rust",
    "status",
    "migration_type",
    "confidence",
    "evidence",
    "semantic_differences",
    "risks",
    "tests_required",
    "differential_scenarios",
    "notes",
]


class Violation:
    __slots__ = ("row_id", "kind", "detail")

    def __init__(self, row_id: str, kind: str, detail: str) -> None:
        self.row_id = row_id
        self.kind = kind
        self.detail = detail

    def __str__(self) -> str:
        return f"[{self.kind}] {self.row_id}: {self.detail}"


def _check_row(row: dict[str, Any], cpp_index: dict[tuple[str, str], dict],
               rust_index: dict[tuple[str, str, str], dict],
               intentional_ids: set[str]) -> list[Violation]:
    violations: list[Violation] = []
    rid = row.get("id", "<missing-id>")

    # Required keys present
    for k in ROW_KEYS:
        if k not in row:
            violations.append(Violation(rid, "missing-key", k))

    # Enum values
    status = row.get("status")
    if status not in ALLOWED_STATUS:
        violations.append(Violation(rid, "bad-status", f"{status!r} not in allowed set"))
    mtype = row.get("migration_type")
    if mtype not in ALLOWED_MIGRATION_TYPE:
        violations.append(Violation(rid, "bad-migration-type", f"{mtype!r} not in allowed set"))
    conf = row.get("confidence")
    if conf not in ALLOWED_CONFIDENCE:
        violations.append(Violation(rid, "bad-confidence", f"{conf!r} not in allowed set"))

    if violations:
        return violations  # don't compound further checks on a broken row

    # status ↔ migration_type compatibility
    if mtype not in STATUS_TO_TYPES[status]:
        violations.append(Violation(
            rid, "status-migration-type-mismatch",
            f"status={status} with migration_type={mtype}",
        ))

    # confidence rules
    if status in STATUS_FORBIDS_LOW and conf == "low":
        violations.append(Violation(rid, "confidence-too-low", f"status={status} requires >= medium"))
    if status in STATUS_FORBIDS_HIGH and conf == "high":
        violations.append(Violation(rid, "confidence-too-high", f"status={status} cannot be high"))

    cpp = row["cpp"]
    for k in ("file", "symbol", "signature", "kind", "body_hash"):
        if k not in cpp:
            violations.append(Violation(rid, "cpp-missing-field", k))

    # cpp.* must match the manifest. The manifest may contain multiple
    # entries per (file, qualified_name) for overloaded methods, so the
    # row must match one of them on body_hash.
    cpp_key = (cpp.get("file", ""), cpp.get("symbol", ""))
    overloads = cpp_index.get(cpp_key, [])
    if not overloads:
        violations.append(Violation(rid, "cpp-not-in-manifest", f"{cpp_key} unknown to cpp_symbol_manifest.json"))
    else:
        manifest_hashes = {m.get("body_hash", "") for m in overloads}
        if cpp.get("body_hash") and cpp["body_hash"] not in manifest_hashes:
            violations.append(Violation(
                rid, "cpp-body-hash-drift",
                f"manifest={sorted(manifest_hashes)} ledger={cpp['body_hash']}",
            ))

    # rust[*] must match the manifest (same overload-bucket logic).
    rust = row.get("rust") or []
    if status in STATUS_REQUIRES_RUST and not rust:
        violations.append(Violation(rid, "rust-required", f"status={status} requires >= 1 rust entry"))
    for i, r in enumerate(rust):
        rkey = (r.get("file", ""), r.get("symbol", ""), r.get("kind", ""))
        rm_list = rust_index.get(rkey, [])
        if not rm_list:
            violations.append(Violation(rid, "rust-not-in-manifest", f"rust[{i}] {rkey} unknown to rust_symbol_manifest.json"))
            continue
        manifest_hashes = {m.get("body_hash", "") for m in rm_list}
        if r.get("body_hash") and r["body_hash"] not in manifest_hashes:
            violations.append(Violation(
                rid, "rust-body-hash-drift",
                f"rust[{i}] manifest={sorted(manifest_hashes)} ledger={r['body_hash']}",
            ))

    # Evidence rules
    evidence = row.get("evidence") or []
    for ev in evidence:
        if not isinstance(ev, str):
            violations.append(Violation(rid, "evidence-not-string", repr(ev)))
            continue
        if not ev.startswith(EVIDENCE_PREFIXES):
            violations.append(Violation(rid, "evidence-bad-prefix", ev[:80]))
        if ev.lower().startswith(("name match", "name-match")):
            violations.append(Violation(rid, "evidence-name-only", ev[:80]))

    if status in STATUS_NEEDS_TEST_BULLET:
        if not any(isinstance(e, str) and e.startswith("test:") for e in evidence):
            violations.append(Violation(rid, "evidence-missing-test", f"status={status} requires a test: bullet"))

    if status in STATUS_NEEDS_TESTS and not row.get("tests_required"):
        violations.append(Violation(rid, "tests-required-empty", f"status={status} must list missing tests"))

    if status == "migrated_with_changes" and not row.get("semantic_differences"):
        violations.append(Violation(rid, "diffs-required", "migrated_with_changes needs semantic_differences"))

    if status == "intentionally_removed":
        # notes or semantic_differences must reference intentional:<id>
        text_blobs = [row.get("notes", "")] + list(row.get("semantic_differences") or [])
        if not any(("intentional:" in s) for s in text_blobs if isinstance(s, str)):
            violations.append(Violation(rid, "intentional-link-missing", "needs intentional:<id> reference"))

    # any intentional:<id> references must exist
    for s in evidence + row.get("semantic_differences", []) + [row.get("notes", "")]:
        if not isinstance(s, str):
            continue
        if "intentional:" in s:
            ref = s.split("intentional:", 1)[1].split()[0].strip(",")
            if ref and ref not in intentional_ids:
                violations.append(Violation(rid, "intentional-id-unknown", ref))

    if status == "output_verified":
        scenarios = row.get("differential_scenarios") or []
        if not any(isinstance(sc, dict) and sc.get("verified") is True for sc in scenarios):
            violations.append(Violation(rid, "output-verified-no-scenario", "needs >=1 differential_scenario with verified: true"))

    return violations


def _load_intentional_ids(path: str) -> set[str]:
    if not os.path.exists(path):
        return set()
    out: set[str] = set()
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.lstrip()
            if line.startswith("- id:"):
                _, _, rest = line.partition(":")
                out.add(rest.strip())
    return out


def _load_manifests(cpp_path: str, rust_path: str):
    with open(cpp_path, "r", encoding="utf-8") as f:
        cpp_symbols = json.load(f)
    with open(rust_path, "r", encoding="utf-8") as f:
        rust_symbols = json.load(f)
    cpp_index: dict[tuple[str, str], list[dict]] = {}
    for s in cpp_symbols:
        cpp_index.setdefault((s["file"], s["qualified_name"]), []).append(s)
    rust_index: dict[tuple[str, str, str], list[dict]] = {}
    for s in rust_symbols:
        rust_index.setdefault((s["file"], s["qualified_name"], s["kind"]), []).append(s)
    return cpp_symbols, rust_symbols, cpp_index, rust_index


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Validate MIGRATION_LEDGER.yml")
    parser.add_argument("--ledger", default=os.path.join(DEFAULT_ROOT, "MIGRATION_LEDGER.yml"))
    parser.add_argument("--cpp", default=os.path.join(DEFAULT_ROOT, "cpp_symbol_manifest.json"))
    parser.add_argument("--rust", default=os.path.join(DEFAULT_ROOT, "rust_symbol_manifest.json"))
    parser.add_argument("--intentional", default=os.path.join(DEFAULT_ROOT, "intentional_differences.yml"))
    parser.add_argument("--max-report", type=int, default=50, help="cap reported violations")
    args = parser.parse_args(argv)

    with open(args.ledger, "r", encoding="utf-8") as f:
        doc = ledger_io.load(f.read())

    if doc.get("schema") != "per-symbol":
        print(f"ERROR: {args.ledger} is not schema=per-symbol (got {doc.get('schema')!r})")
        return 2

    cpp_symbols, _rust_symbols, cpp_index, rust_index = _load_manifests(args.cpp, args.rust)
    intentional_ids = _load_intentional_ids(args.intentional)

    rows = doc.get("symbols") or []
    by_id: dict[str, dict] = {}
    by_body: dict[tuple[str, str, str], list[dict]] = {}
    violations: list[Violation] = []

    for row in rows:
        rid = row.get("id", "<missing>")
        if rid in by_id:
            violations.append(Violation(rid, "duplicate-id", "appears more than once"))
        by_id[rid] = row
        cpp = row.get("cpp") or {}
        bkey = (cpp.get("file", ""), cpp.get("symbol", ""), cpp.get("body_hash", ""))
        by_body.setdefault(bkey, []).append(row)
        violations.extend(_check_row(row, cpp_index, rust_index, intentional_ids))

    # Every manifest entry must appear at least once. Overloaded methods
    # share (file, qualified_name) but have distinct body_hashes, so the
    # coverage key is the triple.
    for sym in cpp_symbols:
        bkey = (sym["file"], sym["qualified_name"], sym.get("body_hash", ""))
        if bkey not in by_body:
            violations.append(Violation(
                f"<unmapped:{sym['file']}::{sym['qualified_name']}:{sym.get('body_hash','')}>",
                "manifest-symbol-missing",
                "no ledger row covers this C++ symbol+body_hash",
            ))

    by_kind: dict[str, int] = {}
    for v in violations:
        by_kind[v.kind] = by_kind.get(v.kind, 0) + 1

    print(f"Ledger rows: {len(rows)}")
    print(f"CPP manifest symbols: {len(cpp_symbols)}")
    print(f"Distinct CPP (file, symbol, body_hash) covered: {len(by_body)}")
    print(f"Violations: {len(violations)}")
    for kind in sorted(by_kind):
        print(f"  {kind}: {by_kind[kind]}")
    if violations:
        print("--- first violations ---")
        for v in violations[: args.max_report]:
            print(v)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
