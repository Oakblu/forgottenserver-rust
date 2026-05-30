#!/usr/bin/env python3
"""Curate dynamic virtual-dispatch edges in the flow graph.

For every in-scope Creature virtual (pure-virtuals + behavioral non-trivials),
adds a curated edge from Creature::<method> to each concrete override in
Player, Monster, and Npc, condition: "dyntype == <Subclass>".

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

_VIRTUAL_RE = re.compile(r"\bvirtual\b")
_NAME_RE = re.compile(r"virtual\s+(?:.*?\s+)?(\w+)\s*\(")
_OVERRIDE_RE = re.compile(r"\b(\w+)\s*\([^)]*\)\s*(?:const\s*)?(?:override|final|override\s+final|final\s+override)\b")

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
    e: dict[str, Any] = {
        "target": {"file": file, "qualified_name": qname},
        "kind": "dynamic",
        "confidence": "curated",
        "condition": condition,
        "order": order,
    }
    return e


def _merge(existing: list[dict], new_curated: list[dict]) -> list[dict]:
    keys = {_edge_key(e) for e in new_curated}
    kept = [e for e in existing if _edge_key(e) not in keys]
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


def _update(nodes: list[dict], file: str, qname: str,
            new_edges: list[dict]) -> bool:
    changed = False
    for n in nodes:
        if n.get("file") == file and n.get("qualified_name") == qname:
            old_edges = list(n.get("edges") or [])
            n["edges"] = _merge(old_edges, new_edges)
            if n["edges"] != old_edges:
                changed = True
    return changed


# ---------------------------------------------------------------------------
# Build edge table
# ---------------------------------------------------------------------------

def build_dispatch_table(
    root: Path,
    scope: set[str],
) -> dict[str, list[tuple[str, str, str]]]:
    """Return {method_name: [(subclass, cpp_file, header_file), ...]}."""
    table: dict[str, list[tuple[str, str, str]]] = {m: [] for m in scope}
    for class_prefix, cpp_file, header_file in _SUBCLASSES:
        hdr = (root / "forgottenserver-upstream" / header_file).read_text()
        overrides = parse_overrides(hdr, class_prefix)
        for method in scope:
            if method in overrides:
                table[method].append((class_prefix, cpp_file, header_file))
    return table


# ---------------------------------------------------------------------------
# Curate creature.yml
# ---------------------------------------------------------------------------

def _update_creature(nodes_dir: Path, dispatch: dict[str, list[tuple[str, str, str]]]) -> bool:
    path = nodes_dir / "creature.yml"
    nodes = _load(path)
    changed = False
    for method, targets in dispatch.items():
        if not targets:
            continue
        new_edges = []
        for order, (class_prefix, cpp_file, _) in enumerate(targets, 1):
            qname = f"{class_prefix}::{method}"
            new_edges.append(_make_edge(
                cpp_file, qname,
                f"dyntype == {class_prefix}", order,
            ))
        changed |= _update(nodes, "src/creature.cpp", f"Creature::{method}", new_edges)
    return _write(path, nodes) or changed


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def curate(root: Path = _ROOT) -> None:
    nodes_dir = root / "flow_graph" / "nodes"

    creature_h = (root / "forgottenserver-upstream" / "src" / "creature.h").read_text()
    all_virtuals, _pure = parse_creature_virtuals(creature_h)
    scope = in_scope_virtuals(all_virtuals)

    dispatch = build_dispatch_table(root, scope)
    total_edges = sum(len(v) for v in dispatch.values())

    written = _update_creature(nodes_dir, dispatch)
    print(
        f"Virtual curation: {len(scope)} in-scope virtuals, "
        f"{total_edges} dispatch edges across Player/Monster/Npc — "
        f"{'1 shard written' if written else '0 shards written (already up-to-date)'}"
    )


if __name__ == "__main__":
    curate()
