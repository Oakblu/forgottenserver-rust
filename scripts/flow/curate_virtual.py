#!/usr/bin/env python3
"""Curate dynamic virtual-dispatch edges in the flow graph.

For every in-scope Creature virtual method (all_virtuals minus trivial
accessors), adds a curated edge from Creature::<method> to each concrete
override in Player, Monster, and Npc, condition: "dyntype == <Subclass>".

Nodes are looked up in creature.yml first (src/creature.cpp implementations),
then in creature.h.yml (pure-virtual / inline declarations).

Trivial type-accessor helpers (getPlayer/getMonster/getNpc/getReceiver) are
excluded — they have no behavioral significance beyond type testing.

Idempotent: running twice produces no diff.

Usage (from repo root):
    python3 scripts/flow/curate_virtual.py
"""
from __future__ import annotations

import re
import sys
from pathlib import Path
from typing import Any

_HERE = Path(__file__).parent
sys.path.insert(0, str(_HERE.parent / "ledger"))
import ledger_io  # noqa: E402

_ROOT = _HERE.parent.parent


def _load_manifest(root: Path) -> frozenset[str]:
    """Return the set of qualified_names in cpp_symbol_manifest.json."""
    import json
    data = json.loads((root / "cpp_symbol_manifest.json").read_text())
    return frozenset(s["qualified_name"] for s in data)


# Methods that merely cast/return `this` or `nullptr` — no behavioral dispatch.
TRIVIAL_ACCESSOR_ALLOWLIST: frozenset[str] = frozenset({
    "getPlayer",
    "getNpc",
    "getMonster",
    "getReceiver",
})

# (class_prefix, source_file, header_file)
_SUBCLASSES: list[tuple[str, str, str]] = [
    ("Player",  "src/player.cpp",  "src/player.h"),
    ("Monster", "src/monster.cpp", "src/monster.h"),
    ("Npc",     "src/npc.cpp",     "src/npc.h"),
]

_NAME_RE = re.compile(r"virtual\s+(?:.*?\s+)?(\w+)\s*\(")
_OVERRIDE_RE = re.compile(
    r"\b(\w+)\s*\([^)]*\)\s*(?:const\s*)?"
    r"(?:override|final|override\s+final|final\s+override)\b"
)

# ---------------------------------------------------------------------------
# Parsing helpers
# ---------------------------------------------------------------------------

def parse_creature_virtuals(header_text: str) -> tuple[set[str], set[str]]:
    """Return (all_virtuals, pure_virtuals) from creature.h text."""
    all_v: set[str] = set()
    pure_v: set[str] = set()
    for line in header_text.splitlines():
        stripped = line.strip()
        if "virtual" not in stripped or stripped.startswith("//"):
            continue
        m = _NAME_RE.search(stripped)
        if not m or m.group(1) == "virtual":
            continue
        name = m.group(1)
        if name == "Creature":
            continue
        all_v.add(name)
        if "= 0" in stripped:
            pure_v.add(name)
    return all_v, pure_v


def parse_overrides(header_text: str, class_name: str) -> set[str]:
    """Return method names declared with override/final in a subclass header."""
    overrides: set[str] = set()
    skip = frozenset(("if", "while", "for", "switch", "return", class_name))
    for m in _OVERRIDE_RE.finditer(header_text):
        name = m.group(1)
        if name not in skip:
            overrides.add(name)
    return overrides


def in_scope_virtuals(all_virtuals: set[str]) -> set[str]:
    """Remove trivial accessors from the full virtual set."""
    return all_virtuals - TRIVIAL_ACCESSOR_ALLOWLIST


# ---------------------------------------------------------------------------
# Edge helpers (same pattern as curate_events.py)
# ---------------------------------------------------------------------------

EdgeKey = tuple[str, str, str]  # (file, qname, condition)


def _edge_key(edge: dict) -> EdgeKey:
    tgt = edge["target"]
    return (tgt["file"], tgt["qualified_name"], edge.get("condition", ""))


def _make_edge(file: str, qname: str, condition: str, order: int) -> dict:
    return {
        "target": {"file": file, "qualified_name": qname},
        "kind": "dynamic",
        "confidence": "curated",
        "condition": condition,
        "order": order,
    }


def _is_virtual_dispatch_edge(edge: dict) -> bool:
    """True if this edge was added by curate_virtual (dyntype == condition)."""
    return (
        edge.get("confidence") == "curated"
        and edge.get("kind") == "dynamic"
        and str(edge.get("condition", "")).startswith("dyntype == ")
    )


def _merge(existing: list[dict], new_curated: list[dict]) -> list[dict]:
    # Drop all stale virtual-dispatch edges first, then add the new set.
    kept = [e for e in existing if not _is_virtual_dispatch_edge(e)]
    keys = {_edge_key(e) for e in new_curated}
    kept = [e for e in kept if _edge_key(e) not in keys]
    return kept + new_curated


def _load(path: Path) -> list[dict]:
    doc = ledger_io.load(path.read_text()) or {}
    return list(doc.get("nodes") or [])


def _write(path: Path, nodes: list[dict]) -> bool:
    existing = ledger_io.load(path.read_text()) if path.exists() else {}
    if (existing or {}).get("nodes") == nodes:
        return False
    path.write_text(ledger_io.dump({"nodes": nodes}))
    return True


def _strip_virtual_dispatch_edges(nodes: list[dict]) -> bool:
    """Remove all virtual-dispatch curated edges from every node. Return True if changed."""
    changed = False
    for n in nodes:
        old = list(n.get("edges") or [])
        new = [e for e in old if not _is_virtual_dispatch_edge(e)]
        if new != old:
            n["edges"] = new
            changed = True
    return changed


def _update_nodes(nodes: list[dict], qname: str,
                  new_edges: list[dict]) -> bool:
    """Append new_edges to all nodes matching qname (handles overloads)."""
    changed = False
    for n in nodes:
        if n.get("qualified_name") == qname:
            existing = list(n.get("edges") or [])
            keys = {_edge_key(e) for e in existing}
            added = [e for e in new_edges if _edge_key(e) not in keys]
            if added:
                n["edges"] = existing + added
                changed = True
    return changed


# ---------------------------------------------------------------------------
# Build edge table
# ---------------------------------------------------------------------------

def build_dispatch_table(
    root: Path,
    scope: set[str],
    manifest: frozenset[str],
) -> dict[str, list[tuple[str, str]]]:
    """Return {method_name: [(class_prefix, cpp_file), ...]} for manifest-present overrides.

    Inline-only overrides (declared in header, not in .cpp, absent from manifest)
    are skipped so edge targets always resolve in the validator.
    """
    table: dict[str, list[tuple[str, str]]] = {m: [] for m in scope}
    for class_prefix, cpp_file, header_file in _SUBCLASSES:
        hdr = (root / "forgottenserver-upstream" / header_file).read_text()
        overrides = parse_overrides(hdr, class_prefix)
        for method in scope:
            if method in overrides:
                qname = f"{class_prefix}::{method}"
                if qname in manifest:
                    table[method].append((class_prefix, cpp_file))
    return table


# ---------------------------------------------------------------------------
# Curate creature.yml + creature.h.yml
# ---------------------------------------------------------------------------

def curate_shards(nodes_dir: Path,
                  dispatch: dict[str, list[tuple[str, str]]]) -> int:
    """Apply dispatch edges to creature.yml and creature.h.yml; return count written.

    Clears ALL existing virtual-dispatch edges first (full replace), ensuring
    no stale edges from a previous wider dispatch set survive.
    """
    cpp_path = nodes_dir / "creature.yml"
    hdr_path = nodes_dir / "creature.h.yml"
    cpp_nodes = _load(cpp_path)
    hdr_nodes = _load(hdr_path)

    # Phase 1: strip all stale virtual-dispatch edges from both shards.
    _strip_virtual_dispatch_edges(cpp_nodes)
    _strip_virtual_dispatch_edges(hdr_nodes)

    cpp_qnames = {n["qualified_name"] for n in cpp_nodes}
    hdr_qnames = {n["qualified_name"] for n in hdr_nodes}

    # Phase 2: add fresh dispatch edges.
    for method, targets in dispatch.items():
        if not targets:
            continue
        new_edges = [
            _make_edge(cpp_file, f"{cls}::{method}", f"dyntype == {cls}", i + 1)
            for i, (cls, cpp_file) in enumerate(targets)
        ]
        base_qname = f"Creature::{method}"
        if base_qname in cpp_qnames:
            _update_nodes(cpp_nodes, base_qname, new_edges)
        elif base_qname in hdr_qnames:
            _update_nodes(hdr_nodes, base_qname, new_edges)
        # if neither exists, skip silently (will surface in check)

    written = 0
    if _write(cpp_path, cpp_nodes):
        written += 1
    if _write(hdr_path, hdr_nodes):
        written += 1
    return written


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def curate(root: Path = _ROOT) -> None:
    nodes_dir = root / "flow_graph" / "nodes"

    creature_h = (root / "forgottenserver-upstream" / "src" / "creature.h").read_text()
    all_virtuals, _pure = parse_creature_virtuals(creature_h)
    scope = in_scope_virtuals(all_virtuals)

    manifest = _load_manifest(root)
    dispatch = build_dispatch_table(root, scope, manifest)
    total_edges = sum(len(v) for v in dispatch.values())

    written = curate_shards(nodes_dir, dispatch)
    print(
        f"Virtual curation: {len(scope)} in-scope virtuals, "
        f"{total_edges} dispatch edges across Player/Monster/Npc — "
        f"{written} shard(s) written"
    )


if __name__ == "__main__":
    curate()
