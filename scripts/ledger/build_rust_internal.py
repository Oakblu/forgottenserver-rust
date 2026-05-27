#!/usr/bin/env python3
"""Generate audit_patches/rust_internal.yml.

Auto-classifies Rust manifest entries that have no expected C++ origin
into named categories. cross_validate.py §2.1 consults this file to
distinguish "Rust scaffolding" from "unreferenced audit gap".

Categories:

  tests/                  qualified_name contains ::tests:: or ends ::tests
  scaffolding/            kinds with no direct C++ analogue:
                            error_type, type_alias, trait, trait_method,
                            module, macro, scheduler_event,
                            database_mapping, associated_type, impl_block
  parent_covered/         children whose parent symbol is referenced by
                          some ledger row:
                            enum_variant whose parent enum is referenced
                            field whose parent struct is referenced
                            associated_function whose impl-block parent
                              struct is referenced

Anything left unreferenced after exemption is a real audit gap.
"""
from __future__ import annotations

import json
import os
import sys
from collections import defaultdict
from typing import Any

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import ledger_io  # noqa: E402

ROOT = os.path.abspath(os.path.join(HERE, os.pardir, os.pardir))

# Kinds that are Rust scaffolding by nature.
SCAFFOLD_KINDS = {
    "error_type",
    "type_alias",
    "trait",
    "trait_method",
    "module",
    "macro",
    "scheduler_event",
    "database_mapping",
    "associated_type",
    "impl_block",
    "static",
}

# (child_kind, parent_kind) — child is covered when parent symbol
# (file, parent_qualified_name) appears under any ledger row's rust[].
PARENT_RULES = {
    "enum_variant": ("enum",),
    "field": ("struct",),
    "associated_const": ("struct", "enum"),
    "associated_function": ("struct", "enum"),
    "trait_impl_method": ("struct", "enum"),
    "impl_method": ("struct", "enum"),
}


def _parent_qn(qn: str) -> str:
    """Drop the last `::seg` from a qualified name."""
    if "::" not in qn:
        return ""
    return qn.rsplit("::", 1)[0]


LIB_FILE_BASENAMES = {"lib.rs", "mod.rs", "error.rs"}

# Rust files known to be 100% Rust-only architectural additions (handle
# layers, registries, glue) that have NO direct C++ symbol-for-symbol
# counterpart but are governed by recorded intentional differences.
# Items in these files are auto-exempted with their tied intentional id.
RUST_ONLY_ARCHITECTURE = {
    "crates/game/src/party.rs": "handles-instead-of-raw-pointers",
    "crates/scripting/src/script.rs": "arc-only-where-shared-lifetime-is-real",
}


def _cpp_files_with_rows(cpp_symbols: list[dict[str, Any]]) -> set[str]:
    return {s["file"] for s in cpp_symbols}


def _rust_file_to_cpp_file(rust_file: str) -> str:
    """Guess the matching C++ file path from a Rust file path.

    Heuristic: take the basename without extension, prepend src/, try
    both `.cpp` and `.h`. e.g. crates/common/src/base64.rs → src/base64.cpp
    """
    base = os.path.basename(rust_file).rsplit(".", 1)[0]
    return base


def classify(rust_symbols: list[dict[str, Any]],
             cpp_symbols: list[dict[str, Any]],
             referenced: set[tuple[str, str, str]]):
    """Return (categories, unclassified) where categories is a dict
    category-name → list of {file, qualified_name, kind} entries."""

    by_qn: dict[tuple[str, str], list[dict]] = defaultdict(list)
    for s in rust_symbols:
        by_qn[(s["file"], s["qualified_name"])].append(s)

    referenced_qn: set[tuple[str, str]] = set()
    referenced_files: set[str] = set()
    for f, qn, _kind in referenced:
        referenced_qn.add((f, qn))
        referenced_files.add(f)

    # Files whose corresponding C++ source has no FUNCTIONAL manifest
    # entries. Only an include-guard macro / typedef / friend
    # declaration is NOT a real audit target — counts as a manifest
    # extraction gap on the C++ side.
    NON_FUNCTIONAL_KINDS = {
        "macro", "friend_declaration", "typedef", "using_alias", "namespace",
    }
    cpp_basenames_functional: dict[str, int] = defaultdict(int)
    for s in cpp_symbols:
        f = s["file"]
        if f.startswith("src/"):
            f = f[4:]
        if f.endswith((".cpp", ".h")):
            f = f.rsplit(".", 1)[0]
        if s.get("kind") not in NON_FUNCTIONAL_KINDS:
            cpp_basenames_functional[f] += 1
    cpp_basenames_with_rows = {f for f, n in cpp_basenames_functional.items() if n > 0}

    categories: dict[str, list[dict]] = {
        "tests": [],
        "scaffolding": [],
        "parent_covered": [],
        "file_audited": [],
        "lib_files": [],
        "manifest_gap": [],
        "rust_only_architecture": [],
    }
    unclassified: list[dict] = []

    for s in rust_symbols:
        key = (s["file"], s["qualified_name"], s["kind"])
        if key in referenced:
            continue
        qn = s["qualified_name"]
        entry = {"file": s["file"], "symbol": qn, "kind": s["kind"]}

        # 1. Test modules
        if "::tests::" in qn or qn.endswith("::tests"):
            categories["tests"].append(entry)
            continue
        # 2. Scaffolding kinds
        if s["kind"] in SCAFFOLD_KINDS:
            categories["scaffolding"].append(entry)
            continue
        # 3. Lib / mod / error infrastructure files (always Rust-only)
        basename = os.path.basename(s["file"])
        if basename in LIB_FILE_BASENAMES:
            categories["lib_files"].append(entry)
            continue
        # 3b. Hand-declared Rust-only architectural files
        if s["file"] in RUST_ONLY_ARCHITECTURE:
            entry["intentional_id"] = RUST_ONLY_ARCHITECTURE[s["file"]]
            categories["rust_only_architecture"].append(entry)
            continue
        # 4. Parent-of-symbol is referenced (e.g. enum_variant whose enum
        #    is referenced).
        rule = PARENT_RULES.get(s["kind"])
        if rule:
            parent = _parent_qn(qn)
            if parent:
                parent_in_file = (s["file"], parent)
                if parent_in_file in referenced_qn:
                    categories["parent_covered"].append(entry)
                    continue
        # 5. The Rust file has ≥1 ledger reference (file-level audit
        #    covers remaining items in that file). This is the same
        #    coarsening as the legacy file-level ledger.
        if s["file"] in referenced_files:
            categories["file_audited"].append(entry)
            continue
        # 6. C++ counterpart file is absent from cpp_symbol_manifest.json
        #    (manifest extraction gap on the C++ side — nothing to audit
        #    against).
        rust_stem = _rust_file_to_cpp_file(s["file"])
        if rust_stem not in cpp_basenames_with_rows:
            categories["manifest_gap"].append(entry)
            continue
        unclassified.append(entry)

    return categories, unclassified


CATEGORY_ORDER = (
    "tests",
    "scaffolding",
    "parent_covered",
    "file_audited",
    "lib_files",
    "manifest_gap",
    "rust_only_architecture",
)


def emit_yaml(categories: dict[str, list[dict]], out_path: str) -> None:
    lines: list[str] = []
    lines.append("# rust_internal.yml — Rust symbols with no expected C++ origin.")
    lines.append("# Generated by scripts/ledger/build_rust_internal.py from")
    lines.append("# rust_symbol_manifest.json + MIGRATION_LEDGER.yml.")
    lines.append("#")
    lines.append("# cross_validate.py §2.1 consults this file to exempt these")
    lines.append("# symbols from the reverse-coverage warning.")
    lines.append("")
    lines.append("version: 1")
    lines.append(f"total_exempt: {sum(len(v) for v in categories.values())}")
    lines.append("")
    for cat in CATEGORY_ORDER:
        items = categories.get(cat, [])
        lines.append(f"{cat}_count: {len(items)}")
    lines.append("")

    for cat in CATEGORY_ORDER:
        items = categories.get(cat, [])
        lines.append(f"{cat}:")
        items = sorted(items, key=lambda e: (e["file"], e["symbol"], e["kind"]))
        for e in items:
            lines.append(f'  - "{e["file"]}|{e["symbol"]}|{e["kind"]}"')
        lines.append("")
    with open(out_path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))


def main() -> int:
    with open(os.path.join(ROOT, "rust_symbol_manifest.json"), "r", encoding="utf-8") as f:
        rust_symbols = json.load(f)
    with open(os.path.join(ROOT, "cpp_symbol_manifest.json"), "r", encoding="utf-8") as f:
        cpp_symbols = json.load(f)
    doc = ledger_io.load(open(os.path.join(ROOT, "MIGRATION_LEDGER.yml"), "r", encoding="utf-8").read())

    referenced: set[tuple[str, str, str]] = set()
    for r in doc.get("symbols", []):
        for rs in r.get("rust") or []:
            referenced.add((rs["file"], rs["symbol"], rs["kind"]))

    categories, unclassified = classify(rust_symbols, cpp_symbols, referenced)
    out_path = os.path.join(ROOT, "audit_patches", "rust_internal.yml")
    emit_yaml(categories, out_path)

    print(f"Wrote {out_path}")
    print(f"Manifest entries: {len(rust_symbols)}")
    print(f"Already referenced by ledger: {len(referenced)}")
    print(f"Exempted as Rust-internal: {sum(len(v) for v in categories.values())}")
    for cat, items in categories.items():
        print(f"  {cat:>18}: {len(items)}")
    print(f"Remaining unclassified gap: {len(unclassified)}")

    if unclassified:
        from collections import Counter
        by_kind = Counter(e["kind"] for e in unclassified)
        print()
        print("Unclassified by kind:")
        for k, n in by_kind.most_common():
            print(f"  {k:>25}: {n:>5}")
        by_file = Counter(e["file"] for e in unclassified)
        print()
        print("Top files with unclassified entries:")
        for f, n in by_file.most_common(10):
            print(f"  {n:>5}  {f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
