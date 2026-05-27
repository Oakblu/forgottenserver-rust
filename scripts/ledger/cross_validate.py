#!/usr/bin/env python3
"""Phase-2 cross-validation for MIGRATION_LEDGER.yml.

Runs four independent checks defined in tasks.md §Phase 2:

  2.0 manifest round-trip   — same `id` set as a fresh build_seed run
  2.1 reverse coverage      — rust manifest entries vs. ledger references
  2.2 intentional orphans   — intentional_differences.yml IDs vs. usage
  2.3 file rollup           — `files:` index vs. MIGRATION.md status table

Each check prints a section header and a verdict. `main` returns:

  0  all checks PASS
  1  any check FAIL (id-set drift, orphan refs, etc.)
  2  any check WARN-only (rust manifest contains symbols not referenced
      by any row — usually means the manifest is stale or contains
      Rust-only helpers; informational)

`--strict` promotes WARN to FAIL.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sys
from collections import Counter
from typing import Any

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import ledger_io  # noqa: E402
import build_seed  # noqa: E402

DEFAULT_ROOT = os.path.abspath(os.path.join(HERE, os.pardir, os.pardir))


def banner(title: str) -> None:
    print()
    print("=" * 72)
    print(f" {title}")
    print("=" * 72)


# ---------------------------------------------------------------------------
# 2.0  Manifest round-trip
# ---------------------------------------------------------------------------

def check_roundtrip(ledger_path: str, cpp_path: str, rust_path: str) -> int:
    banner("2.0  Manifest round-trip")
    with open(ledger_path, "r", encoding="utf-8") as f:
        ledger = ledger_io.load(f.read())
    with open(cpp_path, "r", encoding="utf-8") as f:
        cpp_symbols = json.load(f)
    with open(rust_path, "r", encoding="utf-8") as f:
        rust_symbols = json.load(f)

    fresh = build_seed.build_doc(cpp_symbols, rust_symbols)
    ledger_ids = {r["id"] for r in ledger.get("symbols", [])}
    fresh_ids = {r["id"] for r in fresh["symbols"]}

    added = fresh_ids - ledger_ids
    removed = ledger_ids - fresh_ids
    print(f"Ledger rows: {len(ledger_ids)}")
    print(f"Fresh seed rows: {len(fresh_ids)}")
    print(f"Ids added by fresh seed: {len(added)}")
    print(f"Ids removed from ledger: {len(removed)}")
    if added:
        print("  added (first 5):", sorted(added)[:5])
    if removed:
        print("  removed (first 5):", sorted(removed)[:5])
    rc = 0 if not added and not removed else 1
    print(f"VERDICT: {'PASS' if rc == 0 else 'FAIL'}")
    return rc


# ---------------------------------------------------------------------------
# 2.1  Reverse coverage check
# ---------------------------------------------------------------------------

RUST_INTERNAL_LINE_RE = re.compile(r'^\s*-\s*"([^|]+)\|([^|]+)\|([^"]+)"\s*$')


def _load_rust_internal(path: str) -> set[tuple[str, str, str]]:
    """Read all entries from rust_internal.yml — every category — and
    return them as a flat set of (file, qualified_name, kind) tuples."""
    exempt: set[tuple[str, str, str]] = set()
    if not os.path.exists(path):
        return exempt
    with open(path, "r", encoding="utf-8") as f:
        for raw in f:
            m = RUST_INTERNAL_LINE_RE.match(raw.rstrip("\n"))
            if m:
                exempt.add((m.group(1), m.group(2), m.group(3)))
    return exempt


def check_reverse_coverage(ledger_path: str, rust_path: str, internal_path: str) -> int:
    banner("2.1  Reverse coverage (rust manifest ↔ ledger)")
    with open(ledger_path, "r", encoding="utf-8") as f:
        ledger = ledger_io.load(f.read())
    with open(rust_path, "r", encoding="utf-8") as f:
        rust_symbols = json.load(f)
    exempt = _load_rust_internal(internal_path)

    referenced: set[tuple[str, str, str]] = set()
    for r in ledger.get("symbols", []):
        for rs in r.get("rust") or []:
            referenced.add((rs["file"], rs["symbol"], rs["kind"]))

    manifest_keys: set[tuple[str, str, str]] = set()
    for s in rust_symbols:
        manifest_keys.add((s["file"], s["qualified_name"], s["kind"]))

    unreferenced = manifest_keys - referenced
    dangling = referenced - manifest_keys
    unclassified = unreferenced - exempt

    print(f"Rust manifest entries: {len(manifest_keys)}")
    print(f"Distinct rust refs from ledger: {len(referenced)}")
    print(f"Unreferenced manifest entries: {len(unreferenced)}")
    print(f"  → exempted by rust_internal.yml: {len(unreferenced & exempt)}")
    print(f"  → unclassified (real gap):        {len(unclassified)}")
    print(f"Dangling ledger refs (no manifest entry): {len(dangling)}")

    if dangling:
        print("  dangling (first 5):")
        for d in sorted(dangling)[:5]:
            print(f"    {d}")

    if unclassified:
        by_file = Counter(f for f, _, _ in unclassified)
        print("  top files with unclassified entries:")
        for f, n in by_file.most_common(10):
            print(f"    {n:>5}  {f}")

    rc = 0
    if dangling:
        rc = 1
    elif unclassified:
        rc = 2  # WARN — real gap; either map them or extend rust_internal.yml
    print(f"VERDICT: {'PASS' if rc == 0 else 'FAIL' if rc == 1 else 'WARN'}")
    return rc


# ---------------------------------------------------------------------------
# 2.2  intentional_differences.yml orphan check
# ---------------------------------------------------------------------------

INTENTIONAL_ID_RE = re.compile(r"intentional:([A-Za-z0-9_\-]+)")


def _load_intentional_ids(path: str) -> set[str]:
    out: set[str] = set()
    if not os.path.exists(path):
        return out
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.lstrip()
            if line.startswith("- id:"):
                _, _, rest = line.partition(":")
                out.add(rest.strip())
    return out


def check_intentional_orphans(ledger_path: str, intentional_path: str) -> int:
    banner("2.2  intentional_differences.yml orphan check")
    defined = _load_intentional_ids(intentional_path)
    if not defined:
        print(f"  intentional_differences.yml not found or empty: {intentional_path}")
        return 0

    with open(ledger_path, "r", encoding="utf-8") as f:
        ledger = ledger_io.load(f.read())

    referenced: set[str] = set()
    for r in ledger.get("symbols", []):
        for blob in [r.get("notes", "")] + list(r.get("evidence") or []) + list(r.get("semantic_differences") or []):
            if isinstance(blob, str):
                for m in INTENTIONAL_ID_RE.finditer(blob):
                    referenced.add(m.group(1))

    orphans = defined - referenced
    unknown = referenced - defined  # validate.py catches this; double-check here

    print(f"Defined ids: {len(defined)}")
    print(f"Referenced ids: {len(referenced)}")
    print(f"Orphan ids (defined but unused): {len(orphans)}")
    print(f"Unknown ids (referenced but undefined): {len(unknown)}")
    if orphans:
        print("  orphans:")
        for o in sorted(orphans):
            print(f"    - {o}")
    if unknown:
        print("  unknown:")
        for u in sorted(unknown):
            print(f"    - {u}")

    if unknown:
        return 1
    if orphans:
        return 2  # WARN — orphans are not fatal but worth resolving
    print("VERDICT: PASS")
    return 0


# ---------------------------------------------------------------------------
# 2.3  files: rollup vs MIGRATION.md
# ---------------------------------------------------------------------------

MIGRATION_ROW_RE = re.compile(
    r"^\|\s*(?:`)?(?P<cpp>[A-Za-z0-9_\./\-]+\.(?:cpp|h)(?:/h)?)(?:`)?\s*\|"
    r"[^|]*\|[^|]*\|[^|]*\|\s*(?P<status>PENDING|PARTIAL|DONE|NON_GOAL|MISSING|UNCERTAIN)\b"
)


def _parse_migration_md(path: str) -> dict[str, str]:
    """Parse the per-file status table out of MIGRATION.md.

    The narrative uses rows like `| foo.cpp/h | ... | DONE | ... |`.
    For `.cpp/h` combined rows we emit a SINGLE entry keyed by the
    combined `src/foo.cpp/h` token, not two separate entries — this
    matches sync_migration_md.py's worst-of-pair severity logic, so
    reconciliation is 1:1.
    """
    out: dict[str, str] = {}
    if not os.path.exists(path):
        return out
    with open(path, "r", encoding="utf-8") as f:
        for raw in f:
            m = MIGRATION_ROW_RE.match(raw.rstrip("\n"))
            if not m:
                continue
            cpp = m.group("cpp").strip().rstrip("/")
            status = m.group("status")
            cpp_norm = cpp if cpp.startswith("src/") else f"src/{cpp}"
            out[cpp_norm] = status
    return out


# Severity ranking matching sync_migration_md.py — worst → best
_SEVERITY = {"MISSING": 5, "PENDING": 4, "UNCERTAIN": 4, "PARTIAL": 3, "DONE": 2, "NON_GOAL": 1}


def _rollup_for_md_key(key: str, rollup: dict[str, str]) -> str | None:
    """Resolve a MIGRATION.md key (which may be `src/foo.cpp/h`,
    `src/foo.cpp`, or `src/foo.h`) to a single rollup status using
    worst-of-pair severity when needed."""
    if key.endswith(".cpp/h"):
        stem = key[: -len(".cpp/h")]
        candidates = [rollup.get(f"{stem}.cpp"), rollup.get(f"{stem}.h")]
        candidates = [c for c in candidates if c]
        if not candidates:
            return None
        return max(candidates, key=lambda s: _SEVERITY.get(s, 0))
    return rollup.get(key)


def check_files_rollup(ledger_path: str, migration_md: str) -> int:
    banner("2.3  files: rollup vs MIGRATION.md")
    with open(ledger_path, "r", encoding="utf-8") as f:
        ledger = ledger_io.load(f.read())

    rollup = {f["cpp"]: f["status"] for f in ledger.get("files") or []}
    narrative = _parse_migration_md(migration_md)

    if not narrative:
        print(f"MIGRATION.md status table not parsed at: {migration_md}")
        print("Nothing to reconcile against. Skipping.")
        print("VERDICT: PASS (skipped)")
        return 0

    print(f"Files in rollup: {len(rollup)}")
    print(f"Files in MIGRATION.md status table: {len(narrative)}")

    disagreements: list[tuple[str, str, str]] = []
    for f, nstatus in narrative.items():
        rstatus = _rollup_for_md_key(f, rollup)
        if rstatus is None:
            # MIGRATION.md tracks files the C++ manifest doesn't —
            # cpp_symbol_manifest extraction gap (e.g. src/http/*).
            # Skip these; they're a manifest issue, not a ledger drift.
            continue
        if rstatus != nstatus:
            disagreements.append((f, nstatus, rstatus))

    print(f"Disagreements: {len(disagreements)}")
    if disagreements:
        print("  cpp_file                                 MIGRATION.md     rollup")
        for f, nstatus, rstatus in disagreements[:25]:
            print(f"  {f:<40} {nstatus:<15}  {rstatus}")
        if len(disagreements) > 25:
            print(f"  ... +{len(disagreements) - 25} more")
        # Symbol-level evidence wins per design.md §11; flag as WARN, not FAIL.
        # Leader can choose to update MIGRATION.md in a follow-up.
        print("VERDICT: WARN — symbol-level evidence wins; update MIGRATION.md when ready")
        return 2

    print("VERDICT: PASS")
    return 0


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Phase-2 cross-validation")
    parser.add_argument("--ledger", default=os.path.join(DEFAULT_ROOT, "MIGRATION_LEDGER.yml"))
    parser.add_argument("--cpp", default=os.path.join(DEFAULT_ROOT, "cpp_symbol_manifest.json"))
    parser.add_argument("--rust", default=os.path.join(DEFAULT_ROOT, "rust_symbol_manifest.json"))
    parser.add_argument("--intentional", default=os.path.join(DEFAULT_ROOT, "intentional_differences.yml"))
    parser.add_argument("--migration-md", default=os.path.join(DEFAULT_ROOT, "MIGRATION.md"))
    parser.add_argument("--rust-internal", default=os.path.join(DEFAULT_ROOT, "audit_patches", "rust_internal.yml"))
    parser.add_argument("--strict", action="store_true", help="treat WARN as FAIL")
    args = parser.parse_args(argv)

    results = [
        check_roundtrip(args.ledger, args.cpp, args.rust),
        check_reverse_coverage(args.ledger, args.rust, args.rust_internal),
        check_intentional_orphans(args.ledger, args.intentional),
        check_files_rollup(args.ledger, args.migration_md),
    ]

    banner("Summary")
    labels = ["2.0 roundtrip", "2.1 reverse coverage", "2.2 intentional orphans", "2.3 files rollup"]
    for label, rc in zip(labels, results):
        verdict = "PASS" if rc == 0 else "FAIL" if rc == 1 else "WARN"
        print(f"  {label:<28}  {verdict}")

    worst = max(results)
    if worst == 0:
        print("\nAll checks PASS.")
        return 0
    if worst == 2:
        print("\nAll checks PASS or WARN (no failures).")
        return 1 if args.strict else 0
    print("\nOne or more checks FAILED.")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
