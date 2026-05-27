#!/usr/bin/env python3
"""Build audit_patches/layer_scopes.json from DEPENDENCY_GRAPH.md.

Each layer maps to a set of C++ files (.h and .cpp). The graph lists
headers; we expand each <stem>.h to {<stem>.h, <stem>.cpp} when the
.cpp exists. We also resolve which ledger row ids fall in each scope
by reading the seeded MIGRATION_LEDGER.yml.

This file is the contract a layer-N subagent operates against:
    {
      "0": {
        "layer": 0,
        "title": "Primitives",
        "cpp_files": ["src/position.h", "src/position.cpp", ...],
        "row_count": <n>,
        "row_ids": [...]
      },
      ...
    }
"""
from __future__ import annotations

import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import ledger_io  # noqa: E402

ROOT = os.path.abspath(os.path.join(HERE, os.pardir, os.pardir))
GRAPH = os.path.join(ROOT, "DEPENDENCY_GRAPH.md")
LEDGER = os.path.join(ROOT, "MIGRATION_LEDGER.yml")
CPP_SRC = os.path.join(ROOT, os.pardir, "forgottenserver", "src")
OUT = os.path.join(ROOT, "audit_patches", "layer_scopes.json")

# Files that don't appear in DEPENDENCY_GRAPH.md but must still be
# audited. Each entry maps a src/<path> to its logical layer.
# - game.{cpp,h}: orchestrator, depends on Layers 0–9 but is itself
#   imported by Layer 10 protocol code; place at Layer 10 alongside
#   the protocol crate so the auditor can see both sides.
# - main.cpp: top-level entry point → Layer 11.
# - otpch.h: precompiled header, no real includes → Layer 0.
EXTRA_FILES = {
    "src/game.h":   "10",
    "src/game.cpp": "10",
    "src/main.cpp": "11",
    "src/otpch.h":  "0",
}

LAYER_RE = re.compile(r"^## Layer (\d+) — (.+)$")
ROW_RE = re.compile(r"^\|\s+`([^`]+)`")
TABLE_HEADER_RE = re.compile(r"^\|\s+File\s+\|")


def parse_graph(path: str) -> dict[str, dict]:
    layers: dict[str, dict] = {}
    current: dict | None = None
    in_table = False
    with open(path, "r", encoding="utf-8") as f:
        for raw in f:
            line = raw.rstrip("\n")
            m = LAYER_RE.match(line)
            if m:
                num = m.group(1)
                title = m.group(2)
                current = {"layer": int(num), "title": title, "header_files": []}
                layers[num] = current
                in_table = False
                continue
            if line.startswith("## "):
                current = None
                in_table = False
                continue
            if current is None:
                continue
            if TABLE_HEADER_RE.match(line):
                in_table = True
                continue
            if in_table and line.strip().startswith("| ---"):
                continue
            m = ROW_RE.match(line)
            if m and current is not None:
                fn = m.group(1).strip()
                if "/" not in fn and not fn.endswith((".h", ".cpp")):
                    continue  # skip non-source rows
                current["header_files"].append(fn)
    return layers


def expand_to_src(header_files: list[str]) -> list[str]:
    """Given headers like 'position.h' or 'http/cacheinfo.h', return the
    full src/ paths for both header and any matching .cpp."""
    out: list[str] = []
    for hf in header_files:
        # Headers in the graph are relative to src/ (e.g. "position.h" or
        # "http/cacheinfo.h"). Some are bare (e.g. "tools.h*" — strip *).
        hf = hf.rstrip("*").strip()
        rel = hf if hf.startswith("http/") else hf
        h_path = f"src/{rel}"
        out.append(h_path)
        # Match .cpp neighbour if present.
        cpp_rel = rel[:-2] + ".cpp" if rel.endswith(".h") else None
        if cpp_rel:
            cpp_path = f"src/{cpp_rel}"
            full = os.path.normpath(os.path.join(CPP_SRC, cpp_rel))
            if os.path.exists(full):
                out.append(cpp_path)
    return out


def main() -> int:
    layers = parse_graph(GRAPH)
    with open(LEDGER, "r", encoding="utf-8") as f:
        doc = ledger_io.load(f.read())

    rows = doc.get("symbols", [])
    rows_by_file: dict[str, list[str]] = {}
    for r in rows:
        rows_by_file.setdefault(r["cpp"]["file"], []).append(r["id"])

    scoped_files: set[str] = set()
    out: dict[str, dict] = {}
    for num, info in layers.items():
        cpp_files = expand_to_src(info["header_files"])
        cpp_files = [f for f in cpp_files if f in rows_by_file]
        seen: set[str] = set()
        cpp_files = [f for f in cpp_files if not (f in seen or seen.add(f))]
        cpp_files = [f for f in cpp_files if f not in scoped_files]
        # Add any EXTRA_FILES assigned to this layer (skip if file
        # already taken by a lower layer or doesn't appear in ledger).
        for f, target_layer in EXTRA_FILES.items():
            if target_layer == num and f in rows_by_file and f not in scoped_files:
                cpp_files.append(f)
        scoped_files.update(cpp_files)
        row_ids: list[str] = []
        for f in cpp_files:
            row_ids.extend(rows_by_file.get(f, []))
        out[num] = {
            "layer": info["layer"],
            "title": info["title"],
            "cpp_files": cpp_files,
            "row_count": len(row_ids),
            "row_ids": row_ids,
        }

    uncategorized = sorted(set(rows_by_file) - scoped_files)
    if uncategorized:
        ids: list[str] = []
        for f in uncategorized:
            ids.extend(rows_by_file[f])
        out["99"] = {
            "layer": 99,
            "title": "Uncategorized (not present in DEPENDENCY_GRAPH.md)",
            "cpp_files": uncategorized,
            "row_count": len(ids),
            "row_ids": ids,
        }

    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8") as f:
        json.dump(out, f, indent=2)

    total = sum(info["row_count"] for info in out.values())
    print(f"Wrote {OUT}")
    for num in sorted(out, key=lambda x: int(x)):
        info = out[num]
        print(f"  Layer {num:>2} ({info['title'][:40]:<40}): {len(info['cpp_files']):>3} files, {info['row_count']:>5} rows")
    print(f"  Total: {total} rows / {len(rows)} ledger rows")
    if total != len(rows):
        print(f"  WARNING: layer rows ({total}) != ledger rows ({len(rows)})")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
